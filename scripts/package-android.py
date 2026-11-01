#!/usr/bin/env python3
"""Stage verified native Android inputs, or inspect a final signed prototype APK."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

from android_binary import SYSTEM_LIBRARIES, elf_identity, inspect_apk_libraries

ROOT = Path(__file__).resolve().parents[1]
STAGE = ROOT / "platform/android/app/build/native-input"
ASSETS = ROOT / "crates/tbg-client/assets"
PACKAGE = "io.github.alexandrezanata.tbg.prototype"
BASE = "6e569183b1f589faafab16bdc3f1c2b7a3d83316"


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def source_inventory():
    """Bind native build metadata to tracked client/core, tools, and host sources."""
    paths = git("ls-files", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
                "crates/tbg-client", "crates/tbg-core", "platform/android",
                "scripts/package-android.py", "scripts/android_binary.py").splitlines()
    return {name: digest((ROOT / name).read_bytes()) for name in sorted(paths)}


def asset_inventory():
    paths = git("ls-files", "crates/tbg-client/assets").splitlines()
    result = {}
    for name in paths:
        path = ROOT / name
        if path.suffix.lower() in {".js", ".css", ".html"}:
            continue
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Invalid packaged asset: {name}")
        result[path.relative_to(ASSETS).as_posix()] = digest(path.read_bytes())
    if not result:
        raise ValueError("No tracked assets; refusing an empty APK")
    return result


def rules_identity():
    rules = (ROOT / "crates/tbg-core/src/ruleset_version.rs").read_text()
    value = int(re.search(r"RULESET_VERSION: u32 = (\d+)", rules)[1])
    state = 0xcbf29ce484222325
    watched = re.findall(r'\("([^"]+)", include_str!\("([^"]+)"\)\)', rules)
    if not watched:
        raise ValueError("No canonical ruleset source inventory")
    for name, path in watched:
        raw = (ROOT / "crates/tbg-core/src" / path).read_bytes()
        for byte in name.encode() + b"\0" + raw + b"\xff":
            state = ((state ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return {"version": value, "source_hash": f"{state:016x}"}


def prepare(args):
    source = args.source
    if not re.fullmatch(r"[0-9a-f]{40}", source) or source != git("rev-parse", "HEAD"):
        raise ValueError("--source must identify the checked-out execution commit")
    if git("status", "--porcelain", "--untracked-files=normal"):
        raise ValueError("Commit the reviewed source before staging native inputs")
    subprocess.run(["git", "merge-base", "--is-ancestor", BASE, source], cwd=ROOT, check=True)
    libraries = {"libtbg_client.so": args.library.resolve()}
    libraries.update({path.name: path.resolve() for path in args.dependency})
    if len(libraries) != 1 + len(args.dependency):
        raise ValueError("Duplicate native library inputs")
    binary = args.library.read_bytes()
    # The shared Rust Android entry must retain this exact source marker in the cdylib.
    if f"TBG_ANDROID_SOURCE={source}".encode() not in binary:
        raise ValueError("Library source marker is absent/mismatched; refusing stale .so")
    identities = {}
    for name, path in libraries.items():
        if not re.fullmatch(r"lib[A-Za-z0-9_+.\-]+\.so", name):
            raise ValueError("Invalid native library filename")
        identities[name] = elf_identity(path.read_bytes(),args.abi)
        missing = set(identities[name]["imports"]) - SYSTEM_LIBRARIES - libraries.keys()
        if missing:
            raise ValueError(f"{name}: provide required native dependencies: {sorted(missing)}")
    assets = asset_inventory()
    version = tomllib.loads((ROOT / "crates/tbg-client/Cargo.toml").read_text())["package"]["version"]
    protocol = int(re.search(r"PROTOCOL_VERSION: u16 = (\d+)",
                            (ROOT / "crates/tbg-core/src/protocol.rs").read_text())[1])
    info = {
        "source_sha": source, "reviewed_linux_base": BASE, "linux_version": version,
        "android_revision": 7, "package": PACKAGE, "abi": args.abi,
        "protocol": protocol, "ruleset": rules_identity(),
        "content_sha256": digest((ROOT / "crates/tbg-core/src/content.rs").read_bytes()),
        "cargo_lock_sha256": digest((ROOT / "Cargo.lock").read_bytes()),
        "assets_sha256": digest(json.dumps(assets, sort_keys=True).encode()),
        "native_libraries": {name: digest(path.read_bytes()) for name, path in libraries.items()},
        "toolchain": json.loads((ROOT / "platform/android/toolchain.json").read_text()),
    }
    STAGE.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="native-input-", dir=STAGE.parent) as scratch:
        target = Path(scratch)
        for name, path in libraries.items():
            dest = target / "jniLibs" / args.abi / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(path, dest)
        for name in assets:
            dest = target / "assets" / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ASSETS / name, dest)
        write_json(target / "assets/build-info.json", info)
        write_json(target / "assets/asset-inventory.json", assets)
        write_json(target / "source-inventory.json", source_inventory())
        # Only this generated task-owned directory is replaced; no shared build caches.
        if STAGE.exists():
            shutil.rmtree(STAGE)
        shutil.move(str(target), STAGE)
    verify_stage()
    print(f"Prepared {len(assets)} assets and {len(libraries)} verified ARM64 libraries")


def verify_stage():
    info_path = STAGE / "assets/build-info.json"
    if not info_path.is_file():
        raise ValueError("Native input missing; build Rust explicitly, then run prepare")
    info = json.loads(info_path.read_text())
    recorded = json.loads((STAGE / "source-inventory.json").read_text())
    if recorded != source_inventory():
        raise ValueError("Tracked runtime/package sources changed after staging")
    # Documentation-only descendants are allowed, but the frozen execution SHA remains explicit.
    subprocess.run(["git", "merge-base", "--is-ancestor", info["source_sha"], "HEAD"],
                   cwd=ROOT, check=True)
    if not recorded or info["cargo_lock_sha256"] != digest((ROOT / "Cargo.lock").read_bytes()):
        raise ValueError("Invalid source inventory/Cargo.lock identity")
    if info["ruleset"] != rules_identity():
        raise ValueError("Ruleset identity changed after staging")
    if info["package"] != PACKAGE or info["abi"] not in {"arm64-v8a","x86_64"}:
        raise ValueError("Wrong native prototype package/ABI")
    assets = json.loads((STAGE / "assets/asset-inventory.json").read_text())
    if assets != asset_inventory() or digest(json.dumps(assets, sort_keys=True).encode()) != info["assets_sha256"]:
        raise ValueError("Asset inventory changed after staging")
    for name, expected in assets.items():
        if digest((STAGE / "assets" / name).read_bytes()) != expected:
            raise ValueError(f"Staged asset changed: {name}")
    actual_assets = {p.relative_to(STAGE / "assets").as_posix()
                     for p in (STAGE / "assets").rglob("*") if p.is_file()}
    if actual_assets != assets.keys() | {"build-info.json", "asset-inventory.json"}:
        raise ValueError("Unexpected staged assets")
    libs = STAGE / "jniLibs" / info["abi"]
    if {p.name for p in libs.iterdir()} != info["native_libraries"].keys():
        raise ValueError("Unexpected staged native libraries")
    for name, expected in info["native_libraries"].items():
        if digest((libs / name).read_bytes()) != expected:
            raise ValueError(f"Staged library changed: {name}")
    return info, assets


def inspect_apk(args):
    info, assets = verify_stage()
    libraries = inspect_apk_libraries(args.apk, info["abi"])
    with zipfile.ZipFile(args.apk) as archive:
        for name in ("build-info.json", "asset-inventory.json"):
            if archive.read("assets/" + name) != (STAGE / "assets" / name).read_bytes():
                raise ValueError(f"APK metadata differs from stage: {name}")
        for name, expected in assets.items():
            if digest(archive.read("assets/" + name)) != expected:
                raise ValueError(f"APK asset differs: {name}")
        if libraries.keys() != info["native_libraries"].keys():
            raise ValueError("APK native library inventory differs")
        for name, expected in info["native_libraries"].items():
            if digest(archive.read("lib/"+info["abi"]+"/" + name)) != expected:
                raise ValueError(f"APK library differs: {name}")
    sdk = Path(os.environ.get("ANDROID_HOME", os.environ.get("ANDROID_SDK_ROOT", "")))
    tools = sdk / "build-tools/36.0.0"
    signature = subprocess.check_output([str(tools / "apksigner"), "verify", "--verbose", "--print-certs",
                                         str(args.apk)], text=True)
    certificates = re.findall(r"Signer #\d+ certificate SHA-256 digest: ([0-9a-fA-F]+)", signature)
    if len(certificates) != 1:
        raise ValueError("Expected exactly one prototype signing certificate")
    badging = subprocess.check_output([str(tools / "aapt2"), "dump", "badging", str(args.apk)], text=True)
    if f"package: name='{PACKAGE}'" not in badging:
        raise ValueError("Wrong APK application ID")
    permissions = re.findall(r"uses-permission[^:]*: name='([^']+)'", badging)
    # AndroidX protects its own internal receiver with this signature permission.
    expected_permissions = {"android.permission.INTERNET", PACKAGE + ".DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION"}
    if "android.permission.INTERNET" not in permissions or set(permissions) - expected_permissions:
        raise ValueError(f"Unexpected APK permissions: {permissions}")
    subprocess.run([str(tools / "zipalign"), "-c", "-P", "16", "-v", "4", str(args.apk)],
                   check=True, stdout=subprocess.DEVNULL)
    report = {
        "apk_sha256": digest(args.apk.read_bytes()), "signing_certificate_sha256": certificates[0],
        "build_info": info, "libraries": libraries, "permissions": permissions,
        "package_badging": badging.splitlines()[0],
        "limits": "ELF/ZIP inspection does not establish 16 KB runtime, gameplay or store readiness",
    }
    write_json(args.report, report)
    print(f"Verified APK: {args.apk.name}; SHA-256 {report['apk_sha256']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    stage = commands.add_parser("prepare")
    stage.add_argument("--library", type=Path, required=True)
    stage.add_argument("--dependency", type=Path, action="append", default=[])
    stage.add_argument("--source", required=True)
    stage.add_argument("--abi", choices=["arm64-v8a","x86_64"], default="arm64-v8a")
    verify = commands.add_parser("verify-stage")
    verify.add_argument("--abi", choices=["arm64-v8a","x86_64"])
    apk = commands.add_parser("inspect-apk")
    apk.add_argument("--apk", type=Path, required=True)
    apk.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            prepare(args)
        elif args.command == "verify-stage":
            info, _ = verify_stage()
            if args.abi and info["abi"] != args.abi:
                raise ValueError("Staged ABI does not match the selected APK variant")
            print("Verified current native staging input")
        else:
            inspect_apk(args)
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError, zipfile.BadZipFile) as error:
        parser.exit(1, f"Android packaging refused: {error}\n")


if __name__ == "__main__":
    main()
