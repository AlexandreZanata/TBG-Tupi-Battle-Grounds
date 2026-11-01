#!/usr/bin/env bash
# P66 §5.2 (U07): build the one-time bootstrap archive that adopts a Linux
# install into managed updates. It carries the stable launcher and the
# trusted TUF root out of band (the root is the trust anchor, so publish
# the archive through a channel the player already trusts, e.g. the GitHub
# release page); every later release arrives in game through TUF.
#   bash scripts/managed-bootstrap.sh --launcher target/release/tbg-launcher \
#     --root <repo>/metadata/1.root.json --base-url https://<domain>/updates [--channel stable]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
launcher="" root="" base="" channel="stable" out="$ROOT/target/native"
while [ $# -gt 0 ]; do
	case "$1" in
	--launcher) launcher="$2" ;;
	--root) root="$2" ;;
	--base-url) base="${2%/}" ;;
	--channel) channel="$2" ;;
	--output) out="$2" ;;
	*)
		echo "unknown argument $1" >&2
		exit 2
		;;
	esac
	shift 2
done
if [ ! -x "$launcher" ] || [ ! -f "$root" ]; then
	echo "need --launcher and --root" >&2
	exit 2
fi
[[ "$base" =~ ^https://[A-Za-z0-9.-]+(:[0-9]+)?(/[A-Za-z0-9._/-]*)?$ ]] || {
	echo "--base-url must be an https URL" >&2
	exit 2
}
[[ "$channel" =~ ^[a-z0-9-]{1,32}$ ]] || { echo "bad channel" >&2; exit 2; }

name="tupi-battle-grounds-bootstrap-linux-x86_64"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
dir="$stage/$name"
mkdir -p "$dir"
install -m 755 "$launcher" "$dir/tbg-launcher"
install -m 644 "$root" "$dir/root.json"
cat >"$dir/install.sh" <<EOF
#!/usr/bin/env sh
# Adopt or install Tupi Battle Grounds with in-game updates. Per user, no
# sudo; existing installs and saved data are kept.
set -eu
here="\$(cd "\$(dirname "\$0")" && pwd)"
exec "\$here/tbg-launcher" install --root-json "\$here/root.json" \\
  --metadata-url "$base/metadata/" --targets-url "$base/targets/" --channel "$channel"
EOF
chmod 755 "$dir/install.sh"
mkdir -p "$out"
tar -C "$stage" --owner=0 --group=0 --numeric-owner -czf "$out/$name.tar.gz" "$name"
(cd "$out" && sha256sum "$name.tar.gz" >"$name.tar.gz.sha256")
echo "$out/$name.tar.gz"
