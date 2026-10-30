// Stronghold kind text from strongholds.json, shared by the lobby picker,
// the battle HUD and the Node contract tests. Every number shown comes from
// the data file's stats and deltas, which a Rust test pins to tbg-core.
(() => {
  'use strict';
  const trim = value => String(Number(value.toFixed(2)));
  const energy = (sub, data) => trim(sub / data.energyScale);
  const tiles = milli => { const n = trim(milli / 1000); return n + (n === '1' ? ' tile' : ' tiles'); };
  const seconds = (ticks, data) => (ticks / data.tickHz).toFixed(2) + ' s';
  const percent = bp => trim(bp / 100) + '%';
  const kind = (data, name) => data.kinds.find(def => def.kind === name);
  const stats = (def, tier, branch) => tier > 0 ? def.branches[branch].steps[tier - 1].stats : def.stats;
  const cost = (data, tier) => data.fortifyCost[tier - 1] / data.energyScale;
  function fill(text, s, data) {
    const values = {
      maxHp: String(s.maxHp), damage: String(s.damage), range: tiles(s.range),
      interval: seconds(s.interval, data), spoils: energy(s.spoils, data),
      spoilsCap: energy(s.spoilsCap, data), window: seconds(data.windowTicks, data).replace('.00', ''),
      auraBp: percent(s.auraBp), auraRange: tiles(s.auraRange)
    };
    return text.replace(/\{(\w+)\}/g, (all, key) => key in values ? values[key] : all);
  }
  // The step that reaches `tier` (1 or 2) on `branch`, as short phrases.
  function effect(data, def, branch, tier) {
    const step = def.branches[branch].steps[tier - 1];
    const d = step.delta, s = step.stats, before = stats(def, tier - 1, branch);
    const more = tier > 1 ? ' more' : '';
    const parts = [];
    if (d.hpBp) parts.push('+' + percent(d.hpBp) + more + ' max HP → ' + s.maxHp);
    if (d.dmgBp) parts.push('+' + percent(d.dmgBp) + more + ' damage → ' + s.damage);
    if (d.rangeAdd) parts.push('+' + tiles(d.rangeAdd) + ' range → ' + tiles(s.range));
    if (d.interval) parts.push('Attacks every ' + seconds(s.interval, data) + ' (was ' + seconds(before.interval, data) + ')');
    if (d.spoilsAdd) parts.push('Spoils ' + energy(s.spoils, data) + ' Energy per kill');
    if (d.spoilsCapAdd) parts.push('cap ' + energy(s.spoilsCap, data) + ' Energy per window');
    if (d.auraBpAdd) parts.push('War Banner +' + percent(s.auraBp) + ' damage');
    if (d.auraRangeAdd) parts.push('War Banner radius ' + tiles(s.auraRange));
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
  const icon = name => '#sh-' + String(name).toLowerCase();
  globalThis.TbgStronghold = Object.freeze({ energy, tiles, seconds, percent, kind, stats, cost, fill, effect, sprite, icon });
})();
