#!/usr/bin/env bash
# Vendor MB-Lab 1.8.1 into third_party/MB-Lab (trim dev-only paths).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/third_party/MB-Lab"
TAG="1_8_1"
REPO_URL="https://github.com/animate1978/MB-Lab.git"
TMP="${TMPDIR:-/tmp}/mblab-vendor-${TAG}"

echo "Vendoring MB-Lab ${TAG} → third_party/MB-Lab"

rm -rf "$TMP"
git clone --depth 1 --branch "$TAG" "$REPO_URL" "$TMP"

rm -rf "$DEST"
mkdir -p "$(dirname "$DEST")"
rsync -a --exclude '.git' "$TMP/" "$DEST/"

# Dev / CI only — not needed at Blender runtime
rm -rf "$DEST/.github"
rm -f "$DEST/CHANGELOG.md" "$DEST/.pylintrc" "$DEST/.gitignore"

# TBG trim — optional packs only (keep expressions_* — required at MB-Lab init)
rm -rf \
  "$DEST/data/animations" \
  "$DEST/data/Hair_Data" \
  "$DEST/data/Particle_Hair"

# Anime templates (TBG targets realistic humanoid bases per mblab_bases.md)
rm -rf \
  "$DEST/data/presets/anime_female_base" \
  "$DEST/data/presets/anime_male_base" \
  "$DEST/data/presets/human_female_anime" \
  "$DEST/data/presets/human_male_anime"

rm -rf "$TMP"
du -sh "$DEST"

echo "Done. Run ./scripts/install-mblab.sh to copy into Blender add-ons."
