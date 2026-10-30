// Stronghold picker (D-46). The chosen kind's name lives in localStorage
// (`storageKey`); the game sends it before every match request and the
// server validates it. Like the Arcanas, the choice is editable only while
// the lobby is idle: queueing, a room, Practice or a match hide the lobby.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  const panel = document.getElementById('stronghold-picker');
  if (!root || !panel) return;
  const tower = globalThis.TbgStronghold;
  const get = name => panel.querySelector('[data-s="' + name + '"]');
  const text = (name, value) => { get(name).textContent = value; };
  let data, loading, chosen = '', storageNote = '', opener, cards = [];
  function element(tag, className, content) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (content !== undefined) node.textContent = content;
    return node;
  }
  function artImage(asset, className, alt) {
    const image = element('img', className);
    image.dataset.towerArt = asset.display;
    image.alt = alt; image.decoding = 'async'; image.loading = 'lazy';
    return image;
  }
  function loadArt() {
    // The picker data also prepares the profile chip while idle; pictures wait for open.
    if (!panel.open) return;
    panel.querySelectorAll('[data-tower-art]').forEach(image => {
      if (!image.hasAttribute('src')) image.src = 'assets/stronghold/' + image.dataset.towerArt;
    });
  }
  const locked = () => root.hidden || root.dataset.screen !== 'lobby';
  function load() {
    chosen = data.defaultKind;
    try {
      const raw = localStorage.getItem(data.storageKey);
      if (raw === null) { storageNote = 'Default tower · pick one to keep it'; return; }
      if (!tower.kind(data, raw.trim())) throw new Error('invalid');
      chosen = raw.trim(); storageNote = 'Saved in this browser · accounts keep it on every device';
    } catch (_) { storageNote = 'Saved tower unavailable or invalid · default restored'; }
  }
  function choose(name) {
    if (locked() || !tower.kind(data, name)) return;
    chosen = name;
    try { localStorage.setItem(data.storageKey, name); storageNote = 'Saved · your next match uses ' + name; } catch (_) {
      storageNote = 'Browser storage unavailable · this choice lasts for this visit only';
    }
    render();
  }
  function statList(s) {
    const list = element('dl', 'tower-stats');
    for (const [label, value] of [['HP', s.maxHp], ['Damage', s.damage], ['Attack every', tower.seconds(s.interval, data)], ['Range', tower.tiles(s.range)]]) {
      const row = element('div'); row.append(element('dt', '', label), element('dd', '', String(value))); list.append(row);
    }
    return list;
  }
  function bullets(className, title, lines, def) {
    const box = element('div', className);
    const list = element('ul');
    lines.forEach(line => list.append(element('li', '', tower.fill(line, def.stats, data))));
    box.append(element('h4', '', title), list);
    return box;
  }
  function branchBlock(def, branch) {
    const block = element('section', 'tower-branch');
    const heading = element('h4');
    heading.append(artImage(def.branches[branch].asset, 'tower-branch-art', ''), element('span', '', (branch ? 'B · ' : 'A · ') + def.branches[branch].name));
    block.append(heading);
    const steps = element('ol');
    def.branches[branch].steps.forEach((step, i) => {
      const item = element('li');
      const head = element('span', 'tower-step');
      head.append(element('b', '', 'Tier ' + (i + 1) + ' · ' + step.name), element('span', 'tower-cost', tower.cost(data, i + 1) + ' Energy'));
      item.append(head, element('span', 'tower-effect', tower.effect(data, def, branch, i + 1)));
      steps.append(item);
    });
    block.append(steps);
    return block;
  }
  function card(def) {
    const article = element('article', 'tower-card');
    article.dataset.towerKind = def.kind; article.style.setProperty('--tower', def.color);
    article.setAttribute('aria-label', def.name + ' Stronghold');
    const art = element('div', 'tower-art'); art.append(artImage(def.banner, 'tower-banner', ''), artImage(def.portrait, 'tower-drawing tower-portrait', def.name + ' Stronghold'), element('span', 'tower-flag', 'Selected'));
    const head = element('header', 'tower-head');
    head.append(element('h2', '', def.name), element('p', 'tower-specialty', def.specialty), element('p', 'tower-summary', def.summary));
    const passive = element('section', 'tower-passive');
    passive.append(element('h3', '', 'Passive · ' + def.passive.name), element('p', '', tower.fill(def.passive.text, def.stats, data)));
    const traits = element('div', 'tower-traits');
    traits.append(bullets('tower-pros', 'Advantages', def.advantages, def), bullets('tower-cons', 'Disadvantages', def.disadvantages, def));
    const fort = element('section', 'tower-fortify');
    fort.append(element('h3', '', 'Fortification · choose one branch'), branchBlock(def, 0), branchBlock(def, 1));
    const select = element('button', 'tower-select');
    select.type = 'button';
    select.addEventListener('click', () => choose(def.kind));
    article.addEventListener('click', event => { if (!event.target.closest('button')) choose(def.kind); });
    article.append(art, head, statList(def.stats), passive, traits, fort, select);
    cards.push({def, article, select});
    return article;
  }
  function render() {
    const isLocked = locked();
    panel.dataset.locked = String(isLocked);
    for (const {def, article, select} of cards) {
      const on = def.kind === chosen;
      article.dataset.chosen = String(on);
      select.textContent = on ? 'Selected' : 'Select ' + def.name;
      select.setAttribute('aria-pressed', String(on));
      select.disabled = isLocked;
    }
    const current = data ? tower.kind(data, chosen) : undefined;
    text('status', isLocked ? data.copy.locked : current ? 'Your tower: ' + current.name + ' · ' + current.specialty : '');
    text('storage', storageNote);
    chip();
  }
  // The profile panel shows the current tower at every visit to the lobby.
  function chip() {
    if (!data) return;
    const def = tower.kind(data, chosen) || tower.kind(data, data.defaultKind);
    root.querySelectorAll('[data-tower-name]').forEach(node => { if (node.textContent !== def.name) node.textContent = def.name; });
    root.querySelectorAll('[data-tower-icon]').forEach(node => node.setAttribute('href', tower.icon(def.kind)));
    root.querySelectorAll('[data-tower-chip]').forEach(node => {
      node.style.setProperty('--tower', def.color);
      node.setAttribute('aria-label', 'Stronghold: ' + def.name + '. Change tower');
    });
  }
  async function prepare() {
    const response = await fetch('assets/stronghold/strongholds.json');
    if (!response.ok) throw new Error('Stronghold data could not load. Close and reopen to retry.');
    data = await response.json();
    tower.sprite();
    data.kinds.forEach(def => get('grid').append(card(def)));
    ['intro', 'scope', 'fortify', 'controls', 'badge'].forEach(key => text(key, data.copy[key]));
  }
  const ready = () => (loading ||= prepare().catch(error => { loading = undefined; throw error; }));
  root.querySelectorAll('[data-tower-open]').forEach(button => button.addEventListener('click', async () => {
    if (locked()) return;
    opener = button;
    if (!panel.open) panel.showModal();
    get('error').hidden = true;
    try {
      await ready(); load(); loadArt();
      if (panel.open) { render(); panel.querySelector('.tower-close').focus(); }
    } catch (error) { text('error', error.message); get('error').hidden = false; }
  }));
  panel.addEventListener('close', () => { if (!root.hidden) opener?.focus(); });
  // The server may answer with an account's saved kind while the lobby is
  // hidden; re-read storage whenever the lobby shows again.
  new MutationObserver(() => {
    if (root.dataset.screen !== 'lobby') { if (data) render(); return; }
    ready().then(() => { load(); render(); }).catch(() => {});
  }).observe(root, {attributes: true, attributeFilter: ['data-screen', 'hidden']});
})();
