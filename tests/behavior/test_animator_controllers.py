"""Tests for animator controller generator."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools" / "unity"))

from generate_animator_controllers import BODY_CLASSES, TRIGGERS, build_controller_yaml  # noqa: E402


def test_controller_yaml_includes_triggers_and_attack_state() -> None:
    yaml_text = build_controller_yaml("HumanoidHeavy", "AttackHeavy")
    for trigger in TRIGGERS:
        assert f"m_Name: {trigger}" in yaml_text
    assert "m_Name: AttackHeavy" in yaml_text
    assert "m_Name: Idle" in yaml_text


def test_generate_all_writes_four_body_classes(tmp_path: Path) -> None:
    from generate_animator_controllers import generate_all  # noqa: E402

    paths = generate_all(tmp_path)
    assert len(paths) == len(BODY_CLASSES)
    for body_class in BODY_CLASSES:
        assert (tmp_path / f"{body_class}.controller").is_file()
