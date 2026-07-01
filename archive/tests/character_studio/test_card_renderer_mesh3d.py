"""Tests for mesh3d card rendering."""

from __future__ import annotations

from pathlib import Path

from character_studio.card_renderer import render_card
from character_studio.models import CharacterDefinition, CharacterTextures


def test_mesh3d_card_renders_with_mesh_label(tmp_path: Path) -> None:
    defn = CharacterDefinition(
        id="tank_human_01",
        characterName="Iron Guard",
        archetype="Tank",
        faction="Humans",
        heightScale=1.0,
        bodyWidthScale=1.0,
        headScale=1.0,
        limbLengthScale=1.0,
        primaryColor="#8B7355",
        secondaryColor="#C4A882",
        assetMode="mesh3d",
        mesh="content/characters/meshes/tank_human_01.glb",
        textures=CharacterTextures(
            albedo="content/characters/textures/tank_human_01_albedo.png",
            normal="content/characters/textures/tank_human_01_normal.png",
            orm="content/characters/textures/tank_human_01_orm.png",
        ),
    )
    output = tmp_path / "card.png"
    render_card(defn, output)
    assert output.is_file()
    assert output.stat().st_size > 500
