#!/usr/bin/env python3
"""Publish the local owner review with real engine captures and clearly separate concept references."""
import argparse
from html import escape
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--art-root', type=Path, required=True)
    parser.add_argument('--engine', default='engine-ring-only')
    args = parser.parse_args()
    art = args.art_root.resolve()
    catalog = json.loads((ROOT / 'docs/champion-completion/catalog.json').read_text())
    proposal = json.loads((art / 'proposal.json').read_text())
    ideas = {c['id']: c for c in proposal['champions']}
    manifest = json.loads((art / args.engine / 'capture-manifest.json').read_text())
    records = {(c['kind'], c['mode']): c for c in manifest['captures']}
    labels = {'Undead': 'Mortos-vivos', 'Monsters': 'Monstros', 'Celestials': 'Celestiais', 'Insects': 'Insetos'}
    cards = []
    nav = []
    for c in catalog['champions']:
        name = escape(c['name']['pt'])
        slug = c['id']
        for mode in ['base portrait', 'lineup: idle/walk/attack']:
            record = records[(c['kind'], mode)]
            if not (art / args.engine / record['image']).is_file():
                raise SystemExit(f'Missing actual capture: {c["kind"]} {mode}')
            log = (art / args.engine / record['log']).read_text()
            if 'boot_ready=true, scene_waiting=false, pending_images=0, pending_texts=0, failed_images=0' not in log:
                raise SystemExit(f'Refusing loading-screen capture: {c["kind"]} {mode}')
        portrait = f'{args.engine}/{slug}-portrait.png'
        lineup = f'{args.engine}/{slug}-lineup.png'
        reference = f'evolutions/{slug}.png'
        if slug == 'gorgehowl':
            reference = 'evolutions/gorgehowl-v2.png'
        forms = [{'code': 'BASE', **c['default_skin']}, *c['forms']]
        form_html = ''.join(f'<li><strong>{f["code"]}</strong> {escape(f["pt"])} <small>{escape(f["en"])}</small></li>' for f in forms)
        idea = ideas[slug]
        technical = ''.join(f'<h4>{title}</h4><p>{escape(c[key])}</p>' for key, title in [('shape','Silhueta'),('weapon','Arma'),('projectile','Projétil / impacto'),('idle','Idle'),('walk','Movimento'),('attack','Ataque'),('special','Especial'),('victory','Vitória'),('death','Morte'),('space','Espaço na formação')])
        nav.append(f'<a href="#{slug}">{name}</a>')
        cards.append(f'''<article id="{slug}" data-faction="{c['faction']}"><header><img class="portrait" src="{portrait}" loading="lazy" alt="{name}, modelo instalado sem placa de equipe"><div><p class="eyebrow">{c['code']} · {labels[c['faction']]} · {c['archetype']}</p><h2>{name}</h2><p class="muted">{escape(c['name']['en'])}</p><p>{escape(idea['ability'])} — {escape(idea['kit'])}</p></div></header><a href="{lineup}" target="_blank" rel="noopener"><img class="lineup" width="1600" height="1000" src="{lineup}" loading="lazy" alt="{name}: cinco formas capturadas no motor, nas poses parado, caminhada e ataque"></a><div class="copy"><p class="caption">Colunas: Base · A1 · A2 · B1 · B2. Linhas: parado · caminhada · ataque. Captura real do motor Linux.</p><ul class="forms">{form_html}</ul><details><summary>Arma, habilidades, movimentos, morte e espaço</summary>{technical}<p>Raio cosmético base: {c['crowd_radius']:.2f}; +8% por nível de Evolution. Não altera alcance nem colisão do servidor. Vitória: 3 segundos.</p></details><details><summary>Referência de conceito · imagem ilustrativa</summary><p class="muted">Direção visual inicial; o modelo real instalado aparece acima.</p><a href="{reference}"><img class="concept" src="{reference}" loading="lazy" alt="Conceito de cinco formas de {name}"></a></details></div></article>''')
    css = '''*{box-sizing:border-box}html{color-scheme:dark}body{margin:0;background:#10171f;color:#eee8dd;font:16px/1.55 system-ui,sans-serif}main{max-width:1500px;margin:auto;padding:38px 30px 80px}h1{font-size:38px;line-height:1.12;margin:12px 0 20px}h2{font-size:26px;margin:4px 0}p{margin:0 0 12px}.eyebrow{color:#dfbf85;font-size:12px;text-transform:uppercase;letter-spacing:1px}.muted,.caption,small{color:#afbdc7}.intro{max-width:1000px}.notice{border:1px solid #344553;background:#19232d;padding:18px;margin:24px 0}.toolbar,nav{display:flex;gap:12px 22px;align-items:center;flex-wrap:wrap;margin:24px 0}nav{font-size:13px;border-block:1px solid #344553;padding:20px 0}a,summary{color:#dfbf85}select{font:inherit;background:#19232d;color:inherit;border:1px solid #56616a;padding:8px}article{background:#19232d;border:1px solid #344553;border-radius:8px;margin:36px 0;overflow:hidden;scroll-margin-top:20px}header{display:grid;grid-template-columns:210px 1fr;gap:28px;align-items:center;padding:22px}.portrait{width:210px;height:210px;object-fit:contain;background:#14202a;border-radius:6px}.lineup{display:block;width:100%;height:auto;background:#14202a}.copy{padding:20px 24px 26px}.caption{font-size:13px}.forms{display:flex;gap:15px;list-style:none;flex-wrap:wrap;margin:24px 0;padding:0}.forms li{flex:1;min-width:160px;border-left:2px solid #dfbf85;padding-left:12px}.forms strong,small{display:block}small{font-size:13px}details{border-top:1px solid #344553;padding:16px 0 0;margin:18px 0}summary{cursor:pointer}h4{margin:16px 0 6px;color:#dfbf85}.concept{display:block;max-width:900px;width:100%;margin:18px auto}article[hidden]{display:none}a:focus-visible,summary:focus-visible,select:focus-visible{outline:2px solid #dfbf85;outline-offset:4px}footer{font-size:13px;color:#afbdc7;margin-top:40px}'''
    options = ''.join(f'<option value="{key}">{value}</option>' for key, value in labels.items())
    html = f'''<!doctype html><html lang="pt-BR"><head><meta charset="utf-8"><title>TBG · 17 campeões no motor · sem placas</title><style>{css}</style></head><body><main><p class="eyebrow">Tupi Battle Grounds · revisão local Linux</p><h1>17 campeões · 85 formas.<br>Corpos sem placas de equipe.</h1><p class="intro">Retratos e evoluções capturados no jogo. O corpo usa a mesma aparência nos dois lados; aliado e inimigo são identificados pelos círculos azul e vermelho nos pés durante a batalha.</p><div class="notice">Prévia local de Practice e Training. Os novos campeões estão em proposta de balanceamento; o elenco online continua preservado. Windows e navegador desktop aguardam comparação visual e de movimento.</div><div class="toolbar"><label for="faction">Facção</label><select id="faction"><option value="All">Todas</option>{options}</select><span id="count" aria-live="polite">17 campeões · 85 formas</span><a href="TBG-17-low-poly-evolutions.zip">Baixar imagens e fichas</a></div><nav aria-label="Ir ao campeão">{''.join(nav)}</nav>{''.join(cards)}<footer>Capturas verificadas no Linux. <a href="{args.engine}/capture-manifest.json">Origem e versão de cada imagem</a>. Fotografias do motor e referências ilustrativas ficam identificadas separadamente. Nomes em português e inglês; fichas técnicas em inglês conforme o projeto.</footer></main><script>const filter=document.querySelector('#faction');const cards=[...document.querySelectorAll('article')];filter.addEventListener('change',()=>{{let n=0;for(const card of cards){{card.hidden=filter.value!=='All'&&card.dataset.faction!==filter.value;if(!card.hidden)n++;}}document.querySelector('#count').textContent=n+' campeões · '+n*5+' formas';}});</script></body></html>'''
    (art / 'production-gallery.html').write_text(html)
    (art / 'production-catalog.json').write_text(json.dumps(catalog, indent=2, ensure_ascii=False)+'\n')
    with zipfile.ZipFile(art / 'TBG-17-low-poly-evolutions.zip', 'w', zipfile.ZIP_DEFLATED) as bundle:
        for path in sorted((ROOT / 'docs/champion-completion').glob('*')):
            if path.is_file():
                bundle.write(path, 'briefs/'+path.name)
        for c in catalog['champions']:
            for suffix in ['portrait', 'lineup']:
                path = art / args.engine / f'{c["id"]}-{suffix}.png'
                bundle.write(path, 'engine/'+path.name)
            path = art / 'evolutions' / (f'{c["id"]}.png' if c['id'] != 'gorgehowl' else 'gorgehowl-v2.png')
            bundle.write(path, 'concepts/'+path.name)
        bundle.write(art / args.engine / 'capture-manifest.json', 'engine/capture-manifest.json')
    print(json.dumps({'gallery': str(art / 'production-gallery.html'), 'champions': 17, 'forms': 85, 'portrait_refresh': len([c for c in manifest['captures'] if c['mode'] == 'base portrait'])}))


if __name__ == '__main__':
    main()
