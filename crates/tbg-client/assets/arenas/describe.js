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
    loading ||= fetch('assets/arenas/arenas.json?v=20261030')
      .then(response => { if (!response.ok) throw new Error(t('arenas.error')); return response.json(); })
      .catch(error => { loading = undefined; throw error; });
    return loading;
  }
  // One arena changes one thing; the Forest changes nothing.
  function effect(def, data) {
    if (def.moveBp) return t('arenas.effect.move', { value: signed(def.moveBp) });
    if (def.attackBp) return t('arenas.effect.attack', { value: signed(def.attackBp) });
    if (def.firstEvolveDiscount) return t('arenas.effect.evolve', { value: num(def.firstEvolveDiscount / data.energyScale) });
    if (def.regenBp) return t('arenas.effect.regen', { value: signed(def.regenBp) });
    return t('arenas.effect.none');
  }
  const name = slug => t('arenas.' + slug + '.name');
  const theme = slug => t('arenas.' + slug + '.theme');
  const find = (data, slug) => data.arenas.find(def => def.slug === slug);
  globalThis.TbgArena = { load, effect, name, theme, find, signed };
})();
