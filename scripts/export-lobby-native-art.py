#!/usr/bin/env python3
"""Rasterize the existing browser icon sprite/news into lossless native WebP assets."""

from copy import deepcopy
from pathlib import Path
import subprocess
import tempfile
import xml.etree.ElementTree as ET

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ART = ROOT / 'crates/tbg-client/assets/lobby'
NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)


def render(source, destination, width):
    with tempfile.TemporaryDirectory() as temp:
        png = Path(temp) / 'render.png'
        subprocess.run(['inkscape', str(source), '--export-type=png',
                        f'--export-filename={png}', f'--export-width={width}'],
                       check=True, capture_output=True)
        with Image.open(png) as image:
            image.save(destination, lossless=True, method=6)


def main():
    target = ART / 'native-icons'
    target.mkdir(exist_ok=True)
    sprite = ET.parse(ART / 'icons.svg').getroot()
    with tempfile.TemporaryDirectory() as temp:
        for symbol in sprite:
            root = ET.Element(f'{{{NS}}}svg', {
                'viewBox': symbol.attrib['viewBox'], 'width': '96', 'height': '96',
                'fill': 'white', 'color': 'white',
            })
            root.extend(deepcopy(list(symbol)))
            source = Path(temp) / 'icon.svg'
            ET.ElementTree(root).write(source)
            render(source, target / f'{symbol.attrib["id"]}.webp', 96)
    render(ART / 'news.svg', ART / 'news.webp', 1000)


if __name__ == '__main__':
    main()
