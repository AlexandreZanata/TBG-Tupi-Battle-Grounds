// Lobby Arenas page (ARENAS-G). The choice is the Practice arena, kept in
// localStorage (`storageKey`); online matches always use the server's draw.
// Choosing previews the arena behind the lobby; editable only while idle.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  const panel = document.getElementById('arena-hub');
  const arena = globalThis.TbgArena;
  if (!root || !panel || !arena) return;
  const t = (key, args) => globalThis.TBG_I18N ? globalThis.TBG_I18N.t(key, args) : key;
  const get = name => panel.querySelector('[data-ar="' + name + '"]');
  const locked = () => root.hidden || root.dataset.screen !== 'lobby';
  let data, chosen = 'forest', opener;
  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }
  function read() {
    chosen = data.defaultArena;
    try {
      const raw = (localStorage.getItem(data.storageKey) || '').trim();
      if (arena.find(data, raw)) chosen = raw;
    } catch (_) { /* storage blocked: the default stays */ }
  }
  function save(slug) {
    chosen = slug;
    try {
      localStorage.setItem(data.storageKey, slug);
      get('status').textContent = t('arenas.saved', { name: arena.name(slug) });
    } catch (_) { get('status').textContent = t('arenas.storage_off'); }
  }
  function choose(slug) {
    if (locked() || !arena.find(data, slug)) return;
    save(slug);
    root.dataset.action = 'arena:' + slug;
    render();
  }
  function train(slug) {
    if (locked() || !arena.find(data, slug)) return;
    save(slug);
    panel.close();
    // Practice reads the saved arena; its MatchStart switches the scene.
    root.dataset.action = 'practice';
  }
  function card(def) {
    const item = element('article', 'arena-card');
    item.dataset.slug = def.slug;
    item.style.setProperty('--arena', def.accent);
    const art = element('div', 'arena-art');
    const image = element('img');
    image.alt = ''; image.decoding = 'async'; image.loading = 'lazy';
    image.src = 'assets/arenas/' + def.image;
    art.append(image, element('span', 'arena-flag'));
    const body = element('div', 'arena-body');
    body.append(element('h2'), element('p', 'arena-theme'), element('p', 'arena-effect'), element('small', 'arena-fair'));
    const actions = element('div', 'arena-actions');
    const pick = element('button', 'arena-pick');
    pick.type = 'button';
    pick.addEventListener('click', () => choose(def.slug));
    const go = element('button', 'arena-train');
    go.type = 'button';
    go.addEventListener('click', () => train(def.slug));
    actions.append(pick, go);
    item.append(art, body, actions);
    return item;
  }
  function render() {
    if (!data) return;
    panel.dataset.locked = String(locked());
    panel.querySelectorAll('.arena-card').forEach(item => {
      const def = arena.find(data, item.dataset.slug);
      const mine = def.slug === chosen;
      item.dataset.chosen = String(mine);
      item.querySelector('h2').textContent = arena.name(def.slug);
      item.querySelector('.arena-theme').textContent = arena.theme(def.slug);
      item.querySelector('.arena-effect').textContent = arena.effect(def, data);
      item.querySelector('.arena-fair').textContent = t('arenas.symmetric');
      item.querySelector('.arena-flag').textContent = t('arenas.chosen');
      const pick = item.querySelector('.arena-pick');
      pick.textContent = mine ? t('arenas.chosen') : t('arenas.choose');
      pick.disabled = mine || locked();
      pick.setAttribute('aria-pressed', String(mine));
      const go = item.querySelector('.arena-train');
      go.textContent = t('arenas.train');
      go.disabled = locked();
    });
  }
  async function open(button) {
    if (locked()) return;
    opener = button;
    if (!panel.open) panel.showModal();
    get('error').hidden = true;
    try {
      data ||= await arena.load();
      if (!get('grid').childElementCount) data.arenas.forEach(def => get('grid').append(card(def)));
      read();
      get('status').textContent = '';
      render();
      panel.querySelector('.arena-close').focus();
    } catch (error) { get('error').textContent = error.message || t('arenas.error'); get('error').hidden = false; }
  }
  root.querySelectorAll('[data-arena-open]').forEach(button => button.addEventListener('click', () => open(button)));
  panel.querySelector('.arena-close').addEventListener('click', () => panel.close());
  panel.addEventListener('close', () => { if (!root.hidden) opener?.focus(); });
  document.addEventListener('tbg:lang', render);
  new MutationObserver(render).observe(root, { attributes: true, attributeFilter: ['data-screen', 'hidden'] });
})();
