// Stronghold kind text from strongholds.json, shared by the lobby picker,
// the battle HUD and the Node contract tests. Every number shown comes from
// the data file's stats and deltas, which a Rust test pins to tbg-core.
// Prose resolves through the shared catalog (`stronghold.<kind>.*`, P64);
// the JSON keeps ids, numbers, art and the English canon the tests pin.
(() => {
  'use strict';
  const I = () => globalThis.TBG_I18N;
  const t = (key, args) => I().t(key, args);
  const num = value => I().number(value, 2, 0);
  const energy = (sub, data) => num(sub / data.energyScale);
  const tiles = milli => { const n = milli / 1000; return I().plural('stronghold.tiles', n, {n: num(n)}); };
  const seconds = (ticks, data) => t('stronghold.seconds', {n: I().number(ticks / data.tickHz, 2)});
  const percent = bp => num(bp / 100) + '%';
  const kind = (data, name) => data.kinds.find(def => def.kind === name);
  const stats = (def, tier, branch) => tier > 0 ? def.branches[branch].steps[tier - 1].stats : def.stats;
  const cost = (data, tier) => data.fortifyCost[tier - 1] / data.energyScale;
  // Catalog text for `def` at `path` (`summary`, `passive.text`,
  // `advantage.0`); the JSON English stays the fallback for a missing key.
  const id = def => def.kind.toLowerCase();
  const resolve = (key, english) => I().has(key) ? t(key) : english;
  const name = def => resolve('stronghold.' + id(def) + '.name', def.name);
  const branchName = (def, branch) => resolve('stronghold.' + id(def) + '.branch.' + branch + '.name', def.branches[branch].name);
  const stepName = (def, branch, index) => resolve('stronghold.' + id(def) + '.step.' + (branch * 2 + index) + '.name', def.branches[branch].steps[index].name);
  function english(def, path) {
    const [head, index] = path.split('.');
    if (head === 'advantage') return def.advantages[index];
    if (head === 'disadvantage') return def.disadvantages[index];
    return path.split('.').reduce((node, part) => node?.[part], def);
  }
  const text = (def, path) => resolve('stronghold.' + id(def) + '.' + path, english(def, path));
  const copy = (data, key) => resolve('stronghold.copy.' + key, data.copy[key]);
  function fill(template, s, data) {
    const values = {
      maxHp: String(s.maxHp), damage: String(s.damage), range: tiles(s.range),
      interval: seconds(s.interval, data), spoils: energy(s.spoils, data),
      spoilsCap: energy(s.spoilsCap, data), window: t('stronghold.seconds', {n: num(data.windowTicks / data.tickHz)}),
      auraBp: percent(s.auraBp), auraRange: tiles(s.auraRange)
    };
    return template.replace(/\{(\w+)\}/g, (all, key) => key in values ? values[key] : all);
  }
  // The step that reaches `tier` (1 or 2) on `branch`, as short phrases.
  function effect(data, def, branch, tier) {
    const step = def.branches[branch].steps[tier - 1];
    const d = step.delta, s = step.stats, before = stats(def, tier - 1, branch);
    const more = tier > 1 ? '_more' : '';
    const parts = [];
    if (d.hpBp) parts.push(t('stronghold.effect.hp' + more, {pct: percent(d.hpBp), value: s.maxHp}));
    if (d.dmgBp) parts.push(t('stronghold.effect.dmg' + more, {pct: percent(d.dmgBp), value: s.damage}));
    if (d.rangeAdd) parts.push(t('stronghold.effect.range', {add: tiles(d.rangeAdd), value: tiles(s.range)}));
    if (d.interval) parts.push(t('stronghold.effect.interval', {value: seconds(s.interval, data), before: seconds(before.interval, data)}));
    if (d.spoilsAdd) parts.push(t('stronghold.effect.spoils', {value: energy(s.spoils, data)}));
    if (d.spoilsCapAdd) parts.push(t('stronghold.effect.spoils_cap', {value: energy(s.spoilsCap, data)}));
    if (d.auraBpAdd) parts.push(t('stronghold.effect.aura', {pct: percent(s.auraBp)}));
    if (d.auraRangeAdd) parts.push(t('stronghold.effect.aura_range', {value: tiles(s.auraRange)}));
    return parts.join(', ');
  }
  let spriteLoad;
  // Inline the tower drawings once so every `<use href="#sh-…">` (and its
  // gradients) resolves inside this document.
  function sprite() {
    spriteLoad ||= fetch('assets/stronghold/towers.svg')
      .then(response => { if (!response.ok) throw new Error('tower art unavailable'); return response.text(); })
      .then(markup => {
        const holder = document.createElement('div');
        // Never display:none: Chrome drops gradients defined in a hidden subtree.
        holder.style.cssText = 'position:absolute;width:0;height:0;overflow:hidden';
        holder.setAttribute('aria-hidden', 'true');
        holder.innerHTML = markup; // our own static asset, never player data
        document.body.append(holder);
      })
      .catch(() => { spriteLoad = undefined; });
    return spriteLoad;
  }
  const icon = kindName => '#sh-' + String(kindName).toLowerCase();
  globalThis.TbgStronghold = Object.freeze({ energy, tiles, seconds, percent, kind, stats, cost, fill, effect, sprite, icon, name, branchName, stepName, text, copy });
})();
