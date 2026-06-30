"""Mobile game mesh budget constants — see tools/mesh-pipeline/BUDGETS.md."""

from __future__ import annotations

MAX_TRIANGLES = 8_000
MAX_BONES = 64
MAX_MATERIALS = 2
MAX_TEXTURE_SIZE = 1024
TARGET_FILE_BYTES = 2 * 1024 * 1024
HARD_MAX_FILE_BYTES = 5 * 1024 * 1024
MIN_CLIPS_SKINNED = 6
MAX_CLIPS_SKINNED = 12
