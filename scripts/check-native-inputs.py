#!/usr/bin/env python3
"""Reject checkout-dependent text bytes before native compilation and packaging."""
import argparse
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SCOPES = ["Cargo.lock", "Cargo.toml", "crates/tbg-core", "crates/tbg-client"]
TEXT = {".rs", ".toml", ".lock", ".json", ".txt", ".md", ".wgsl", ".js", ".css", ".html", ".svg"}


def problems(root):
    paths = subprocess.check_output(
        ["git", "ls-files", "-z", "--", *SCOPES], cwd=root,
    ).decode("utf-8").split("\0")
    errors = []
    for name in filter(None, paths):
        if Path(name).suffix not in TEXT:
            continue
        data = (root / name).read_bytes()
        if b"\r\n" in data:
            errors.append(f"{name}: CRLF changes native input hashes; checkout must use LF")
        try:
            data.decode("utf-8", errors="strict")
        except UnicodeDecodeError:
            errors.append(f"{name}: native text input must be UTF-8")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    errors = problems(args.root)
    if errors:
        raise SystemExit("\n".join(errors))
    print("Native inputs use canonical UTF-8/LF bytes; binary assets are untouched.")


if __name__ == "__main__":
    main()
