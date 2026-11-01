#!/usr/bin/env bash
# Landing derivatives from a 1920x1080 native capture: a 16:9 crop below the client's debug
# performance line (top-left, y < 100), then 1600 and 800 px WebP. Crop and resize only; no
# repainting. Usage: media_from_capture.sh <capture.png> <name>  ->  web/landing/media/captures/<name>-{1600,800}.webp
set -euo pipefail
[ $# = 2 ] || { sed -n '2,5p' "$0"; exit 2; }
out="$(cd "$(dirname "$0")/../landing/media/captures" && pwd)"
[ "$(identify -format '%wx%h' "$1")" = 1920x1080 ] || { echo "expected a 1920x1080 capture" >&2; exit 2; }
for width in 1600 800; do
	convert "$1" -crop 1742x980+89+100 +repage -resize "${width}x" -quality 82 -define webp:method=6 \
		"$out/$2-$width.webp"
done
identify -format '%f %wx%h %b\n' "$out/$2"-*.webp
