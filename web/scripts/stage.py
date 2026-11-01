#!/usr/bin/env python3
"""Stage one publishable site into a fresh directory (the release that gets uploaded)."""

import argparse
import os
import sys
from pathlib import Path

from sites import SITES, ManifestError, files
from asset_versions import rendered


def stage(site, out):
    if site.local_only:
        raise SystemExit(f"{site.name} is a local preview only; it is never staged for publication")
    if out.exists() and any(out.iterdir()):
        raise SystemExit(f"{out} is not empty; stage into a fresh directory")
    try:
        mapping = files(site)
    except ManifestError as error:
        raise SystemExit(f"manifest error: {error}")
    for published, body in rendered(mapping).items():
        target = out / published
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(body)
        os.chmod(target, 0o644)
    for folder in [out, *[p for p in out.rglob("*") if p.is_dir()]]:
        os.chmod(folder, 0o755)
    total = sum(p.stat().st_size for p in out.rglob("*") if p.is_file())
    print(f"staged {site.name}: {len(mapping)} files, {total / 1024:.0f} KiB -> {out}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("site", choices=sorted(SITES))
    parser.add_argument("out", type=Path)
    args = parser.parse_args()
    stage(SITES[args.site], args.out.resolve())
    return 0


if __name__ == "__main__":
    sys.exit(main())
