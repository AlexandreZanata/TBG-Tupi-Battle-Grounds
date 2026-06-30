"""Procedural card PNG renderer — headless pipeline for CI and Unity parity."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

from .defaults import EVOLUTION_MODIFIERS
from .models import CharacterDefinition


CARD_WIDTH = 512
CARD_HEIGHT = 768
BACKGROUND = "#1A1A2E"
FRAME = "#E8C547"


def _hex_to_rgb(hex_color: str) -> tuple[int, int, int]:
    value = hex_color.lstrip("#")
    return tuple(int(value[i : i + 2], 16) for i in (0, 2, 4))


def render_card(defn: CharacterDefinition, output_path: Path) -> Path:
    if defn.assetMode == "mesh3d":
        return _render_mesh3d_card(defn, output_path)
    return _render_procedural_card(defn, output_path)


def _render_procedural_card(defn: CharacterDefinition, output_path: Path) -> Path:
    image = Image.new("RGB", (CARD_WIDTH, CARD_HEIGHT), _hex_to_rgb(BACKGROUND))
    draw = ImageDraw.Draw(image)

    _draw_frame(draw)
    center_x = CARD_WIDTH // 2
    base_y = CARD_HEIGHT - 180
    modifiers = EVOLUTION_MODIFIERS[defn.evolutionStage]
    scale = modifiers.scale_multiplier

    primary = _hex_to_rgb(defn.primaryColor)
    secondary = _hex_to_rgb(defn.secondaryColor)
    if modifiers.saturate_colors:
        primary = _saturate(primary, 1.25)
        secondary = _saturate(secondary, 1.25)

    _draw_silhouette(draw, center_x, base_y, defn, primary, secondary, scale)
    _draw_labels(draw, defn)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(output_path, format="PNG")
    return output_path


def _render_mesh3d_card(defn: CharacterDefinition, output_path: Path) -> Path:
    image = Image.new("RGB", (CARD_WIDTH, CARD_HEIGHT), _hex_to_rgb(BACKGROUND))
    draw = ImageDraw.Draw(image)
    _draw_frame(draw)

    primary = _hex_to_rgb(defn.primaryColor)
    draw.rectangle((80, 180, CARD_WIDTH - 80, CARD_HEIGHT - 220), fill=primary)
    draw.text((40, 40), "MESH3D", fill=(255, 255, 255))
    if defn.mesh:
        mesh_name = Path(defn.mesh).name
        draw.text((40, 70), mesh_name, fill=(200, 200, 200))
    _draw_labels(draw, defn)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    image.save(output_path, format="PNG")
    return output_path


def _saturate(rgb: tuple[int, int, int], factor: float) -> tuple[int, int, int]:
    return tuple(min(255, int(channel * factor)) for channel in rgb)


def _draw_frame(draw: ImageDraw.ImageDraw) -> None:
    margin = 24
    draw.rectangle(
        (margin, margin, CARD_WIDTH - margin, CARD_HEIGHT - margin),
        outline=_hex_to_rgb(FRAME),
        width=4,
    )


def _draw_silhouette(
    draw: ImageDraw.ImageDraw,
    center_x: int,
    base_y: int,
    defn: CharacterDefinition,
    primary: tuple[int, int, int],
    secondary: tuple[int, int, int],
    scale: float,
) -> None:
    unit_h = int(220 * defn.heightScale * scale)
    torso_w = int(70 * defn.bodyWidthScale * scale)
    head_r = int(28 * defn.headScale * scale)
    limb_h = int(80 * defn.limbLengthScale * scale)

    torso_top = base_y - unit_h
    head_cy = torso_top - head_r - 8
    draw.ellipse(
        (center_x - head_r, head_cy - head_r, center_x + head_r, head_cy + head_r),
        fill=primary,
    )
    draw.rectangle(
        (center_x - torso_w // 2, torso_top, center_x + torso_w // 2, base_y - 40),
        fill=primary,
    )
    foot_w = max(20, torso_w // 3)
    draw.rectangle(
        (center_x - torso_w // 2, base_y - 40, center_x - torso_w // 2 + foot_w, base_y),
        fill=secondary,
    )
    draw.rectangle(
        (center_x + torso_w // 2 - foot_w, base_y - 40, center_x + torso_w // 2, base_y),
        fill=secondary,
    )
    arm_w = 16
    draw.rectangle(
        (center_x - torso_w // 2 - arm_w, torso_top + 20, center_x - torso_w // 2, torso_top + 20 + limb_h),
        fill=secondary,
    )
    draw.rectangle(
        (center_x + torso_w // 2, torso_top + 20, center_x + torso_w // 2 + arm_w, torso_top + 20 + limb_h),
        fill=secondary,
    )
    _draw_accessories(draw, center_x, torso_top, torso_w, defn, secondary)


def _draw_accessories(
    draw: ImageDraw.ImageDraw,
    center_x: int,
    torso_top: int,
    torso_w: int,
    defn: CharacterDefinition,
    color: tuple[int, int, int],
) -> None:
    for accessory in defn.accessories:
        if accessory == "Shield":
            draw.ellipse(
                (center_x - torso_w - 30, torso_top + 30, center_x - torso_w + 10, torso_top + 90),
                fill=color,
            )
        elif accessory == "Weapon":
            draw.rectangle(
                (center_x + torso_w // 2 + 20, torso_top, center_x + torso_w // 2 + 28, torso_top + 100),
                fill=color,
            )
        elif accessory == "Helmet":
            draw.rectangle(
                (center_x - 20, torso_top - 50, center_x + 20, torso_top - 30),
                fill=color,
            )
        elif accessory == "ArmorBlock":
            draw.rectangle(
                (center_x - torso_w // 2 + 5, torso_top + 10, center_x + torso_w // 2 - 5, torso_top + 50),
                fill=color,
            )


def _draw_labels(draw: ImageDraw.ImageDraw, defn: CharacterDefinition) -> None:
    title = defn.characterName
    subtitle = f"{defn.archetype} · {defn.faction}"
    draw.text((40, CARD_HEIGHT - 140), title, fill=(255, 255, 255))
    draw.text((40, CARD_HEIGHT - 110), subtitle, fill=(200, 200, 200))
    draw.text((40, CARD_HEIGHT - 80), f"Stage {defn.evolutionStage}", fill=(180, 180, 180))
