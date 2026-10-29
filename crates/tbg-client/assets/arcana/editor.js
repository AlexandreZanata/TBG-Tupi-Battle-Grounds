// Browser-only Arcana draft editor. No actions cross the Rust/network bridge.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  const panel = document.getElementById('arcana-editor');
  if (!root || !panel) return;
  const rules = globalThis.TbgArcanaPreset;
  const get = name => panel.querySelector('[data-a="' + name + '"]');
  const text = (name, value) => { get(name).textContent = value; };
  const desktop = matchMedia('(min-width:1280px) and (min-height:720px) and (orientation:landscape)');
  let data, loading, ranks, opener, storageNote = '', cards = [], branches = [];
  function element(tag, className, content) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (content !== undefined) node.textContent = content;
    return node;
  }
  function art(asset, className, size, alt) {
    const image = element('img', className);
    image.width = size; image.alt = alt; image.decoding = 'async'; image.loading = 'lazy';
    image.src = 'assets/arcana/' + asset.display;
    return image;
  }
  function control(label, content, onClick) {
    const button = element('button', '', content);
    button.type = 'button'; button.setAttribute('aria-label', label);
    button.addEventListener('click', onClick);
    return button;
  }
  function save() {
    try {
      localStorage.setItem(data.storageKey, JSON.stringify({schema:data.previewSchema, ranks}));
      storageNote = 'Saved locally · this device only';
    } catch (_) { storageNote = 'Browser storage unavailable · draft kept for this visit'; }
  }
  function load() {
    ranks = [...data.defaultPreset];
    try {
      const raw = localStorage.getItem(data.storageKey);
      if (raw === null) { storageNote = 'Default selected · edits save on this device'; return; }
      if (raw.length > 1024) throw new Error('oversized');
      const stored = JSON.parse(raw);
      if (stored?.schema !== data.previewSchema || !rules.editable(stored.ranks, data)) throw new Error('invalid');
      ranks = [...stored.ranks]; storageNote = 'Restored from this browser';
    } catch (_) { storageNote = 'Saved preview unavailable or invalid · default restored for this visit'; }
  }
  function change(index, delta) {
    const next = [...ranks]; next[index] += delta;
    if (!rules.editable(next, data)) return;
    ranks = next; save(); render();
  }
  function nodeCard(node) {
    const card = element('article', 'arcana-node');
    card.dataset.node = node.id;
    const add = control('Add a rank to ' + node.name, undefined, () => change(node.index, 1));
    add.className = 'arcana-node-main';
    add.append(art(node.asset, 'arcana-node-icon', 72, ''));
    const copy = element('span', 'arcana-node-copy');
    copy.append(element('strong', '', node.name), element('span', 'arcana-per-rank', node.perRankText + ' per rank'), element('span', 'arcana-max', 'Rank 2: ' + node.rankTwoText));
    const pips = element('span', 'arcana-pips');
    for (let i = 0; i < data.maxRank; i++) pips.append(element('i'));
    copy.append(pips); add.append(copy);
    card.addEventListener('contextmenu', event => { event.preventDefault(); change(node.index, -1); });
    card.addEventListener('click', event => { if (!event.target.closest('button')) change(node.index, 1); });
    add.addEventListener('keydown', event => {
      if (['+', '=', '-', '_'].includes(event.key)) {
        event.preventDefault(); change(node.index, event.key === '-' || event.key === '_' ? -1 : 1);
      }
    });
    const units = data.roster.filter(unit => node.archetypes.includes(unit.archetype) && (!node.requiresExistingHeal || unit.existingHeal));
    const affected = element('p', 'arcana-affected');
    affected.append(element('b', '', node.eligibility), document.createTextNode(' · ' + units.map(unit => unit.name).join(', ')));
    const toolbar = element('div', 'arcana-node-tools');
    const minus = control('Remove a rank from ' + node.name, '−', () => change(node.index, -1));
    const rank = element('output'); rank.setAttribute('aria-label', node.name + ' rank');
    const plus = control('Add a rank to ' + node.name, '+', () => change(node.index, 1));
    toolbar.append(minus, rank, plus); card.append(add, affected, toolbar);
    cards.push({node, card, add, minus, plus, rank, pips}); return card;
  }
  function familyCard(family) {
    const section = element('section', 'arcana-family');
    section.dataset.family = family.id; section.style.setProperty('--family', family.color);
    section.setAttribute('aria-label', family.name + ' family');
    const header = element('header', 'arcana-family-header');
    const image = art(family.banner, 'arcana-banner', 900, ''); image.height = 360;
    const emblem = art(family.emblem, 'arcana-emblem', 160, family.name + ' emblem'); emblem.height = 160;
    header.append(image, emblem, element('h2', '', family.name), element('p', '', family.description));
    const nodes = element('div', 'arcana-nodes');
    data.nodes.filter(node => node.family === family.id).forEach(node => nodes.append(nodeCard(node)));
    section.append(header, nodes); get('families').append(section);
    branches.push({family, section, header, nodes});
  }
  function render() {
    const left = data.pointBudget - rules.spent(ranks);
    text('points', 'Points left ' + left + ' / ' + data.pointBudget);
    text('status', rules.complete(ranks, data) ? data.copy.complete : data.copy.partial.replace('{budget}', data.pointBudget));
    get('status').dataset.complete = String(rules.complete(ranks, data));
    text('storage', storageNote);
    for (const {node, card, add, minus, plus, rank, pips} of cards) {
      const value = ranks[node.index]; card.dataset.rank = value;
      rank.textContent = value + ' / ' + data.maxRank;
      minus.disabled = value === 0;
      plus.disabled = left === 0 || value === data.maxRank;
      // Keep the main card focusable at the cap so minus/right-click remain usable.
      add.dataset.addBlocked = String(plus.disabled);
      add.setAttribute('aria-label', node.name + ', rank ' + value + ' of ' + data.maxRank + '. ' + node.perRankText + ' per rank. Add with Enter or plus; remove with minus.');
      [...pips.children].forEach((pip, index) => { pip.dataset.filled = String(index < value); });
    }
    requestAnimationFrame(connect);
  }
  function connect() {
    if (!panel.open) return;
    const map = panel.querySelector('.arcana-map'), bounds = map.getBoundingClientRect();
    const anchor = (node, edge) => {
      const r = node.getBoundingClientRect();
      return {x:r.left + r.width / 2 - bounds.left, y:(edge === 'top' ? r.top : r.bottom) - bounds.top};
    };
    const lines = get('lines'); lines.replaceChildren();
    const path = (a, b, color, active) => {
      const line = document.createElementNS('http://www.w3.org/2000/svg', 'path');
      const mid = (a.y + b.y) / 2;
      line.setAttribute('d', `M${a.x},${a.y} C${a.x},${mid} ${b.x},${mid} ${b.x},${b.y}`);
      line.style.setProperty('--family', color); line.dataset.active = String(active); lines.append(line);
    };
    const center = anchor(get('center'), 'bottom');
    for (const branch of branches) {
      const nodes = data.nodes.filter(node => node.family === branch.family.id);
      const invested = nodes.some(node => ranks[node.index] > 0);
      branch.section.dataset.active = String(invested);
      path(center, anchor(branch.header, 'top'), branch.family.color, invested);
      for (const node of nodes) {
        path(anchor(branch.header, 'bottom'), anchor(cards[node.index].card, 'top'), branch.family.color, ranks[node.index] > 0);
      }
    }
  }
  async function prepare() {
    const response = await fetch('assets/arcana/arcana.json');
    if (!response.ok) throw new Error('Arcana preview data could not load. Close and reopen to retry.');
    data = await response.json(); load();
    data.families.forEach(familyCard);
    ['intro','scope','controls','eligibility'].forEach(key => text(key, data.copy[key]));
    document.getElementById('arcana-badge').textContent = data.copy.badge;
    text('summon', data.summon.text);
    text('default', 'Default [' + data.defaultPreset.join(',') + ']');
  }
  root.querySelectorAll('[data-arcana-open]').forEach(button => button.addEventListener('click', async () => {
    if (!desktop.matches || root.hidden || root.dataset.screen !== 'lobby') return;
    opener = button;
    if (!panel.open) panel.showModal();
    text('status', 'Loading Arcana preview…');
    get('error').hidden = true;
    try {
      loading ||= prepare().catch(error => { loading = undefined; throw error; });
      await loading;
      if (panel.open) { render(); panel.querySelector('.arcana-close').focus(); }
    } catch (error) { text('error', error.message); get('error').hidden = false; text('status', 'Preview unavailable'); }
  }));
  get('default').addEventListener('click', () => { if (data) { ranks = [...data.defaultPreset]; save(); render(); } });
  get('reset').addEventListener('click', () => { if (data) { ranks = data.nodes.map(() => 0); save(); render(); } });
  panel.addEventListener('close', () => { if (!root.hidden) opener?.focus(); });
  desktop.addEventListener('change', () => { if (!desktop.matches) panel.close(); });
  new ResizeObserver(() => requestAnimationFrame(connect)).observe(panel);
})();
