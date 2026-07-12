#!/usr/bin/env python3
"""Generate a tiny Mamba-3 SISO fixture from the pinned upstream checkout.

This script is developer tooling only. It is deliberately not invoked by Cargo,
does not download packages or weights, and requires the official CUDA runtime.
"""

import argparse
import hashlib
import importlib
import json
import os
import struct
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

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


class FixtureDigest:
    def __init__(self) -> None:
        self.value = 0xCBF29CE484222325

    def bytes(self, value: bytes) -> None:
        for byte in value:
            self.value = ((self.value ^ byte) * 0x100000001B3) & ((1 << 64) - 1)

    def boolean(self, value: bool) -> None:
        self.bytes(bytes([int(value)]))

    def u64(self, value: int) -> None:
        self.bytes(struct.pack("<Q", value))

    def f32(self, value: float) -> None:
        self.bytes(struct.pack("<f", value))

    def string(self, value: str) -> None:
        raw = value.encode("utf-8")
        self.u64(len(raw))
        self.bytes(raw)

    def optional_string(self, value: Optional[str]) -> None:
        self.boolean(value is not None)
        if value is not None:
            self.string(value)


def digest_precision(digest: FixtureDigest, value: str) -> None:
    digest.bytes(bytes([{"F32": 0, "F64Unsupported": 1}[value]]))


def digest_tensor_1d(digest: FixtureDigest, value: dict) -> None:
    digest.u64(value["dim"])
    digest.boolean(value["paper_only"])
    digest.u64(len(value["values"]))
    for item in value["values"]:
        digest.f32(item)


def digest_tensor_2d(digest: FixtureDigest, value: dict) -> None:
    digest.u64(value["rows"])
    digest.u64(value["cols"])
    digest.boolean(value["paper_only"])
    digest.u64(len(value["values"]))
    for item in value["values"]:
        digest.f32(item)


def digest_state(digest: FixtureDigest, value: dict) -> None:
    digest_tensor_2d(digest, value["angle_state"])
    digest_tensor_1d(digest, value["ssm_state"])
    digest_tensor_2d(digest, value["previous_key"])
    digest_tensor_2d(digest, value["previous_value"])
    digest.u64(value["step_index"])


def digest_tensor_list(digest: FixtureDigest, values: list[dict]) -> None:
    digest.u64(len(values))
    for value in values:
        digest_tensor_1d(digest, value)


def fixture_digest(payload: dict) -> str:
    """Typed binary FNV-1a encoding shared with Mamba3SisoReferenceFixtureV0."""
    digest = FixtureDigest()
    metadata, provenance, config = payload["metadata"], payload["provenance"], payload["metadata"]["config"]
    digest.u64(payload["format_version"])
    digest.u64(metadata["format_version"])
    digest.string(metadata["architecture"])
    for key in ["input_dim", "state_dim", "head_dim", "expansion"]:
        digest.u64(config[key])
    digest.bytes(bytes([{"Half": 0, "Full": 1}[config["rope_fraction"]]]))
    digest.f32(config["norm_epsilon"])
    digest.f32(config["a_floor"])
    digest.u64(config["mimo_rank"])
    digest_precision(digest, config["precision"])
    digest.boolean(config["short_convolution_enabled"])
    digest.u64(metadata["parameter_count"])
    digest.string(metadata["reference_commit"])
    digest.boolean(metadata["reference_only"])
    for key in ["case_id", "official_repository", "official_commit"]:
        digest.string(provenance[key])
    digest.u64(len(provenance["official_source_paths"]))
    for value in provenance["official_source_paths"]:
        digest.string(value)
    digest.u64(len(provenance["official_source_hashes"]))
    for value in provenance["official_source_hashes"]:
        digest.string(value["path"])
        digest.string(value["sha256"])
    digest.string(provenance["paper_identifier"])
    digest.string(provenance["generator_sha256"])
    digest.optional_string(provenance.get("instrumentation_patch_sha256"))
    digest.string(provenance["python_version"])
    digest.string(provenance["pytorch_version"])
    digest_precision(digest, provenance["dtype"])
    digest.string(provenance["device"])
    digest.optional_string(provenance.get("cuda_runtime"))
    digest.optional_string(provenance.get("triton_version"))
    digest.optional_string(provenance.get("cute_version"))
    digest.u64(len(provenance["parameter_ordering"]))
    for value in provenance["parameter_ordering"]:
        digest.string(value)
    digest.u64(provenance["parameter_count"])
    parameters = payload["parameters"]
    digest_tensor_2d(digest, parameters["input_projection"])
    for key in ["dt_bias"]:
        digest_tensor_1d(digest, parameters[key])
    for key in ["b_bias", "c_bias"]:
        digest_tensor_2d(digest, parameters[key])
    for key in ["b_norm_scale", "c_norm_scale", "skip"]:
        digest_tensor_1d(digest, parameters[key])
    digest_tensor_2d(digest, parameters["output_projection"])
    digest_state(digest, payload["initial_state"])
    digest_tensor_list(digest, payload["input"])
    digest.boolean(payload.get("expected_output") is not None)
    if payload.get("expected_output") is not None:
        digest_tensor_list(digest, payload["expected_output"])
    digest.boolean(payload.get("expected_state") is not None)
    if payload.get("expected_state") is not None:
        digest.u64(len(payload["expected_state"]))
        for state in payload["expected_state"]:
            digest_state(digest, state)
    for key in ["absolute", "relative", "state_absolute"]:
        digest.f32(payload["tolerance"][key])
    return f"fnv1a64-{digest.value:016x}"


def require_environment(reference_root: Path) -> tuple[object, object]:
    if not reference_root.is_dir():
        fail("MAMBA_REFERENCE_ROOT must name a local official checkout")
    origin = subprocess.check_output(
        ["git", "-C", str(reference_root), "remote", "get-url", "origin"], text=True
    ).strip()
    if not origin.endswith("state-spaces/mamba.git"):
        fail(f"official checkout origin is not state-spaces/mamba: {origin}")
    commit = subprocess.check_output(
        ["git", "-C", str(reference_root), "rev-parse", "HEAD"], text=True
    ).strip()
    if commit != COMMIT:
        fail(f"official checkout commit is {commit}, expected {COMMIT}")
    if not all((reference_root / source).is_file() for source in SOURCE_PATHS):
        fail("official source paths are missing")
    dirty = subprocess.check_output(
        ["git", "-C", str(reference_root), "status", "--porcelain"], text=True
    ).strip()
    if dirty:
        fail("official checkout is dirty; instrumentation requires recorded patch provenance")
    sys.path.insert(0, str(reference_root))
    try:
        torch = importlib.import_module("torch")
        mamba3 = importlib.import_module("mamba_ssm.modules.mamba3")
    except ModuleNotFoundError as error:
        fail(f"official Python dependency is unavailable: {error}")
    if not torch.cuda.is_available():
        fail("the pinned official step path requires a CUDA-capable reference environment")
    return torch, mamba3


def source_hashes(reference_root: Path) -> list[dict]:
    return [
        {"path": source, "sha256": hashlib.sha256((reference_root / source).read_bytes()).hexdigest()}
        for source in SOURCE_PATHS
    ]


def version_or_none(module_name: str) -> Optional[str]:
    try:
        module = importlib.import_module(module_name)
    except ModuleNotFoundError:
        return None
    return getattr(module, "__version__", None)


def reject_unsafe_output_path(output_path: Path) -> None:
    repository_root = Path(__file__).resolve().parents[2]
    resolved = output_path.resolve()
    for restricted in [repository_root / "src", repository_root / "docs"]:
        try:
            resolved.relative_to(restricted.resolve())
        except ValueError:
            continue
        fail("output path must be test fixture data, not source or documentation")


def write_fixture(output_path: Path, reference_root: Path, torch: object, mamba3: object, case_id: str) -> None:
    seed = {"A": 71, "B": 73, "C": 79, "D": 83, "E": 89}[case_id]
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
    case_inputs = {
        "A": [[0.2, -0.1], [0.4, 0.3], [-0.3, 0.5]],
        "B": [[0.1, -0.2], [0.3, 0.4], [-0.2, 0.5]],
        "C": [[0.9, -0.4], [-0.1, 0.7], [0.2, 0.3]],
        "D": [[0.1, 0.2], [0.2, -0.3], [0.5, 0.1], [-0.4, 0.6]],
        "E": [[0.1, -0.2], [0.3, 0.4], [-0.2, 0.5]],
    }
    if case_id == "A":
        parameters["input_projection"]["values"][-config["input_dim"]:] = [0.0, 0.0]
    if case_id == "B":
        parameters["input_projection"]["values"][-config["input_dim"]:] = [0.25, -0.15]
    input_rows = [tensor_1d(row) for row in case_inputs[case_id]]

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
    if case_id == "B":
        angle.fill_(0.25)
        ssm.fill_(0.05)
        key.fill_(0.10)
        value.fill_(0.20)
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
            if not all(torch.isfinite(tensor).all().item() for tensor in [output, angle, ssm, key, value]):
                fail(f"official step {step} produced a non-finite value")
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
            "case_id": case_id,
            "official_repository": "state-spaces/mamba",
            "official_commit": COMMIT,
            "official_source_paths": SOURCE_PATHS,
            "official_source_hashes": source_hashes(reference_root),
            "paper_identifier": "arXiv:2603.15569",
            "generator_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "instrumentation_patch_sha256": None,
            "python_version": sys.version.split()[0],
            "pytorch_version": torch.__version__,
            "dtype": "F32",
            "device": torch.cuda.get_device_name(0),
            "cuda_runtime": torch.version.cuda,
            "triton_version": version_or_none("triton"),
            "cute_version": version_or_none("cutlass"),
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
    with tempfile.NamedTemporaryFile("w", dir=output_path.parent, delete=False) as temporary:
        temporary.write(json.dumps(fixture, indent=2) + "\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(output_path)
    print(f"wrote {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", default=os.environ.get("MAMBA_REFERENCE_ROOT"))
    parser.add_argument("--output", required=True)
    parser.add_argument("--case", choices=["A", "B", "C", "D", "E"], required=True)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()
    if not args.reference_root:
        fail("pass --reference-root or set MAMBA_REFERENCE_ROOT")
    reference_root = Path(args.reference_root)
    output_path = Path(args.output)
    reject_unsafe_output_path(output_path)
    if output_path.exists() and not args.force:
        fail("output already exists; pass --force to replace an existing fixture")
    torch, mamba3 = require_environment(reference_root)
    write_fixture(output_path, reference_root, torch, mamba3, args.case)


if __name__ == "__main__":
    main()
