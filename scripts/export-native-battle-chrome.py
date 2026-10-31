#!/usr/bin/env python3
"""Export small Linux HUD shapes matching the existing browser CSS geometry."""

from pathlib import Path
import subprocess
import tempfile

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / 'crates/tbg-client/assets/battle/native'


def export(name, width, height, body):
    source = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}">{body}</svg>'
    with tempfile.TemporaryDirectory() as temp:
        svg = Path(temp) / 'shape.svg'
        png = Path(temp) / 'shape.png'
        svg.write_text(source)
        subprocess.run(['inkscape', str(svg), '--export-type=png',
                        f'--export-filename={png}', f'--export-width={width * 2}'],
                       check=True, capture_output=True)
        with Image.open(png) as image:
            image.save(TARGET / f'{name}.webp', lossless=True, method=6)


def main():
    TARGET.mkdir(exist_ok=True)
    for team, top, bottom in [('ally', '#4cc4ff', '#0b4b88'),
                              ('enemy', '#ff7a7f', '#8e0f22')]:
        export(f'crest-{team}', 42, 48, f'''
          <defs><linearGradient id="g" x2="0" y2="1"><stop stop-color="#f3d79a"/><stop offset="1" stop-color="#9a7444"/></linearGradient>
          <linearGradient id="t" x2=".3" y2="1"><stop stop-color="{top}"/><stop offset="1" stop-color="{bottom}"/></linearGradient></defs>
          <path d="M0 0H42V28L21 48 0 28Z" fill="url(#g)"/>
          <path d="M3 3H39V27L21 44 3 27Z" fill="url(#t)"/>''')
    export('resources', 860, 44, '''
      <defs><linearGradient id="e" x2="1" y2="1"><stop stop-color="#f2deb1"/><stop offset=".38" stop-color="#95764b"/><stop offset=".65" stop-color="#d5b882"/><stop offset="1" stop-color="#746045"/></linearGradient>
      <linearGradient id="b" x2=".6" y2="1"><stop stop-color="#263440"/><stop offset=".48" stop-color="#111e2b"/><stop offset="1" stop-color="#0b1421"/></linearGradient></defs>
      <path d="M11 0H849L860 11V33L849 44H11L0 33V11Z" fill="url(#e)"/>
      <path d="M12 3H848L857 12V32L848 41H12L3 32V12Z" fill="#050b12"/>
      <path d="M14 5H846L855 14V30L846 39H14L5 30V14Z" fill="url(#b)"/>''')
    export('flag', 17, 19, '<path d="M3 1h2v17H3zM5 2l10 2-2 5-8-2z" fill="#e45b5b"/>')


if __name__ == '__main__':
    main()
