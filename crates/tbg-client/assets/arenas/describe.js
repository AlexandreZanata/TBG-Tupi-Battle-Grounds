// Arena text from arenas.json, shared by the lobby Arenas page, the battle
// banner and the Node tests. Every number comes from the data file, which a
// Rust test pins to tbg-core (GAME-DESIGN §12, D-55).
(() => {
  'use strict';
  const t = (key, args) => globalThis.TBG_I18N ? globalThis.TBG_I18N.t(key, args) : key;
  // Locale-aware decimals when the runtime offers them (P64); Node tests
  // without it read the plain number.
  const num = value => globalThis.TBG_I18N?.number ? globalThis.TBG_I18N.number(value, 2, 0) : String(value);
  const signed = bp => (bp > 0 ? '+' : '−') + num(Math.abs(bp) / 100) + '%';
  let loading;
  function load() {
    loading ||= fetch('assets/arenas/arenas.json?v=20261101')
      .then(response => { if (!response.ok) throw new Error(t('arenas.error')); return response.json(); })
      .catch(error => { loading = undefined; throw error; });
    return loading;
  }
  // Modifiers remain identical for both seats, including a buff/debuff tradeoff.
  function effect(def, data) {
    const lines = [];
    if (def.moveBp) lines.push(t('arenas.effect.move', { value: signed(def.moveBp) }));
    if (def.attackBp) lines.push(t('arenas.effect.attack', { value: signed(def.attackBp) }));
    if (def.regenBp) lines.push(t('arenas.effect.regen', { value: signed(def.regenBp) }));
    if (def.firstEvolveDiscount) lines.push(t('arenas.effect.evolve', { value: num(def.firstEvolveDiscount / data.energyScale) }));
    return lines.length ? lines.join(' · ') : t('arenas.effect.none');
  }
  const name = slug => t('arenas.' + slug + '.name');
  const theme = slug => t('arenas.' + slug + '.theme');
  const find = (data, slug) => data.arenas.find(def => def.slug === slug);
  globalThis.TbgArena = { load, effect, name, theme, find, signed };
})();
