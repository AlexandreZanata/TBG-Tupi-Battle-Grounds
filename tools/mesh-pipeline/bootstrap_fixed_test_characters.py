#!/usr/bin/env python3
"""Create definition JSON + pilot roster entries for fixed test sculpt roster."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "content/characters/fixed_test_imports.json"
DEFINITIONS_DIR = ROOT / "content/characters/definitions"
ROSTER_PATH = ROOT / "content/characters/pilot_roster.json"
SHOWCASE_ORDER_JS = ROOT / "tools/character_studio/static/js/showcase/showcaseOrder.js"

NEW_CHARACTERS: tuple[dict[str, str], ...] = (
    {
        "id": "demo_enforcer_01",
        "characterName": "Enforcer Pilot",
        "archetype": "Tank",
        "bodyClass": "HumanoidHeavy",
        "behaviorProfile": "tank_melee_v1",
        "animationSet": "humanoid_heavy_combat_v1",
        "animatorController": "content/characters/animations/HumanoidHeavy.controller",
        "accessories": ["Shield"],
    },
    {
        "id": "demo_skirmisher_01",
        "characterName": "Skirmisher Pilot",
        "archetype": "DPS",
        "bodyClass": "HumanoidLight",
        "behaviorProfile": "dps_melee_v1",
        "animationSet": "humanoid_light_combat_v1",
        "animatorController": "content/characters/animations/HumanoidLight.controller",
        "accessories": ["Weapon"],
    },
    {
        "id": "demo_oracle_01",
        "characterName": "Oracle Pilot",
        "archetype": "Control",
        "bodyClass": "HumanoidHeavy",
        "behaviorProfile": "control_cast_v1",
        "animationSet": "humanoid_heavy_combat_v1",
        "animatorController": "content/characters/animations/HumanoidHeavy.controller",
        "accessories": ["Helmet"],
    },
    {
        "id": "demo_ranger_01",
        "characterName": "Ranger Pilot",
        "archetype": "Ranged",
        "bodyClass": "HumanoidLight",
        "behaviorProfile": "ranged_shot_v1",
        "animationSet": "humanoid_light_combat_v1",
        "animatorController": "content/characters/animations/HumanoidLight.controller",
        "accessories": ["Weapon"],
    },
    {
        "id": "demo_warden_01",
        "characterName": "Warden Pilot",
        "archetype": "Tank",
        "bodyClass": "HumanoidHeavy",
        "behaviorProfile": "tank_melee_v1",
        "animationSet": "humanoid_heavy_combat_v1",
        "animatorController": "content/characters/animations/HumanoidHeavy.controller",
        "accessories": ["Shield"],
    },
    {
        "id": "demo_gladiator_01",
        "characterName": "Gladiator Pilot",
        "archetype": "DPS",
        "bodyClass": "HumanoidLight",
        "behaviorProfile": "dps_melee_v1",
        "animationSet": "humanoid_light_combat_v1",
        "animatorController": "content/characters/animations/HumanoidLight.controller",
        "accessories": ["Weapon"],
    },
    {
        "id": "demo_raider_01",
        "characterName": "Raider Pilot",
        "archetype": "DPS",
        "bodyClass": "HumanoidLight",
        "behaviorProfile": "dps_melee_v1",
        "animationSet": "humanoid_light_combat_v1",
        "animatorController": "content/characters/animations/HumanoidLight.controller",
        "accessories": ["Weapon"],
    },
)


def _definition(record: dict[str, str]) -> dict:
    unit_id = record["id"]
    return {
        "id": unit_id,
        "characterName": record["characterName"],
        "archetype": record["archetype"],
        "faction": "Humans",
        "assetMode": "mesh3d",
        "meshShowcase": f"content/characters/meshes/showcase/{unit_id}.glb",
        "mesh": f"content/characters/meshes/game/{unit_id}.glb",
        "animationSet": record["animationSet"],
        "materialProfile": "humans_pbr_v1",
        "behaviorProfile": record["behaviorProfile"],
        "animatorController": record["animatorController"],
        "heightScale": 1.0,
        "bodyWidthScale": 1.0,
        "headScale": 1.0,
        "limbLengthScale": 1.0,
        "primaryColor": "#FFFFFF",
        "secondaryColor": "#FFFFFF",
        "accessories": record["accessories"],
        "evolutionStage": 0,
    }


def _pilot_entry(record: dict[str, str]) -> dict:
    return {
        "id": record["id"],
        "bodyClass": record["bodyClass"],
        "faction": "Humans",
        "behaviorProfile": record["behaviorProfile"],
        "materialProfile": "humans_pbr_v1",
        "animatorController": record["animatorController"],
    }


def _write_showcase_order(order: list[str]) -> None:
    lines = ["/** Showcase sculpt display order. */", "", "export const SHOWCASE_ORDER = ["]
    lines.extend(f'  "{unit_id}",' for unit_id in order)
    lines.append("];")
    lines.append("")
    SHOWCASE_ORDER_JS.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    order = list(manifest["showcaseOrder"])
    new_ids = {entry["id"] for entry in NEW_CHARACTERS}

    for record in NEW_CHARACTERS:
        path = DEFINITIONS_DIR / f"{record['id']}.json"
        path.write_text(json.dumps(_definition(record), indent=2) + "\n", encoding="utf-8")
        print(f"definition: {path.relative_to(ROOT)}")

    roster = json.loads(ROSTER_PATH.read_text(encoding="utf-8"))
    pilots = [entry for entry in roster["pilots"] if entry["id"] not in new_ids]
    pilots.extend(_pilot_entry(record) for record in NEW_CHARACTERS)
    pilots.sort(key=lambda entry: order.index(entry["id"]))
    roster["pilots"] = pilots
    ROSTER_PATH.write_text(json.dumps(roster, indent=2) + "\n", encoding="utf-8")
    print(f"roster: {ROSTER_PATH.relative_to(ROOT)} ({len(pilots)} pilots)")

    _write_showcase_order(order)
    print(f"showcase order: {SHOWCASE_ORDER_JS.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
