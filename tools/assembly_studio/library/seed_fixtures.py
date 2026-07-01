"""Seed modular library fixtures from archived blocky TBG template."""

from __future__ import annotations

import json
from pathlib import Path

from assembly_studio.library.glb_write import copy_scaled_body, extract_armature_only, write_box_part
from assembly_studio.library.sockets import BASE_IDS

REPO = Path(__file__).resolve().parents[3]
LIBRARY = REPO / "content" / "characters" / "library"
TEMPLATE = REPO / "archive/content/characters/seed-glb-pipeline/meshes/game/demo_blocky_walk_01.glb"

BASE_PROFILES: dict[str, dict] = {
    "male": {"label": "Male body", "heightM": 1.8, "bodyClass": "HumanoidLight", "scale": (1.0, 1.0, 1.0)},
    "female": {"label": "Female body", "heightM": 1.65, "bodyClass": "HumanoidLight", "scale": (0.92, 0.96, 0.92)},
    "small": {"label": "Small body", "heightM": 1.2, "bodyClass": "HumanoidLight", "scale": (0.72, 0.72, 0.72)},
    "large": {"label": "Large body", "heightM": 2.1, "bodyClass": "HumanoidHeavy", "scale": (1.28, 1.22, 1.28)},
}

PART_SPECS: list[dict] = [
    {"id": "head_round_01", "category": "heads", "mesh_size": (0.22, 0.24, 0.22), "bases": ["male", "female", "small", "large"]},
    {"id": "head_square_01", "category": "heads", "mesh_size": (0.24, 0.22, 0.2), "bases": ["male", "female", "small"]},
    {"id": "head_angular_01", "category": "heads", "mesh_size": (0.2, 0.26, 0.18), "bases": ["male", "female", "large"]},
    {"id": "hair_short_01", "category": "hair", "mesh_size": (0.26, 0.12, 0.26), "bases": ["male", "female", "small", "large"]},
    {"id": "hair_long_01", "category": "hair", "mesh_size": (0.28, 0.28, 0.2), "bases": ["male", "female"]},
    {"id": "armor_chest_plate_01", "category": "armor", "mesh_size": (0.42, 0.38, 0.22), "bases": ["male", "female", "large"]},
    {"id": "armor_chest_leather_01", "category": "armor", "mesh_size": (0.38, 0.34, 0.2), "bases": ["male", "female", "small"]},
    {"id": "weapon_sword_01", "category": "weapons", "mesh_size": (0.06, 0.7, 0.12), "bases": ["male", "female", "small", "large"]},
    {"id": "weapon_axe_01", "category": "weapons", "mesh_size": (0.28, 0.5, 0.1), "bases": ["male", "female", "large"]},
    {"id": "hat_cap_01", "category": "hats", "mesh_size": (0.24, 0.1, 0.24), "bases": ["male", "female", "small", "large"]},
]

CATEGORY_SOCKET = {
    "heads": "TBG_Head", "hair": "TBG_Head", "hats": "TBG_Head",
    "armor": "TBG_Spine1", "weapons": "TBG_Hand.R",
}


def seed_bases() -> None:
    if not TEMPLATE.is_file():
        raise FileNotFoundError(f"missing template GLB: {TEMPLATE}")
    for base_id, profile in BASE_PROFILES.items():
        base_dir = LIBRARY / "bases" / base_id
        body_path = base_dir / "body.glb"
        copy_scaled_body(TEMPLATE, body_path, tuple(profile["scale"]))
        rig_path = LIBRARY / "rigs" / base_id / "armature.glb"
        extract_armature_only(body_path, rig_path)
        manifest = {
            "id": base_id,
            "label": profile["label"],
            "heightM": profile["heightM"],
            "bodyClass": profile["bodyClass"],
            "mesh": "body.glb",
            "rig": f"../../rigs/{base_id}/armature.glb",
        }
        (base_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def seed_parts() -> None:
    for spec in PART_SPECS:
        category = spec["category"]
        part_dir = LIBRARY / "parts" / category
        mesh_name = f"{spec['id']}.glb"
        write_box_part(part_dir / mesh_name, tuple(spec["mesh_size"]))
        thumb = part_dir / f"{spec['id']}.webp"
        if not thumb.is_file():
            _write_placeholder_thumb(thumb)
        manifest = {
            "id": spec["id"],
            "category": category,
            "baseCompat": spec["bases"],
            "socket": CATEGORY_SOCKET[category],
            "mesh": mesh_name,
            "thumbnail": f"{spec['id']}.webp",
        }
        (part_dir / f"{spec['id']}.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def _write_placeholder_thumb(path: Path) -> None:
    from PIL import Image

    path.parent.mkdir(parents=True, exist_ok=True)
    color = (90, 110, 140) if "weapon" in path.stem else (160, 140, 120)
    Image.new("RGB", (128, 128), color).save(path, format="WEBP")


def main() -> None:
    seed_bases()
    seed_parts()
    print(f"Seeded library under {LIBRARY.relative_to(REPO)}")


if __name__ == "__main__":
    main()
