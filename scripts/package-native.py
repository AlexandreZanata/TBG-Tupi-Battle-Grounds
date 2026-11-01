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
    parser.add_argument("--no-archive", action="store_true",
                        help="Prepare a commit-specific local preview directory without compression")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "crates/tbg-client/Cargo.toml").read_text())["package"]["version"]
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    identity = subprocess.check_output([str(args.binary.resolve()), "--version"], text=True).strip()
    if f" {version} ·" not in identity or f"source {commit}" not in identity:
        raise SystemExit("Build with TBG_SOURCE_COMMIT=$(git rev-parse HEAD); executable and source must match")
    metadata=json.loads(subprocess.check_output([str(args.binary.resolve()),"--build-info"],text=True))
    if metadata.get("source_commit")!=commit or metadata.get("version")!=version or metadata.get("profile")!="release":
        raise SystemExit("Native delivery requires matching release build metadata")
    name = f"tupi-battle-grounds-{version}-{args.platform}-x86_64"
    if args.no_archive:
        name += f"-dev-{commit[:12]}"
    package = args.output / name
    args.output.mkdir(parents=True,exist_ok=True)
    if package.exists():
        raise SystemExit("Refusing to reuse existing staging; choose a fresh output directory")
    package.mkdir()
    binary_name = "tbg-client.exe" if args.platform == "windows" else "tbg-client"
    shutil.copy2(args.binary, package / binary_name)
    (package / binary_name).chmod(0o755)
    shutil.copytree(ROOT / "crates/tbg-client/assets", package / "assets", dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns("*.js", "*.css", "*.html"))
    shutil.copy2(ROOT / "THIRD_PARTY_NOTICES.md", package)
    shutil.copytree(ROOT / "deploy/native/licenses",package / "licenses")
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
    files=[]
    for path in sorted((package / "assets").rglob("*")):
        if path.is_file():files.append({"path":path.relative_to(package / "assets").as_posix(),"size":path.stat().st_size,"sha256":hashlib.sha256(path.read_bytes()).hexdigest()})
    assets_hash=hashlib.sha256(json.dumps(files,sort_keys=True,separators=(",",":")).encode()).hexdigest()
    (package / "build-info.json").write_text(json.dumps({
        "version": version, "commit": commit, "platform": args.platform,
        "architecture": "x86_64", "assets_sha256":assets_hash,
        "features":metadata["features"],"target":metadata["target"],"profile":metadata["profile"],
        "cargo_lock_sha256":hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
        "protocol": int(re.search(r"PROTOCOL_VERSION: u16 = (\d+)", (ROOT / "crates/tbg-core/src/protocol.rs").read_text()).group(1)),
        "binary_sha256": hashlib.sha256((package / binary_name).read_bytes()).hexdigest(),
    }, indent=2) + "\n")
    manifest=[{"path":p.relative_to(package).as_posix(),"size":p.stat().st_size,"sha256":hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(package.rglob("*")) if p.is_file()]
    (package / "file-manifest.json").write_text(json.dumps({"schema":1,"source_commit":commit,"files":manifest},indent=2)+"\n")
    if args.no_archive:
        print(package.resolve())
        return
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
