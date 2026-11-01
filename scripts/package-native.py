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
    parser.add_argument("--runner", type=Path,
                        help="Unofficial cross builds only: command that runs the binary for its identity "
                             "checks (e.g. a Wine wrapper). Official Windows packages are checked on Windows (P68).")
    parser.add_argument("--managed", action="store_true",
                        help="P66: build a managed-update release target (flat archive + descriptor)")
    parser.add_argument("--build-id", help="P66 managed: release build id ([a-z0-9._-])")
    parser.add_argument("--sequence", type=int, help="P66 managed: monotonic release sequence")
    parser.add_argument("--notes", type=Path, help="P66 managed: release-notes JSON (EN + pt-BR)")
    parser.add_argument("--channel", default="stable", help="P66 managed: release channel")
    parser.add_argument("--security-floor", type=int, default=0,
                        help="P66 managed: lowest release sequence still allowed to run")
    args = parser.parse_args()
    if args.managed:
        return managed(args)
    version = tomllib.loads((ROOT / "crates/tbg-client/Cargo.toml").read_text())["package"]["version"]
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    run = [str(args.runner.resolve())] if args.runner else []
    if args.runner and args.platform != "windows":
        raise SystemExit("--runner is only for unofficial cross-built Windows packages")
    identity = subprocess.check_output([*run, str(args.binary.resolve()), "--version"], text=True).strip()
    if f" {version} ·" not in identity or f"source {commit}" not in identity:
        raise SystemExit("Build with TBG_SOURCE_COMMIT=$(git rev-parse HEAD); executable and source must match")
    metadata=json.loads(subprocess.check_output([*run, str(args.binary.resolve()),"--build-info"],text=True))
    if metadata.get("source_commit")!=commit or metadata.get("version")!=version or metadata.get("profile")!="release":
        raise SystemExit("Native delivery requires matching release build metadata")
    name = f"tupi-battle-grounds-{version}-{args.platform}-x86_64"
    if args.no_archive:
        name += f"-dev-{commit[:12]}"
    package = args.output / name
    if package.exists():
        raise SystemExit("Refusing to reuse existing package staging; choose a fresh output directory")
    package.mkdir(parents=True)
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
        "contract":metadata.get("contract"),
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


def managed(args):
    """P66: a flat release archive (files at its root, as the launcher
    extracts them) plus the descriptor `tbg-release publish` signs."""
    if args.platform != "linux":
        raise SystemExit("managed packages are Linux-only until Windows acceptance (P66 M4)")
    if not (args.build_id and args.sequence and args.notes):
        raise SystemExit("--managed needs --build-id, --sequence and --notes")
    if not re.fullmatch(r"[a-z0-9._-]{1,80}", args.build_id) or args.build_id in (".", ".."):
        raise SystemExit("invalid --build-id")
    version = tomllib.loads((ROOT / "crates/tbg-client/Cargo.toml").read_text())["package"]["version"]
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    binary = args.binary.resolve()
    identity = subprocess.check_output([str(binary), "--version"], text=True).strip()
    if f"source {commit}" not in identity:
        raise SystemExit("Build with TBG_SOURCE_COMMIT=$(git rev-parse HEAD); executable and source must match")
    contract = subprocess.check_output([str(binary), "--contract"], text=True).strip()
    notes = json.loads(args.notes.read_text())
    if notes.get("build_id") != args.build_id:
        raise SystemExit("the notes name another build id")
    stage = args.output / f"managed-{args.build_id}"
    outputs = [stage, args.output / f"package-{args.build_id}.tar.gz",
               args.output / f"descriptor-{args.build_id}.json", args.output / f"notes-{args.build_id}.json"]
    if any(path.exists() for path in outputs):
        raise SystemExit("Refusing to overwrite a managed release; choose a fresh build id/output")
    stage.mkdir(parents=True)
    shutil.copy2(binary, stage / "tbg-client")
    (stage / "tbg-client").chmod(0o755)
    shutil.copytree(ROOT / "crates/tbg-client/assets", stage / "assets",
                    ignore=shutil.ignore_patterns("*.js", "*.css", "*.html"))
    shutil.copy2(ROOT / "THIRD_PARTY_NOTICES.md", stage)
    # The same binary links the D-97 vault/HTTP crates; ship their notices too.
    shutil.copytree(ROOT / "deploy/native/licenses", stage / "licenses")
    if (ROOT / "LICENSE").exists():
        shutil.copy2(ROOT / "LICENSE", stage)
    (stage / "release-notes.json").write_text(json.dumps(notes, ensure_ascii=False, indent=2) + "\n")
    (stage / "build-info.json").write_text(json.dumps({
        "build_id": args.build_id, "version": version, "commit": commit, "platform": "linux",
        "architecture": "x86_64", "contract": contract,
        "binary_sha256": hashlib.sha256((stage / "tbg-client").read_bytes()).hexdigest(),
        "cargo_lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
    }, indent=2) + "\n")
    files = [{"path":path.relative_to(stage).as_posix(),"size":path.stat().st_size,
              "sha256":hashlib.sha256(path.read_bytes()).hexdigest()}
             for path in sorted(stage.rglob("*")) if path.is_file()]
    (stage / "file-manifest.json").write_text(json.dumps({"schema":1,"source_commit":commit,"files":files},indent=2)+"\n")
    archive = args.output / f"package-{args.build_id}.tar.gz"
    with tarfile.open(archive, "w:gz") as dest:
        for path in sorted(stage.rglob("*")):
            dest.add(path, arcname=str(path.relative_to(stage)), recursive=False)
    descriptor = {
        "schema": 1, "build_id": args.build_id, "release_sequence": args.sequence,
        "display_version": version, "channel": args.channel, "platform": "linux",
        "architecture": "x86_64", "install_mode": "managed",
        "package_target": f"package-{args.build_id}.tar.gz",
        "notes_target": f"notes-{args.build_id}.json",
        "compatibility": {"match_contract": contract, "presentation": f"p-{commit[:12]}"},
        "updater_abi": 1, "minimum_bootstrap": 1,
        "security_floor_sequence": args.security_floor,
        "local_state_reader": [1], "local_state_writer": 1, "minimum_glibc": "2.39",
    }
    (args.output / f"descriptor-{args.build_id}.json").write_text(json.dumps(descriptor, indent=2) + "\n")
    (args.output / f"notes-{args.build_id}.json").write_text(json.dumps(notes, ensure_ascii=False, indent=2) + "\n")
    shutil.rmtree(stage)
    print(archive.resolve())


if __name__ == "__main__":
    main()
