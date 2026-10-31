#!/usr/bin/env python3
"""Export existing lobby art and code-native account glyphs to lossless WebP."""

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
    account_icons()


def account_icons():
    icons = {
        'account-user': '<circle cx="12" cy="6" r="4"/><path d="M4 22v-4a8 8 0 0 1 16 0v4Z"/>',
        'account-add': '<circle cx="9" cy="6" r="3.5"/><path d="M2 21v-4a7 7 0 0 1 12-5v9Z"/><path d="M18 10v10m-5-5h10" fill="none" stroke="white" stroke-width="2.5" stroke-linecap="round"/>',
        'account-key': '<circle cx="16" cy="7" r="5" fill="none" stroke="white" stroke-width="3"/><path d="M12.5 10.5 3 20m1-1 3 3m0-6 3 3" fill="none" stroke="white" stroke-width="3" stroke-linejoin="round"/>',
    }
    target = ART / 'native-icons'
    target.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as temp:
        source = Path(temp) / 'account.svg'
        for name, body in icons.items():
            source.write_text(f'<svg xmlns="{NS}" viewBox="0 0 24 24" fill="white">{body}</svg>')
            render(source, target / f'{name}.webp', 96)


if __name__ == '__main__':
    main()
