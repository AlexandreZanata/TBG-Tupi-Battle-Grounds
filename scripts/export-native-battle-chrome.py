#!/usr/bin/env python3
"""Export Linux HUD shapes and the existing browser Stronghold symbols."""

from pathlib import Path
import subprocess
import tempfile
import xml.etree.ElementTree as ET

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
    towers()
    shop_card()
    export('council-energy',24,20,'''<path d="M4 8V5Q4 2 8 2H16Q20 2 20 5V8" fill="none" stroke="white" stroke-width="2"/><rect x="2" y="7" width="20" height="12" rx="2" fill="white"/><path d="M7 8v10M17 8v10" stroke="#28323b" stroke-width="1.5"/><rect x="10" y="10" width="4" height="5" fill="#28323b"/>''')
    for state, shackle in [('closed', 'M7 11V7a5 5 0 0 1 10 0v4'),
                            ('open', 'M7 11V7a5 5 0 0 1 9-3')]:
        export('shop-lock-' + state, 24, 26, f'''
          <path d="{shackle}" fill="none" stroke="#f6d680" stroke-width="2.5" stroke-linecap="round"/>
          <rect x="4" y="10" width="16" height="13" rx="2" fill="#243545" stroke="#f6d680" stroke-width="1.5"/>
          <circle cx="12" cy="15" r="2" fill="#f6d680"/><path d="M12 16v3" stroke="#f6d680" stroke-width="2"/>''')


def towers():
    sprite = ET.parse(ROOT / 'crates/tbg-client/assets/stronghold/towers.svg').getroot()
    ns = '{http://www.w3.org/2000/svg}'
    definitions = ET.tostring(sprite.find(ns + 'defs'), encoding='unicode')
    for symbol in sprite.findall(ns + 'symbol'):
        body = definitions + ''.join(ET.tostring(child, encoding='unicode') for child in symbol)
        export('tower-' + symbol.attrib['id'].removeprefix('sh-'), 120, 150, body)


def shop_card():
    export('shop-card', 166, 224, '''
      <defs><linearGradient id="card" x2=".7" y2="1"><stop stop-color="#203647"/><stop offset=".5" stop-color="#142029"/><stop offset="1" stop-color="#091522"/></linearGradient>
      <linearGradient id="gold" x2="1" y2="1"><stop stop-color="#f7da87"/><stop offset=".5" stop-color="#c79d43"/><stop offset="1" stop-color="#f6d878"/></linearGradient>
      <linearGradient id="cost" x2="0" y2="1"><stop stop-color="#1e3244"/><stop offset="1" stop-color="#07111c"/></linearGradient></defs>
      <rect x="2" y="2" width="162" height="220" rx="10" fill="url(#card)" stroke="#07101a" stroke-width="4"/>
      <path d="M4 8L10 3H156L163 10V214L156 221H10L3 214V10Z" fill="none" stroke="url(#gold)" stroke-width="1.5"/>
      <path d="M7 17L17 7H149L159 17V207L149 217H17L7 207Z" fill="none" stroke="#e4bc57" stroke-width="1"/>
      <path d="M9 9h10L9 19ZM157 9h-10l10 10ZM9 215h10L9 205ZM157 215h-10l10-10Z" fill="none" stroke="#e4bc57" stroke-width="1"/>
      <path d="M10 190H156L160 195V211L153 218H13L6 211V195Z" fill="url(#cost)" stroke="#c8a66a" stroke-width="1"/>
      <path d="M11 187H155" stroke="#597586" stroke-opacity=".5"/>
      <path d="M10 20L76 9H151L10 153Z" fill="#456780" opacity=".07"/>''')


if __name__ == '__main__':
    main()
