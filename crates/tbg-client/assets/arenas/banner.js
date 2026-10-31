// In-match arena presentation (ARENAS-G): a start banner with the arena and
// its rule, and a small chip in the score header that shows it again.
(() => {
  'use strict';
  const root = document.getElementById('tbg-battle');
  const arena = globalThis.TbgArena;
  if (!root || !arena) return;
  const banner = root.querySelector('[data-arena-banner]');
  const chip = root.querySelector('[data-arena-chip]');
  if (!banner || !chip) return;
  const t = (key, args) => globalThis.TBG_I18N ? globalThis.TBG_I18N.t(key, args) : key;
  let data, shown = '', timer;
  function fill() {
    const def = data && arena.find(data, root.dataset.arena);
    if (!def) return false;
    banner.style.setProperty('--arena', def.accent);
    chip.style.setProperty('--arena', def.accent);
    banner.querySelector('[data-arena-name]').textContent = arena.name(def.slug);
    banner.querySelector('[data-arena-effect]').textContent = arena.effect(def, data);
    banner.querySelector('[data-arena-fair]').textContent = t('arenas.symmetric');
    banner.querySelector('[data-arena-label]').textContent = t('arenas.banner.label');
    chip.querySelector('b').textContent = arena.name(def.slug);
    chip.title = arena.effect(def, data);
    chip.hidden = false;
    return true;
  }
  function show(ms) {
    if (!fill()) return;
    banner.hidden = false;
    banner.dataset.visible = 'true';
    clearTimeout(timer);
    timer = setTimeout(() => { banner.dataset.visible = 'false'; }, ms);
  }
  // A new match (or a reconnect after a reload) announces its arena once.
  function update() {
    if (root.hidden) { shown = ''; return; }
    const slug = root.dataset.arena || '';
    if (!slug || slug === shown) { fill(); return; }
    shown = slug;
    arena.load().then(loaded => { data = loaded; show(4500); }).catch(() => {});
  }
  chip.addEventListener('click', () => show(3500));
  document.addEventListener('tbg:lang', fill);
  new MutationObserver(update).observe(root, { attributes: true, attributeFilter: ['data-arena', 'hidden'] });
})();
