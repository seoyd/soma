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
OFFICIAL_REPOSITORY_SUFFIX = "state-spaces/mamba.git"
SOURCE_HASHES = {
    "mamba_ssm/modules/mamba3.py": "930c3dfa04dea8444b1c9ee8b6ac9cbbc7ef492dc5ae1f4c83051ef953eca33c",
    "mamba_ssm/ops/cute/mamba3/mamba3_step_fn.py": "5c82f3936308cfc90bb3bcdfd410c60fc80dfc3afe72c4338831341398cd7ca4",
    "mamba_ssm/ops/triton/layernorm_gated.py": "eb6252e247b90f1c8a75946efbc1a221e0c4da701b6757ddae49f3495cf7a42f",
    "mamba_ssm/ops/triton/mamba3/mamba3_mimo_rotary_step.py": "e117b5da3d2ddfcfa66673a88dfd88a6d688100291d3b6a1b0b3c24c82ef79d6",
}
SOURCE_PATHS = list(SOURCE_HASHES)
CASE_IDS = ("A", "B", "C", "D", "E")
CASE_SEEDS = {"A": 71, "B": 73, "C": 79, "D": 83, "E": 89}
ORACLE_CONFIG = {
    "input_dim": 2,
    "state_dim": 8,
    "head_dim": 2,
    "expansion": 1,
    "rope_fraction": "Half",
    "norm_epsilon": 1e-5,
    "a_floor": 1e-4,
    "mimo_rank": 1,
    "precision": "F32",
    "short_convolution_enabled": False,
}
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


def checked_command(reference_root: Path, arguments: list[str]) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(reference_root), *arguments], text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError) as error:
        fail(f"official checkout git verification failed: {error}")


def verify_checkout(reference_root: Path) -> None:
    if not reference_root.is_dir():
        fail("SOMA_MAMBA_OFFICIAL_DIR must name a local official checkout")
    origin = checked_command(reference_root, ["remote", "get-url", "origin"])
    if not origin.endswith(OFFICIAL_REPOSITORY_SUFFIX):
        fail("official checkout origin is not state-spaces/mamba")
    commit = checked_command(reference_root, ["rev-parse", "HEAD"])
    if commit != COMMIT:
        fail("official checkout is not at the pinned commit")
    dirty = checked_command(reference_root, ["status", "--porcelain"])
    if dirty:
        fail("official checkout is dirty; use an unmodified pinned checkout")
    for source, expected_hash in SOURCE_HASHES.items():
        source_path = reference_root / source
        if not source_path.is_file():
            fail("required official source path is missing")
        actual_hash = hashlib.sha256(source_path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            fail(f"official source hash mismatch for {source}")


def require_environment(reference_root: Path, device: str, dtype: str) -> tuple[object, object]:
    verify_checkout(reference_root)
    if dtype != "float32":
        fail("the Rust fixture contract currently accepts only a faithful float32 oracle")
    sys.path.insert(0, str(reference_root))
    try:
        torch = importlib.import_module("torch")
        mamba3 = importlib.import_module("mamba_ssm.modules.mamba3")
    except ModuleNotFoundError as error:
        fail(f"official Python dependency is unavailable: {error}")
    if not torch.cuda.is_available():
        fail("the pinned official step path requires a CUDA-capable reference environment")
    if not device.startswith("cuda:") or not device.removeprefix("cuda:").isdigit():
        fail("device must use the cuda:<index> form")
    device_index = int(device.removeprefix("cuda:"))
    if device_index >= torch.cuda.device_count():
        fail("selected CUDA device is unavailable")
    if getattr(mamba3, "mamba3_step_fn", None) is None:
        fail("the official CuTe Mamba-3 step route is unavailable")
    return torch, mamba3


def source_hashes(reference_root: Path) -> list[dict]:
    return [{"path": source, "sha256": SOURCE_HASHES[source]} for source in SOURCE_PATHS]


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


def assert_parameter_mapping(parameters: dict, config: dict, projection_rows: int) -> None:
    expected = {
        "input_projection": projection_rows * config["input_dim"],
        "dt_bias": config["input_dim"] * config["expansion"] // config["head_dim"],
        "b_bias": (config["input_dim"] * config["expansion"] // config["head_dim"]) * config["state_dim"],
        "c_bias": (config["input_dim"] * config["expansion"] // config["head_dim"]) * config["state_dim"],
        "b_norm_scale": config["state_dim"],
        "c_norm_scale": config["state_dim"],
        "skip": config["input_dim"] * config["expansion"] // config["head_dim"],
        "output_projection": config["input_dim"] * (config["input_dim"] * config["expansion"]),
    }
    if list(parameters) != PARAMETER_ORDERING or any(
        len(parameters[name]["values"]) != count for name, count in expected.items()
    ):
        fail("parameter shape or ordering does not match the official SISO mapping")


def capture_state(angle: object, ssm: object, key: object, value: object, config: dict, nheads: int, nangles: int, step_index: int) -> dict:
    expected_shapes = {
        "angle": (1, nheads, nangles),
        "ssm": (1, nheads, config["head_dim"], config["state_dim"]),
        "key": (1, 1, nheads, config["state_dim"]),
        "value": (1, nheads, config["head_dim"]),
    }
    actual_shapes = {"angle": tuple(angle.shape), "ssm": tuple(ssm.shape), "key": tuple(key.shape), "value": tuple(value.shape)}
    if actual_shapes != expected_shapes:
        fail("official cache state shape does not match the selected SISO mapping")
    return {
        "angle_state": tensor_2d(nheads, nangles, angle[0].float().cpu().flatten().tolist()),
        "ssm_state": tensor_1d(ssm[0].float().cpu().flatten().tolist()),
        "previous_key": tensor_2d(nheads, config["state_dim"], key[0, 0].float().cpu().flatten().tolist()),
        "previous_value": tensor_2d(nheads, config["head_dim"], value[0].float().cpu().flatten().tolist()),
        "step_index": step_index,
    }


def validate_fixture_payload(fixture: dict) -> None:
    if fixture_digest(fixture) != fixture["provenance"]["digest"]:
        fail("fixture digest validation failed before write")
    encoded = json.dumps(fixture, allow_nan=False)
    if not encoded:
        fail("fixture serialization produced no content")


def write_fixture(output_path: Path, reference_root: Path, torch: object, mamba3: object, case_id: str, device: str, seed: int, instrumentation_patch_sha256: Optional[str]) -> None:
    config = ORACLE_CONFIG.copy()
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
    assert_parameter_mapping(parameters, config, projection_rows)
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
        device=device,
        dtype=torch.float32,
    ).eval()
    with torch.no_grad():
        model.in_proj.weight.copy_(torch.tensor(parameters["input_projection"]["values"], device=device).reshape(projection_rows, config["input_dim"]))
        model.dt_bias.copy_(torch.tensor(parameters["dt_bias"]["values"], device=device))
        model.B_bias.copy_(torch.tensor(parameters["b_bias"]["values"], device=device).reshape(nheads, 1, config["state_dim"]))
        model.C_bias.copy_(torch.tensor(parameters["c_bias"]["values"], device=device).reshape(nheads, 1, config["state_dim"]))
        model.B_norm.weight.copy_(torch.tensor(parameters["b_norm_scale"]["values"], device=device))
        model.C_norm.weight.copy_(torch.tensor(parameters["c_norm_scale"]["values"], device=device))
        model.D.copy_(torch.tensor(parameters["skip"]["values"], device=device))
        model.out_proj.weight.copy_(torch.tensor(parameters["output_projection"]["values"], device=device).reshape(config["input_dim"], d_inner))

    angle, ssm, key, value = model.allocate_inference_cache(1, len(input_rows), device=device, dtype=torch.float32)
    if case_id == "B":
        angle.fill_(0.25)
        ssm.fill_(0.05)
        key.fill_(0.10)
        value.fill_(0.20)
    initial_state = capture_state(angle, ssm, key, value, config, nheads, nangles, 0)
    continuation_cache = None
    if case_id == "D":
        continuation_cache = tuple(tensor.clone() for tensor in (angle, ssm, key, value))
    expected_output, expected_state = [], []
    with torch.no_grad():
        for step, row in enumerate(input_rows, start=1):
            output, angle, ssm, key, value = model.step(
                torch.tensor([row["values"]], device=device), angle, ssm, key, value
            )
            if not all(torch.isfinite(tensor).all().item() for tensor in [output, angle, ssm, key, value]):
                fail(f"official step {step} produced a non-finite value")
            expected_output.append(tensor_1d(output[0].float().cpu().tolist()))
            expected_state.append(capture_state(angle, ssm, key, value, config, nheads, nangles, step))

    if continuation_cache is not None:
        split_outputs = []
        split_at = len(input_rows) // 2
        split_angle, split_ssm, split_key, split_value = continuation_cache
        for row in input_rows[:split_at]:
            output, split_angle, split_ssm, split_key, split_value = model.step(
                torch.tensor([row["values"]], device=device), split_angle, split_ssm, split_key, split_value
            )
            split_outputs.append(output.detach().float().cpu())
        for row in input_rows[split_at:]:
            output, split_angle, split_ssm, split_key, split_value = model.step(
                torch.tensor([row["values"]], device=device), split_angle, split_ssm, split_key, split_value
            )
            split_outputs.append(output.detach().float().cpu())
        expected_outputs = [torch.tensor(item["values"]) for item in expected_output]
        if any(not torch.equal(actual, expected) for actual, expected in zip(split_outputs, expected_outputs)):
            fail("official case D full and split streaming outputs diverged")
        split_final = capture_state(split_angle, split_ssm, split_key, split_value, config, nheads, nangles, len(input_rows))
        if split_final != expected_state[-1]:
            fail("official case D full and split streaming state diverged")

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
            "instrumentation_patch_sha256": instrumentation_patch_sha256,
            "python_version": sys.version.split()[0],
            "pytorch_version": torch.__version__,
            "dtype": "F32",
            "device": torch.cuda.get_device_name(int(device.removeprefix("cuda:"))),
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
    validate_fixture_payload(fixture)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=output_path.parent, delete=False) as temporary:
        temporary.write(json.dumps(fixture, indent=2) + "\n")
        temporary_path = Path(temporary.name)
    try:
        stored = json.loads(temporary_path.read_text())
        validate_fixture_payload(stored)
    except (OSError, json.JSONDecodeError) as error:
        temporary_path.unlink(missing_ok=True)
        fail(f"temporary fixture validation failed: {error}")
    temporary_path.replace(output_path)
    print(f"wrote {output_path}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", default=os.environ.get("SOMA_MAMBA_OFFICIAL_DIR"))
    parser.add_argument("--output", required=True)
    parser.add_argument("--case", choices=CASE_IDS, required=True)
    parser.add_argument("--device", default=os.environ.get("SOMA_MAMBA_ORACLE_DEVICE", "cuda:0"))
    parser.add_argument("--dtype", default=os.environ.get("SOMA_MAMBA_ORACLE_DTYPE", "float32"))
    parser.add_argument("--seed", type=int)
    parser.add_argument("--instrumentation-patch")
    parser.add_argument("--overwrite", "--force", action="store_true")
    args = parser.parse_args()
    if not args.reference_root:
        fail("pass --reference-root or set SOMA_MAMBA_OFFICIAL_DIR")
    if args.instrumentation_patch:
        fail("instrumentation patches are not supported; the official cache API already exposes required state")
    reference_root = Path(args.reference_root)
    output_path = Path(args.output)
    reject_unsafe_output_path(output_path)
    if output_path.exists() and not args.overwrite:
        fail("output already exists; pass --overwrite to replace an existing fixture")
    torch, mamba3 = require_environment(reference_root, args.device, args.dtype)
    write_fixture(
        output_path,
        reference_root,
        torch,
        mamba3,
        args.case,
        args.device,
        CASE_SEEDS[args.case] if args.seed is None else args.seed,
        None,
    )


if __name__ == "__main__":
    main()
