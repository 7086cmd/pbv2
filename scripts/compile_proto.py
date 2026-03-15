#!/usr/bin/env python3
"""
Compile proto/pbv2.proto → importer/pbv2_pb2.py

Requires grpcio-tools:
    uv run --with grpcio-tools python scripts/compile_proto.py

Or, if installed in the importer venv:
    python scripts/compile_proto.py
"""

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).parent.parent
PROTO_DIR = ROOT / "proto"
PROTO_FILE = PROTO_DIR / "pbv2.proto"
OUT_DIR = ROOT / "importer"


def main() -> None:
    cmd = [
        sys.executable,
        "-m",
        "grpc_tools.protoc",
        f"--proto_path={PROTO_DIR}",
        f"--python_out={OUT_DIR}",
        str(PROTO_FILE),
    ]
    print(f"Running: {' '.join(str(c) for c in cmd)}")
    result = subprocess.run(cmd, check=False)
    if result.returncode != 0:
        sys.exit(result.returncode)
    out = OUT_DIR / "pbv2_pb2.py"
    print(f"Generated: {out}")


if __name__ == "__main__":
    main()
