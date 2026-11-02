#!/usr/bin/env python3
"""Build the local AB-01/OG-02 reference/native-image review from verified captures."""
import argparse
import hashlib
from html import escape
import json
from pathlib import Path
import re
import shutil

ROOT = Path(__file__).resolve().parents[1]
OGRES = ['SiegeWalker','BasaltBulwark','CinderMaul','HookBrawler','FlintRunner','ClayBinder','GustDrummer','HideMender','PebbleSlinger']
NEW = ['NacreDuelist','ReefLancer','TrenchFist','PearlSlinger','AbyssHarpooner','UndertowHerald','TrenchLamplighter','FoamWeaver']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--ogre-concepts', type=Path, required=True)
    args = parser.parse_args()
    out = args.output.resolve()
    manifest = json.loads((out/'engine-final/capture-manifest.json').read_text())
    images = {}
    for c in manifest['captures']:
        path = out/'engine-final'/c['image']
        if hashlib.sha256(path.read_bytes()).hexdigest() != c['sha256']:
            raise SystemExit(f'Capture hash changed: {path}')
        images[(c['kind'], len(c['forms']))] = 'engine-final/'+c['image']
    i18n = {lang:json.loads((ROOT/f'crates/tbg-client/assets/i18n/{lang}.json').read_text()) for lang in ['en','pt-BR']}
    content = {c['kind']: c for c in json.loads((ROOT/'docs/abyssal-faction/catalog.json').read_text())['champions']}
    references = out/'references'
    references.mkdir(exist_ok=True)
    cards = []
    categories = [('ogres','Ogros refinados',OGRES),('abyssals','Abissais',['Trident',*NEW]),('plants','Substituto das Plantas',['PodSnapper'])]
    for category, title, kinds in categories:
        cards.append(f'<section id="{category}"><h2>{title} <span>{len(kinds)}</span></h2>')
        for kind in kinds:
            key = 'unit.'+re.sub(r'(?<!^)(?=[A-Z])','_',kind).lower()
            pt, en = i18n['pt-BR'][key+'.name'], i18n['en'][key+'.name']
            portrait, forms = images[(kind,1)], images[(kind,5)]
            intro = i18n['pt-BR'][key+'.lore']
            if kind == 'Trident': intro = 'Transferido das Plantas para os Abissais. Mantém o kit de linha de frente e as evoluções existentes.'
            if kind == 'GustDrummer': intro = 'Carrega e rola um barril até o primeiro contato. Cura em área; uma interceptação inimiga causa pouco dano. Recarga base de 10 segundos.'
            evolution = ''.join(f'<li><b>{code}</b> {escape(i18n["pt-BR"][key+f".form.{i}.name"])} <small>{escape(i18n["en"][key+f".form.{i}.name"])}</small></li>' for i,code in enumerate(['A1','A2','B1','B2']))
            cards.append(f'<article id="{kind.lower()}"><div class="intro"><img class="portrait" src="{portrait}" alt="Modelo real de {escape(pt)}" loading="lazy"><div><p class="eyebrow">{title}</p><h3>{escape(pt)} <small>{escape(en)}</small></h3><p>{escape(intro)}</p><ul>{evolution}</ul></div></div><a href="{forms}" target="_blank"><img class="forms" src="{forms}" alt="Base, A1, A2, B1, B2 em repouso, caminhada e ataque" loading="lazy"></a><p class="caption">Captura real no Linux · colunas: Base, A1, A2, B1, B2 · linhas: repouso, caminhada, ataque. Clique para ampliar.</p>')
            concept = None
            if kind == 'HookBrawler': concept = out/'hook-brawler-owner-reference.png'
            elif kind == 'GustDrummer': concept = out/'concepts/barrelkeeper.png'
            elif kind in content: concept = out/f'concepts/{kind.lower()}.png'
            elif kind in OGRES: concept = args.ogre_concepts/f'{kind.lower()}.png'
            if concept and concept.exists():
                dest = references/f'{kind.lower()}.png'
                shutil.copy2(concept, dest)
                cards.append(f'<details><summary>Ver referência visual · cinco formas</summary><a href="references/{dest.name}" target="_blank"><img src="references/{dest.name}" alt="Referência visual de {escape(pt)}" loading="lazy"></a><p class="caption">Referência ilustrada para direção de arte; os modelos reais são as capturas acima.</p></details>')
            cards.append('</article>')
        cards.append('</section>')
    training = out/'final/training/linux-training.json'
    if training.exists():
        proof = json.loads(training.read_text())
        labels = {'barrel-allies':'Barril: cura dos aliados', 'barrel-intercept':'Barril: interceptação inimiga',
                  'abyssal-crowd':'Abissais no laboratório', 'barrel-rear':'Mestre do Barril: aliados atrás',
                  'foam-rear':'Tecelã de Espuma: aliados atrás'}
        cards.append('<section id="training"><h2>Laboratório Linux</h2><p>Cenas reais de treino com os novos campeões.</p>')
        for scene in proof['scenes']:
            label = escape(labels[scene['case']])
            cards.append(f'<article><h3>{label}</h3><a href="final/training/{scene["image"]}" target="_blank"><img src="final/training/{scene["image"]}" alt="{label}" loading="lazy"></a></article>')
        cards.append('</section>')
    style = '''html{scroll-behavior:smooth;scroll-padding-top:90px}*{box-sizing:border-box}body{margin:0;background:#101b22;color:#eef1df;font:17px/1.6 system-ui,sans-serif}header,main,footer{max-width:1180px;margin:auto;padding:36px 28px}header{padding-top:65px}h1{font-size:clamp(34px,5vw,64px);line-height:1.08;max-width:900px;margin:12px 0}h2{font-size:32px;border-bottom:1px solid #37505a;padding-bottom:12px;margin:55px 0 25px}h2 span{font-size:18px;color:#9bdbd1}h3{font-size:29px;line-height:1.2;margin:0 0 14px}small{display:block;color:#9cafb1;font-size:15px;font-weight:400}nav{position:sticky;top:0;background:#101b22ef;backdrop-filter:blur(10px);z-index:2;border-block:1px solid #33484d;padding:15px;display:flex;gap:26px;justify-content:center}a{color:#a8e4d8}article{margin:30px 0 55px;padding:28px;border:1px solid #37505a;border-radius:18px;background:#15232b}img{max-width:100%;border-radius:10px}.intro{display:grid;grid-template-columns:280px 1fr;gap:30px;align-items:center}.portrait{width:280px}.eyebrow{text-transform:uppercase;letter-spacing:.15em;color:#ddb879;font-size:12px}.forms{width:100%;margin-top:24px}.caption{font-size:13px;color:#a4b9bd}ul{display:grid;grid-template-columns:1fr 1fr;list-style:none;padding:0;gap:12px}details{border-top:1px solid #37505a;padding-top:16px}summary{cursor:pointer;color:#a8e4d8}details img{margin-top:18px}footer{font-size:14px;color:#a4b9bd}.lead{max-width:800px;color:#b9cac9}.pill{display:inline-block;padding:4px 14px;border:1px solid #53766e;border-radius:30px;color:#b5e5d9;font-size:14px}section{scroll-margin-top:80px}@media(max-width:900px){.intro{grid-template-columns:220px 1fr}.portrait{width:220px}}'''
    html = '<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>TBG · Ogros e Abissais</title><style>'+style+'</style></head><body><header><p class="eyebrow">Tupi Battle Grounds · revisão visual local</p><h1>Ogros mais expressivos.<br>Abissais em campo.</h1><p class="lead">19 campeões em revisão: nove Ogros refinados, nove Abissais com o Tridente e o Estala-Vagens para as Plantas. Modelos reais e evoluções ao lado das referências visuais.</p><span class="pill">81 campeões · 9 facções · prévia Linux</span></header><nav><a href="#ogres">Ogros</a><a href="#abyssals">Abissais</a><a href="#plants">Plantas</a><a href="http://localhost:8604/#champions">Landing page</a></nav><main>'+''.join(cards)+'</main><footer>Referência de apresentação: Linux. Comparação visual com Windows e navegador desktop do jogo ainda pendente. Conceitos de IA e modelos reais possuem origens registradas separadamente; esta galeria não representa ativação do elenco competitivo.</footer></body></html>'
    (out/'index.html').write_text(html)
    print(f'Review gallery: {len(OGRES)+len(NEW)+2} champions, {len(images)} verified captures')


if __name__ == '__main__':
    main()
