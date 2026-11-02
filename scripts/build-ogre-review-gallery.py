#!/usr/bin/env python3
"""Build the local Ogre review from authored briefs, concepts and source-stamped native captures."""
import argparse
from html import escape
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--art-root',type=Path,required=True)
    args=parser.parse_args();art=args.art_root.resolve()
    catalog=json.loads((ROOT/'docs/ogre-faction/catalog.json').read_text())
    records=json.loads((art/'reference-manifest.json').read_text())['images']
    concepts={c['kind']:c['image'] for c in records}
    engine='engine-final' if (art/'engine-final/capture-manifest.json').exists() else 'engine'
    cards=[]
    for c in catalog['champions']:
        k=c['kind'];slug=k.lower();portrait=art/engine/f'{slug}-portrait.png';lineup=art/engine/f'{slug}-lineup.png'
        actual=(f'<h3>Modelo real no motor Linux</h3><img class="portrait" src="{engine}/{slug}-portrait.png" alt="{escape(c["name"]["pt"])}"><img src="{engine}/{slug}-lineup.png" alt="Cinco formas em três poses">' if portrait.is_file() and lineup.is_file() else '<p>Captura real do motor em preparação.</p>')
        forms=''.join(f'<li>{f["code"]}: {escape(f["pt"])} / {escape(f["en"])}</li>' for f in c['forms'])
        cards.append(f'<article id="{slug}"><h2>{escape(c["name"]["pt"])} <small>{escape(c["name"]["en"])}</small></h2><p>{c["faction"]} · {c["archetype"]}</p>{actual}<p>{escape(c.get("kit_pt",c["kit"]))}</p><ul>{forms}</ul><details><summary>Referência ilustrativa de conceito · Base, A1, A2, B1, B2</summary><img src="{concepts[k]}" alt="Conceito low-poly de cinco formas"><p>Ilustração de referência; as capturas reais do modelo aparecem acima quando prontas.</p></details></article>')
    old=art/engine/'siegewalker-lineup.png'
    existing='<article><h2>Arqueiro Ogro / Ogre Archer</h2><p>Transferido para Ogros. Alcance igual ao Arqueiro nas cinco formas; reserva visual inclui bazuca e braços, sem pedestal.</p>'+(f'<img src="{engine}/{old.name}" alt="Arqueiro Ogro: cinco formas sem pedestal">' if old.is_file() else '')+'</article>'
    css='*{box-sizing:border-box}body{margin:0;background:#14202a;color:#eee8dd;font:16px/1.6 system-ui}main{max-width:1500px;margin:auto;padding:36px}article{padding:24px;background:#1b2935;border:1px solid #42505c;margin:26px 0;border-radius:10px}img{width:100%;display:block;margin:18px 0}.portrait{width:240px;height:240px}small{font-size:16px;color:#c6b698}a,summary{color:#dfbf85}summary{cursor:pointer}h1,h2{line-height:1.2}li{margin:5px 0}'
    nav=' · '.join(f'<a href="#{c["kind"].lower()}">{escape(c["name"]["pt"])}</a>' for c in catalog['champions'])
    page=f'<!doctype html><html lang="pt-BR"><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>Ogros · TBG</title><style>{css}</style><main><h1>Ogros · nove campeões na nova facção</h1><p>Oito novos Ogros e a Balista de Rebites para Máquinas. Catálogo completo: 72 campeões, oito Facções, nove por Facção.</p><p>Ogros: 1 tanque, 3 corpo a corpo, 2 de distância, 2 de controle, 1 suporte. Novos campeões em teste na prévia local. Windows e web ainda aguardam comparação visual.</p><nav>{nav}</nav>{existing}{"".join(cards)}<footer>As referências de IA e as capturas do motor preservam suas próprias origens; conferir os manifests de cada pasta.</footer></main></html>'
    (art/'index.html').write_text(page)
    print(art/'index.html')
if __name__=='__main__':main()
