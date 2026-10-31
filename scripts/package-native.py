#!/usr/bin/env python3
"""Package a native client with its assets, provenance and SHA-256 digest."""

import argparse
import hashlib
import json
import re
from pathlib import Path
import shutil
import subprocess
import tarfile
import tomllib
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--platform", choices=["linux", "windows"], required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=ROOT / "target/native")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "crates/tbg-client/Cargo.toml").read_text())["package"]["version"]
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    identity = subprocess.check_output([str(args.binary.resolve()), "--version"], text=True).strip()
    if f" {version} ·" not in identity or f"source {commit}" not in identity:
        raise SystemExit("Build with TBG_SOURCE_COMMIT=$(git rev-parse HEAD); executable and source must match")
    name = f"tupi-battle-grounds-{version}-{args.platform}-x86_64"
    package = args.output / name
    package.mkdir(parents=True, exist_ok=True)
    binary_name = "tbg-client.exe" if args.platform == "windows" else "tbg-client"
    shutil.copy2(args.binary, package / binary_name)
    (package / binary_name).chmod(0o755)
    shutil.copytree(ROOT / "crates/tbg-client/assets", package / "assets", dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns("*.js", "*.css", "*.html"))
    shutil.copy2(ROOT / "THIRD_PARTY_NOTICES.md", package)
    license_file = ROOT / "LICENSE"
    if license_file.exists():
        shutil.copy2(license_file, package)
    else:
        (package / "LICENSE-STATUS.txt").write_text(
            "The project owner has not selected a project license.\n"
            "See THIRD_PARTY_NOTICES.md and assets/fonts/OFL.txt for third-party notices.\n")
    shutil.copy2(ROOT / "deploy/native/README.txt", package / "README.txt")
    if args.platform == "linux":
        shutil.copy2(ROOT / "scripts/install-linux.sh", package / "install.sh")
        (package / "install.sh").chmod(0o755)
    (package / "build-info.json").write_text(json.dumps({
        "version": version, "commit": commit, "platform": args.platform,
        "architecture": "x86_64", "protocol": int(re.search(r"PROTOCOL_VERSION: u16 = (\d+)", (ROOT / "crates/tbg-core/src/protocol.rs").read_text()).group(1)),
        "binary_sha256": hashlib.sha256((package / binary_name).read_bytes()).hexdigest(),
    }, indent=2) + "\n")
    if args.platform == "linux":
        archive = args.output / f"{name}.tar.gz"
        with tarfile.open(archive, "w:gz") as dest:
            dest.add(package, arcname=name)
    else:
        archive = args.output / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as dest:
            for path in sorted(package.rglob("*")):
                if path.is_file():
                    dest.write(path, Path(name) / path.relative_to(package))
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_name(archive.name + ".sha256").write_text(f"{digest}  {archive.name}\n")
    print(archive.resolve())


if __name__ == "__main__":
    main()
