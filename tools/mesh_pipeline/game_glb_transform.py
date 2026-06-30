"""Shared gltf-transform CLI invocation."""

from __future__ import annotations

import shutil
import subprocess


def gltf_transform_cmd() -> list[str]:
    local = shutil.which("gltf-transform")
    if local:
        return [local]
    npx = shutil.which("npx")
    if npx is None:
        raise RuntimeError("npx is required for gltf-transform")
    return [npx, "--yes", "@gltf-transform/cli"]


def run_gltf_transform(args: list[str]) -> None:
    subprocess.run([*gltf_transform_cmd(), *args], check=True)
