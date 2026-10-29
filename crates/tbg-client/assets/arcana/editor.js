// Arcana editor (D-40/P40). The preset lives in localStorage (`storageKey`);
// the game reads it when a match starts and the server validates it. The
// same three family banners and node art are reused; children and supremes
// add their own frames and connection lines.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  const panel = document.getElementById('arcana-editor');
  if (!root || !panel) return;
  const rules = globalThis.TbgArcanaPreset;
  const get = name => panel.querySelector('[data-a="' + name + '"]');
  const text = (name, value) => { get(name).textContent = value; };
  let data, loading, ranks, opener, storageNote = '', notice = '', cards = [], branches = [];
  function element(tag, className, content) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (content !== undefined) node.textContent = content;
    return node;
  }
  function art(asset, className, size, alt) {
    const image = element('img', className);
    image.width = size; image.height = size; image.alt = alt; image.decoding = 'async'; image.loading = 'lazy';
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
      storageNote = 'Saved · used by your next match (accounts keep it on every device)';
    } catch (_) { storageNote = 'Browser storage unavailable · draft kept for this visit'; }
  }
  function load() {
    ranks = [...data.defaultPreset];
    try {
      const raw = localStorage.getItem(data.storageKey);
      if (raw === null) { storageNote = 'Default selected · your edits are saved'; return; }
      if (raw.length > 4096) throw new Error('oversized');
      const stored = JSON.parse(raw);
      if (!Array.isArray(stored?.ranks)) throw new Error('invalid');
      if (stored.ranks.length === 6) {
        ranks = rules.migrate(stored.ranks, data); save();
        storageNote = 'Saved six-node preset migrated to twelve points';
        return;
      }
      if (!rules.editable(stored.ranks, data)) throw new Error('invalid');
      ranks = [...stored.ranks]; storageNote = 'Restored from this browser';
    } catch (_) { storageNote = 'Saved preset unavailable or invalid · default restored'; }
  }
  function change(index, delta) {
    const node = data.nodes[index];
    const value = ranks[index];
    if (delta > 0) {
      if (value >= node.maxRank) notice = node.name + ' is at its maximum rank.';
      else if (rules.spent(ranks, data) + node.cost > data.pointBudget) notice = 'Not enough points for ' + node.name + '.';
      else if (node.requires.some(([i, rank]) => ranks[i] < rank)) {
        notice = 'Requires ' + rules.missing(ranks, data, node).join(', ') + '.';
      } else {
        notice = ''; ranks = [...ranks]; ranks[index] += 1; save();
      }
    } else if (value > 0) {
      const blocked = rules.dependents(ranks, data, index, value - 1);
      if (blocked.length) notice = 'Remove ' + blocked.join(', ') + ' first.';
      else { notice = ''; ranks = [...ranks]; ranks[index] -= 1; save(); }
    }
    render();
  }
  function requirement(node) {
    if (!node.requires.length) return 'No prerequisite';
    const missing = rules.missing(ranks, data, node);
    return missing.length ? 'Requires ' + missing.join(', ') : 'Requirements met';
  }
  function nodeCard(node) {
    const card = element('article', 'arcana-node arcana-node-' + node.kind);
    card.dataset.node = node.id; card.dataset.kind = node.kind;
    const cost = element('span', 'arcana-cost', node.cost + (node.cost > 1 ? ' points' : ' point'));
    const add = control('Add a rank to ' + node.name, undefined, () => change(node.index, 1));
    add.className = 'arcana-node-main';
    add.append(art(node.asset, 'arcana-node-icon', 72, ''));
    const copy = element('span', 'arcana-node-copy');
    copy.append(element('strong', '', node.name));
    copy.append(element('span', 'arcana-per-rank', node.perRankText));
    copy.append(element('span', 'arcana-effect', node.effect));
    const pips = element('span', 'arcana-pips');
    for (let i = 0; i < node.maxRank; i++) pips.append(element('i'));
    copy.append(pips); add.append(copy);
    card.addEventListener('contextmenu', event => { event.preventDefault(); change(node.index, -1); });
    card.addEventListener('click', event => { if (!event.target.closest('button')) change(node.index, 1); });
    add.addEventListener('keydown', event => {
      if (['+', '=', '-', '_'].includes(event.key)) {
        event.preventDefault(); change(node.index, event.key === '-' || event.key === '_' ? -1 : 1);
      }
    });
    const affected = element('p', 'arcana-affected');
    const units = data.roster.filter(unit => node.archetypes.includes(unit.archetype) && (!node.requiresExistingHeal || unit.existingHeal));
    affected.append(element('b', '', node.eligibility), document.createTextNode(' · ' + units.map(unit => unit.name).join(', ')));
    const requires = element('p', 'arcana-requires');
    const toolbar = element('div', 'arcana-node-tools');
    const minus = control('Remove a rank from ' + node.name, '−', () => change(node.index, -1));
    const rank = element('output'); rank.setAttribute('aria-label', node.name + ' rank');
    const plus = control('Add a rank to ' + node.name, '+', () => change(node.index, 1));
    toolbar.append(minus, rank, plus);
    card.append(cost, add, affected, requires, toolbar);
    cards[node.index] = {node, card, add, minus, plus, rank, pips, requires};
    return card;
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
    const familyNodes = data.nodes.filter(node => node.family === family.id);
    const row = kind => {
      const line = element('div', 'arcana-node-row arcana-' + kind + '-row');
      familyNodes.filter(node => node.kind === kind).forEach(node => line.append(nodeCard(node)));
      return line;
    };
    nodes.append(row('original'), row('child'), row('supreme'));
    section.append(header, nodes); get('families').append(section);
    branches.push({family, section, header, nodes, cards: familyNodes.map(node => node.index)});
  }
  function render() {
    const spent = rules.spent(ranks, data);
    text('points', 'Points left ' + (data.pointBudget - spent) + ' / ' + data.pointBudget);
    text('status', notice || (rules.complete(ranks, data) ? data.copy.complete : data.copy.partial.replace('{budget}', data.pointBudget)));
    get('status').dataset.complete = String(!notice && rules.complete(ranks, data));
    text('storage', storageNote);
    for (const {node, card, add, minus, plus, rank, pips, requires} of cards) {
      const value = ranks[node.index];
      const addable = rules.canAdd(ranks, data, node.index);
      const locked = value === 0 && node.requires.some(([i, r]) => ranks[i] < r);
      card.dataset.rank = value; card.dataset.locked = String(locked);
      rank.textContent = value + ' / ' + node.maxRank;
      minus.disabled = value === 0;
      plus.disabled = !addable;
      add.dataset.addBlocked = String(!addable);
      add.setAttribute('aria-label', node.name + ', rank ' + value + ' of ' + node.maxRank + '. Cost ' + node.cost + ' points. ' + requirement(node) + '. Add with Enter or plus; remove with minus.');
      [...pips.children].forEach((pip, index) => { pip.dataset.filled = String(index < value); });
      requires.textContent = requirement(node);
      requires.dataset.met = String(!locked);
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
      const invested = branch.cards.some(index => ranks[index] > 0);
      branch.section.dataset.active = String(invested);
      path(center, anchor(branch.header, 'top'), branch.family.color, invested);
      const originals = branch.cards.filter(index => data.nodes[index].kind === 'original');
      for (const index of originals) {
        path(anchor(branch.header, 'bottom'), anchor(cards[index].card, 'top'), branch.family.color, ranks[index] > 0);
      }
      for (const index of branch.cards) {
        const node = data.nodes[index];
        if (node.kind === 'child') {
          for (const [parent] of node.requires) {
            path(anchor(cards[parent].card, 'bottom'), anchor(cards[index].card, 'top'), branch.family.color, ranks[index] > 0);
          }
        }
        if (node.kind === 'supreme') {
          for (const [parent] of node.requires.filter(([, rank]) => rank === 1)) {
            path(anchor(cards[parent].card, 'bottom'), anchor(cards[index].card, 'top'), branch.family.color, ranks[index] > 0);
          }
        }
      }
    }
  }
  async function prepare() {
    const response = await fetch('assets/arcana/arcana.json');
    if (!response.ok) throw new Error('Arcana data could not load. Close and reopen to retry.');
    data = await response.json(); load();
    data.families.forEach(familyCard);
    ['intro','scope','controls','eligibility'].forEach(key => text(key, data.copy[key]));
    document.getElementById('arcana-badge').textContent = data.copy.badge;
    text('summon', data.summon.text);
    text('default', 'Restore default');
  }
  root.querySelectorAll('[data-arcana-open]').forEach(button => button.addEventListener('click', async () => {
    if (root.hidden || root.dataset.screen !== 'lobby') return;
    opener = button;
    if (!panel.open) panel.showModal();
    text('status', 'Loading Arcanas…');
    get('error').hidden = true;
    try {
      loading ||= prepare().catch(error => { loading = undefined; throw error; });
      await loading;
      load();
      if (panel.open) { render(); panel.querySelector('.arcana-close').focus(); }
    } catch (error) { text('error', error.message); get('error').hidden = false; text('status', 'Arcanas unavailable'); }
  }));
  get('default').addEventListener('click', () => { if (data) { notice = ''; ranks = [...data.defaultPreset]; save(); render(); } });
  get('reset').addEventListener('click', () => { if (data) { notice = ''; ranks = data.nodes.map(() => 0); save(); render(); } });
  panel.addEventListener('close', () => { if (!root.hidden) opener?.focus(); });
  new ResizeObserver(() => requestAnimationFrame(connect)).observe(panel);
})();
