#!/usr/bin/env python3
"""Machine-readable pre-flight for the pinned official Mamba-3 oracle."""

import argparse
import hashlib
import importlib
import json
import os
import platform
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional, Tuple, Union

from generate_oracle import COMMIT, OFFICIAL_REPOSITORY_SUFFIX, ORACLE_CONFIG, SOURCE_HASHES

READY_STATUSES = {"ReadyF32", "ReadyBf16Only"}


def result(status: str, detail: str, **fields: object) -> dict:
    payload = {
        "status": status,
        "detail": detail,
        "python_version": sys.version.split()[0],
        "os": platform.system(),
        "architecture": platform.machine(),
        **fields,
    }
    return payload


def git_value(reference_root: Path, *arguments: str) -> str:
    return subprocess.check_output(
        ["git", "-C", str(reference_root), *arguments], text=True, stderr=subprocess.DEVNULL
    ).strip()


def verify_checkout(reference_root: Path) -> Union[Tuple[str, str, list], dict]:
    if not reference_root.is_dir():
        return result("MissingOfficialCheckout", "official checkout directory is unavailable")
    try:
        origin = git_value(reference_root, "remote", "get-url", "origin")
        commit = git_value(reference_root, "rev-parse", "HEAD")
        dirty = git_value(reference_root, "status", "--porcelain")
    except (OSError, subprocess.CalledProcessError):
        return result("MissingOfficialCheckout", "official checkout is not a usable Git checkout")
    if not origin.endswith(OFFICIAL_REPOSITORY_SUFFIX):
        return result("CommitMismatch", "official checkout origin is not state-spaces/mamba")
    if commit != COMMIT:
        return result("CommitMismatch", "official checkout is not at the pinned commit")
    if dirty:
        return result("DirtyOfficialCheckout", "official checkout has unrecorded modifications")
    hashes = []
    for source, expected_hash in SOURCE_HASHES.items():
        source_path = reference_root / source
        if not source_path.is_file():
            return result("OfficialRouteUnavailable", "required official source file is unavailable")
        actual_hash = hashlib.sha256(source_path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            return result("OfficialRouteUnavailable", f"official source hash mismatch for {source}")
        hashes.append({"path": source, "sha256": actual_hash})
    return origin, commit, hashes


def validate_output_directory(output_dir: Path) -> Optional[str]:
    repository_root = Path(__file__).resolve().parents[2]
    resolved = output_dir.resolve()
    for restricted in (repository_root / "src", repository_root / "docs", repository_root / ".git"):
        try:
            resolved.relative_to(restricted.resolve())
        except ValueError:
            continue
        return "output directory is reserved for project inputs"
    try:
        resolved.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(dir=resolved, delete=True):
            pass
    except OSError:
        return "output directory is not writable"
    return None


def preflight(reference_root: Optional[Path], output_dir: Path, device: str, dtype: str) -> dict:
    output_error = validate_output_directory(output_dir)
    if output_error:
        return result("OutputDirectoryRejected", output_error, requested_device=device, requested_dtype=dtype)
    if reference_root is None:
        return result("MissingOfficialCheckout", "official checkout directory is unavailable", requested_device=device, requested_dtype=dtype)
    checkout = verify_checkout(reference_root)
    if isinstance(checkout, dict):
        checkout["requested_device"] = device
        checkout["requested_dtype"] = dtype
        return checkout
    origin, commit, hashes = checkout
    if dtype not in {"float32", "bfloat16"}:
        return result("OfficialRouteUnavailable", "dtype must be float32 or bfloat16", requested_device=device, requested_dtype=dtype)
    if not device.startswith("cuda:") or not device.removeprefix("cuda:").isdigit():
        return result("CudaUnavailable", "device must use the cuda:<index> form", requested_device=device, requested_dtype=dtype)
    try:
        torch = importlib.import_module("torch")
    except ModuleNotFoundError:
        return result("PyTorchUnavailable", "PyTorch cannot be imported", requested_device=device, requested_dtype=dtype)
    if not torch.cuda.is_available():
        return result("CudaUnavailable", "PyTorch reports no CUDA device", requested_device=device, requested_dtype=dtype, pytorch_version=torch.__version__)
    device_index = int(device.removeprefix("cuda:"))
    device_count = torch.cuda.device_count()
    if device_index >= device_count:
        return result("CudaUnavailable", "selected CUDA device index is unavailable", requested_device=device, requested_dtype=dtype, pytorch_version=torch.__version__, cuda_device_count=device_count)
    capability = torch.cuda.get_device_capability(device_index)
    device_name = torch.cuda.get_device_name(device_index)
    metadata = {
        "requested_device": device,
        "requested_dtype": dtype,
        "pytorch_version": torch.__version__,
        "cuda_runtime": torch.version.cuda,
        "cuda_device_count": device_count,
        "device_class": device_name,
        "compute_capability": f"{capability[0]}.{capability[1]}",
        "official_repository": "state-spaces/mamba",
        "official_commit": commit,
        "official_source_hashes": hashes,
    }
    if capability != (9, 0) or "H100" not in device_name.upper():
        return result("UnsupportedGpuArchitecture", "the pinned official step route is documented as H100-only", **metadata)
    sys.path.insert(0, str(reference_root))
    try:
        mamba3 = importlib.import_module("mamba_ssm.modules.mamba3")
    except (ImportError, OSError) as error:
        return result("OfficialDependencyUnavailable", f"official Mamba-3 import failed: {type(error).__name__}", **metadata)
    if getattr(mamba3, "mamba3_step_fn", None) is None:
        return result("OfficialRouteUnavailable", "official CuTe Mamba-3 step function is unavailable", **metadata)
    try:
        allocation_dtype = torch.float32 if dtype == "float32" else torch.bfloat16
        probe = torch.tensor([0.125], device=device, dtype=allocation_dtype)
        if not torch.isfinite(probe).all().item():
            return result("OfficialRouteUnavailable", "deterministic CUDA allocation is non-finite", **metadata)
        model = mamba3.Mamba3(
            d_model=ORACLE_CONFIG["input_dim"],
            d_state=ORACLE_CONFIG["state_dim"],
            expand=ORACLE_CONFIG["expansion"],
            headdim=ORACLE_CONFIG["head_dim"],
            ngroups=1,
            rope_fraction=0.5,
            A_floor=ORACLE_CONFIG["a_floor"],
            is_outproj_norm=False,
            is_mimo=False,
            device=device,
            dtype=allocation_dtype,
        ).eval()
        angle, ssm, key, value = model.allocate_inference_cache(1, 1, device=device, dtype=allocation_dtype)
        with torch.no_grad():
            output, angle, ssm, key, value = model.step(
                torch.zeros((1, ORACLE_CONFIG["input_dim"]), device=device, dtype=allocation_dtype),
                angle,
                ssm,
                key,
                value,
            )
        if not all(torch.isfinite(tensor).all().item() for tensor in (output, angle, ssm, key, value)):
            return result("OfficialRouteUnavailable", "official deterministic step is non-finite", **metadata)
    except (AssertionError, RuntimeError, ValueError) as error:
        return result("OfficialRouteUnavailable", f"official deterministic step failed: {type(error).__name__}", **metadata)
    if dtype == "bfloat16":
        return result("ReadyBf16Only", "official BF16 route is available; Rust fixture import remains F32-only", selected_route="official_cuda_bfloat16", **metadata)
    return result("ReadyF32", "official CUDA F32 step route prerequisites are available", selected_route="official_cuda_float32", **metadata)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reference-root", default=os.environ.get("SOMA_MAMBA_OFFICIAL_DIR"))
    parser.add_argument("--output-dir", default=os.environ.get("SOMA_MAMBA_ORACLE_OUT", "target/mamba3_oracle"))
    parser.add_argument("--device", default=os.environ.get("SOMA_MAMBA_ORACLE_DEVICE", "cuda:0"))
    parser.add_argument("--dtype", default=os.environ.get("SOMA_MAMBA_ORACLE_DTYPE", "float32"))
    args = parser.parse_args()
    reference_root = Path(args.reference_root) if args.reference_root else None
    report = preflight(reference_root, Path(args.output_dir), args.device, args.dtype)
    print(json.dumps(report, sort_keys=True))
    raise SystemExit(0 if report["status"] in READY_STATUSES else 1)


if __name__ == "__main__":
    main()
