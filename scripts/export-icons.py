#!/usr/bin/env python3
"""Export platform icons from the approved raster emblem using Pillow."""

import json
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / "art/branding"
RUNTIME = ROOT / "crates/tbg-client/assets/branding"
BACKGROUND = (12, 19, 33, 255)
SIZES = (16, 24, 32, 48, 64, 128, 256, 512, 1024)


def fit(source, size, fraction=0.86, opaque=False):
    """Keep the whole spear visible and let platform masks supply the corners."""
    image = source.copy()
    side = round(size * fraction)
    image.thumbnail((side, side), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (size, size), BACKGROUND if opaque else (0, 0, 0, 0))
    canvas.alpha_composite(image, ((size - image.width) // 2, (size - image.height) // 2))
    return canvas.convert("RGB") if opaque else canvas


def save(image, path):
    path.parent.mkdir(parents=True, exist_ok=True)
    image.save(path, optimize=True)


def main():
    source = Image.open(ART / "tbg-emblem-master.png").convert("RGBA")
    # Ignore almost transparent fringe only when measuring the content box.
    # Pixels and antialiasing inside that box remain unchanged.
    box = source.getchannel("A").point(lambda value: 255 if value >= 32 else 0).getbbox()
    source = source.crop(box)
    for size in SIZES:
        save(fit(source, size), ART / f"desktop/tbg-{size}.png")
    RUNTIME.mkdir(parents=True, exist_ok=True)
    save(fit(source, 512), RUNTIME / "tbg-icon-512.png")
    fit(source, 512).save(RUNTIME / "tbg-emblem.webp", lossless=True, method=6)
    fit(source, 256).save(RUNTIME / "tbg.ico", sizes=[(s, s) for s in SIZES if s <= 256])
    save(fit(source, 180, opaque=True), RUNTIME / "apple-touch-icon.png")

    ios = ART / "ios/AppIcon.appiconset"
    save(fit(source, 1024, fraction=0.78, opaque=True), ios / "AppIcon-1024.png")
    (ios / "Contents.json").write_text(json.dumps({
        "images": [{"filename": "AppIcon-1024.png", "idiom": "universal",
                    "platform": "ios", "size": "1024x1024"}],
        "info": {"author": "xcode", "version": 1},
    }, indent=2) + "\n")

    android = ART / "android/res"
    for density, scale in (("mdpi", 1), ("hdpi", 1.5), ("xhdpi", 2),
                           ("xxhdpi", 3), ("xxxhdpi", 4)):
        foreground = fit(source, round(108 * scale), fraction=64 / 108)
        save(foreground, android / f"drawable-{density}/tbg_foreground.png")
        mono = Image.new("RGBA", foreground.size, "white")
        mono.putalpha(foreground.getchannel("A"))
        save(mono, android / f"drawable-{density}/tbg_monochrome.png")
        save(fit(source, round(48 * scale), fraction=0.72, opaque=True),
             android / f"mipmap-{density}/ic_launcher.png")
    save(fit(source, 512, fraction=0.78, opaque=True), ART / "android/play-store-512.png")
    values = android / "values"
    values.mkdir(parents=True, exist_ok=True)
    (values / "tbg_icon.xml").write_text(
        '<resources><color name="tbg_icon_background">#0C1321</color></resources>\n')
    for api, monochrome in ((26, ""), (33, '\n    <monochrome android:drawable="@drawable/tbg_monochrome" />')):
        directory = android / f"mipmap-anydpi-v{api}"
        directory.mkdir(parents=True, exist_ok=True)
        (directory / "ic_launcher.xml").write_text(
            '<?xml version="1.0" encoding="utf-8"?>\n'
            '<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">\n'
            '    <background android:drawable="@color/tbg_icon_background" />\n'
            '    <foreground android:drawable="@drawable/tbg_foreground" />'
            + monochrome + '\n</adaptive-icon>\n')


if __name__ == "__main__":
    main()
