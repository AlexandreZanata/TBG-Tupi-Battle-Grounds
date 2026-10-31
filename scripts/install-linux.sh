#!/usr/bin/env bash
# Install this exact portable package per user; keep data and older versions.
set -euo pipefail
package_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
python3 - "$package_dir" <<'PY'
import json
from pathlib import Path
import os
import shutil
import sys

package = Path(sys.argv[1])
info = json.loads((package / "build-info.json").read_text())
version = info["version"]
if not version or any(c not in "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-" for c in version):
    raise SystemExit("Invalid package version")
import hashlib
if hashlib.sha256((package / "tbg-client").read_bytes()).hexdigest() != info["binary_sha256"]:
    raise SystemExit("Executable checksum mismatch; download the package again")
base = Path.home() / ".local"
install = base / "opt/tupi-battle-grounds" / version
if install.exists():
    existing = install / "build-info.json"
    if not existing.exists() or json.loads(existing.read_text()) != info:
        raise SystemExit(f"Refusing to replace a different build at {install}")
else:
    shutil.copytree(package, install)
(install / "tbg-client").chmod(0o755)
launcher = base / "bin/tupi-battle-grounds"
launcher.parent.mkdir(parents=True, exist_ok=True)
if launcher.exists() and not launcher.is_symlink():
    raise SystemExit(f"Refusing to replace an existing file at {launcher}")
temporary = launcher.with_name("tupi-battle-grounds.new")
if temporary.is_symlink():
    temporary.unlink()
temporary.symlink_to(install / "tbg-client")
os.replace(temporary, launcher)
applications = Path(os.environ.get("XDG_DATA_HOME", str(base / "share"))) / "applications"
applications.mkdir(parents=True, exist_ok=True)
escaped = str(install / "tbg-client").replace("\\", "\\\\").replace('"', '\\"').replace('`', '\\`').replace('$', '\\$')
(applications / "tupi-battle-grounds.desktop").write_text(
    "[Desktop Entry]\nType=Application\nName=Tupi Battle Grounds\n"
    "Comment=Real-time 1v1 strategy with browser and native cross-play\n"
    f'Exec="{escaped}"\nIcon={install}/assets/lobby/crest-v2.svg\n'
    "Terminal=false\nCategories=Game;StrategyGame;\nStartupWMClass=tupi-battle-grounds\n")
print(f"Installed {version} at {install}")
print(f"Launcher: {launcher}")
print("Open Tupi Battle Grounds from your application menu.")
PY
