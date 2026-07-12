#!/usr/bin/env python3
"""Generate a tiny Mamba-3 SISO fixture from the pinned upstream checkout.

This script is developer tooling only. It is deliberately not invoked by Cargo,
does not download packages or weights, and requires the official CUDA runtime.
"""

import argparse
import importlib
import json
import os
import subprocess
import sys
from pathlib import Path

COMMIT = "f577286d052741c35d39cd43bdc3fad27120f22c"
SOURCE_PATHS = [
    "mamba_ssm/modules/mamba3.py",
    "mamba_ssm/ops/triton/mamba3/mamba3_siso_step.py",
]
PARAMETER_ORDERING = [
    "input_projection",
    "dt_bias",
    "b_bias",
    "c_bias",
    "b_norm_scale",
    "c_norm_scale",
    "skip",
    "output_projection",
]


def fail(message: str) -> None:
    raise SystemExit(f"oracle generation failed: {message}")


def deterministic_value(seed: int, index: int, salt: int) -> float:
    mixed = (seed * 1_103_515_245 + (index + 1) * 12_345 + salt * 97) & ((1 << 64) - 1)
    return ((mixed % 97) - 48) / 192.0


def matrix(rows: int, cols: int, seed: int, salt: int) -> list[float]:
    return [deterministic_value(seed, index, salt) for index in range(rows * cols)]


def tensor_1d(values: list[float]) -> dict:
    return {"values": values, "dim": len(values), "paper_only": True}


def tensor_2d(rows: int, cols: int, values: list[float]) -> dict:
    return {"values": values, "rows": rows, "cols": cols, "paper_only": True}


def fixture_digest(payload: dict) -> str:
    """Use the compact deterministic payload representation expected by the Rust fixture API."""
    canonical = json.dumps(payload, separators=(",", ":"), ensure_ascii=True)
    return "fnv1a64-" + format(
        _fnv1a64(canonical.encode("utf-8")), "016x"
    )


def _fnv1a64(data: bytes) -> int:
    value = 0xCBF29CE484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001B3) & ((1 << 64) - 1)
    return value


def require_environment(reference_root: Path) -> tuple[object, object]:
    if not reference_root.is_dir():
        fail("MAMBA_REFERENCE_ROOT must name a local official checkout")
    commit = subprocess.check_output(
        ["git", "-C", str(reference_root), "rev-parse", "HEAD"], text=True
    ).strip()
    if commit != COMMIT:
        fail(f"official checkout commit is {commit}, expected {COMMIT}")
    if not all((reference_root / source).is_file() for source in SOURCE_PATHS):
        fail("official source paths are missing")
    sys.path.insert(0, str(reference_root))
    try:
        torch = importlib.import_module("torch")
        mamba3 = importlib.import_module("mamba_ssm.modules.mamba3")
    except ModuleNotFoundError as error:
        fail(f"official Python dependency is unavailable: {error}")
    if not torch.cuda.is_available():
        fail("the pinned official step path requires a CUDA-capable reference environment")
    return torch, mamba3


def write_fixture(output_path: Path, torch: object, mamba3: object) -> None:
    seed = 73
    config = {
        "input_dim": 2,
        "state_dim": 4,
        "head_dim": 2,
        "expansion": 1,
        "rope_fraction": "Half",
        "norm_epsilon": 1e-5,
        "a_floor": 1e-4,
        "mimo_rank": 1,
        "precision": "F32",
        "short_convolution_enabled": False,
    }
    d_inner = config["input_dim"] * config["expansion"]
    nheads = d_inner // config["head_dim"]
    nangles = (config["state_dim"] // 2) // 2
    projection_rows = 2 * d_inner + 2 * config["state_dim"] + 3 * nheads + nangles
    parameters = {
        "input_projection": tensor_2d(
            projection_rows,
            config["input_dim"],
            matrix(projection_rows, config["input_dim"], seed, 101),
        ),
        "dt_bias": tensor_1d([0.0] * nheads),
        "b_bias": tensor_2d(nheads, config["state_dim"], matrix(nheads, config["state_dim"], seed, 102)),
        "c_bias": tensor_2d(nheads, config["state_dim"], matrix(nheads, config["state_dim"], seed, 103)),
        "b_norm_scale": tensor_1d([1.0 + deterministic_value(seed, index, 104) * 0.1 for index in range(config["state_dim"])]),
        "c_norm_scale": tensor_1d([1.0 + deterministic_value(seed, index, 105) * 0.1 for index in range(config["state_dim"])]),
        "skip": tensor_1d([deterministic_value(seed, index, 106) for index in range(nheads)]),
        "output_projection": tensor_2d(config["input_dim"], d_inner, matrix(config["input_dim"], d_inner, seed, 107)),
    }
    input_rows = [tensor_1d([0.1, -0.2]), tensor_1d([0.3, 0.4]), tensor_1d([-0.2, 0.5])]

    model = mamba3.Mamba3(
        d_model=config["input_dim"],
        d_state=config["state_dim"],
        expand=config["expansion"],
        headdim=config["head_dim"],
        ngroups=1,
        rope_fraction=0.5,
        A_floor=config["a_floor"],
        is_outproj_norm=False,
        is_mimo=False,
        device="cuda",
        dtype=torch.float32,
    ).eval()
    with torch.no_grad():
        model.in_proj.weight.copy_(torch.tensor(parameters["input_projection"]["values"], device="cuda").reshape(projection_rows, config["input_dim"]))
        model.dt_bias.copy_(torch.tensor(parameters["dt_bias"]["values"], device="cuda"))
        model.B_bias.copy_(torch.tensor(parameters["b_bias"]["values"], device="cuda").reshape(nheads, 1, config["state_dim"]))
        model.C_bias.copy_(torch.tensor(parameters["c_bias"]["values"], device="cuda").reshape(nheads, 1, config["state_dim"]))
        model.B_norm.weight.copy_(torch.tensor(parameters["b_norm_scale"]["values"], device="cuda"))
        model.C_norm.weight.copy_(torch.tensor(parameters["c_norm_scale"]["values"], device="cuda"))
        model.D.copy_(torch.tensor(parameters["skip"]["values"], device="cuda"))
        model.out_proj.weight.copy_(torch.tensor(parameters["output_projection"]["values"], device="cuda").reshape(config["input_dim"], d_inner))

    angle, ssm, key, value = model.allocate_inference_cache(1, len(input_rows), device="cuda", dtype=torch.float32)
    initial_state = {
        "angle_state": tensor_2d(nheads, nangles, angle[0].cpu().flatten().tolist()),
        "ssm_state": tensor_1d(ssm[0].cpu().flatten().tolist()),
        "previous_key": tensor_2d(nheads, config["state_dim"], key[0, 0].cpu().flatten().tolist()),
        "previous_value": tensor_2d(nheads, config["head_dim"], value[0].cpu().flatten().tolist()),
        "step_index": 0,
    }
    expected_output, expected_state = [], []
    with torch.no_grad():
        for step, row in enumerate(input_rows, start=1):
            output, angle, ssm, key, value = model.step(
                torch.tensor([row["values"]], device="cuda"), angle, ssm, key, value
            )
            expected_output.append(tensor_1d(output[0].float().cpu().tolist()))
            expected_state.append({
                "angle_state": tensor_2d(nheads, nangles, angle[0].float().cpu().flatten().tolist()),
                "ssm_state": tensor_1d(ssm[0].float().cpu().flatten().tolist()),
                "previous_key": tensor_2d(nheads, config["state_dim"], key[0, 0].float().cpu().flatten().tolist()),
                "previous_value": tensor_2d(nheads, config["head_dim"], value[0].float().cpu().flatten().tolist()),
                "step_index": step,
            })

    parameter_count = sum(len(value["values"]) for value in parameters.values())
    fixture = {
        "format_version": 1,
        "metadata": {
            "format_version": 1,
            "architecture": "mamba3-siso-reference-v0",
            "config": config,
            "parameter_count": parameter_count,
            "reference_commit": COMMIT,
            "reference_only": True,
        },
        "provenance": {
            "official_repository": "state-spaces/mamba",
            "official_commit": COMMIT,
            "official_source_paths": SOURCE_PATHS,
            "paper_identifier": "arXiv:2603.15569",
            "python_version": sys.version.split()[0],
            "pytorch_version": torch.__version__,
            "dtype": "F32",
            "device": torch.cuda.get_device_name(0),
            "parameter_ordering": PARAMETER_ORDERING,
            "parameter_count": parameter_count,
            "digest": "",
        },
        "parameters": parameters,
        "initial_state": initial_state,
        "input": input_rows,
        "expected_output": expected_output,
        "expected_state": expected_state,
        "tolerance": {"absolute": 1e-5, "relative": 1e-5, "state_absolute": 1e-5},
    }
    fixture["provenance"]["digest"] = fixture_digest(fixture)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(fixture, indent=2) + "\n")
    print(f"wrote {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", default=os.environ.get("MAMBA_REFERENCE_ROOT"))
    parser.add_argument("--output", default="tests/fixtures/mamba3/official_siso_reference_v0.json")
    args = parser.parse_args()
    if not args.reference_root:
        fail("pass --reference-root or set MAMBA_REFERENCE_ROOT")
    torch, mamba3 = require_environment(Path(args.reference_root))
    write_fixture(Path(args.output), torch, mamba3)


if __name__ == "__main__":
    main()
