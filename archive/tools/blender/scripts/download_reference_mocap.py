#!/usr/bin/env python3
"""Download official CMU mocap BVH clips into .local/reference/mocap/."""

from __future__ import annotations

import argparse
import sys
import urllib.error
import urllib.request
from pathlib import Path

_REPO = Path(__file__).resolve().parents[3]
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))

from tools.blender.package_path import REPO_ROOT, resolve_repo_path  # noqa: E402

CMU_RAW = "https://raw.githubusercontent.com/una-dinosauria/cmu-mocap/master/data/{dir}/{name}"
DEFAULT_CLIPS = ("35_01.bvh", "35_04.bvh", "39_01.bvh")


def _cmu_url(name: str) -> str:
    return CMU_RAW.format(dir=f"0{name[:2]}", name=name)


def download_clip(name: str, out_dir: Path) -> Path:
    dest = out_dir / name
    url = _cmu_url(name)
    try:
        with urllib.request.urlopen(url, timeout=120) as resp:
            data = resp.read()
    except urllib.error.HTTPError as exc:
        raise SystemExit(f"HTTP {exc.code} for {url}") from exc
    except urllib.error.URLError as exc:
        raise SystemExit(f"Network error for {url}: {exc.reason}") from exc
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(data)
    return dest


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Download CMU reference mocap BVH files")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path(".local/reference/mocap"),
        help="Output directory (repo-relative)",
    )
    parser.add_argument(
        "--clips",
        nargs="*",
        default=list(DEFAULT_CLIPS),
        help=f"BVH filenames (default: {' '.join(DEFAULT_CLIPS)})",
    )
    args = parser.parse_args(argv)

    out_dir = resolve_repo_path(args.out_dir)
    for name in args.clips:
        path = download_clip(name, out_dir)
        print(f"Wrote {path} ({path.stat().st_size // 1024} KiB)")

    print(
        "\nSource: Carnegie Mellon Graphics Lab Motion Capture Database\n"
        "  http://mocap.cs.cmu.edu/\n"
        "  BVH port: https://github.com/una-dinosauria/cmu-mocap\n"
        "Walk pipeline: tools/blender/scripts/OPEN-SOURCE-WALK.md"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
