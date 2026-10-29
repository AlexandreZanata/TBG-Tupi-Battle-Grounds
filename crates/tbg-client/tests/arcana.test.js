// Client preview contracts: GAME-DESIGN §9 is authoritative for node data.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const client = path.resolve(__dirname, '..');
const assets = path.join(client, 'assets/arcana');
const data = JSON.parse(fs.readFileSync(path.join(assets, 'arcana.json'), 'utf8'));
require('../assets/arcana/preset.js');
const rules = globalThis.TbgArcanaPreset;

test('BR-AR02/03/10: six nodes match the canon table, serialization order and values', () => {
  const canon = fs.readFileSync(path.resolve(client, '../../docs/GAME-DESIGN.md'), 'utf8').split('## 9. Arcanas extra')[1];
  const rows = [...canon.matchAll(/^\| (\d+) \| (\w+) \/ (\w+) \| ([^|]+) \| \+(\d+)% ([^|]+) \| \+(\d+)% \|$/gm)];
  assert.equal(rows.length, data.nodes.length);
  assert.equal(data.nodes.length, 6);
  rows.forEach((row, index) => {
    const [, order, family, name, eligibility, percent, effect, maximum] = row;
    const node = data.nodes[index];
    assert.equal(node.index, Number(order));
    assert.equal(node.family, family.toLowerCase());
    assert.equal(node.name, name);
    assert.equal(node.eligibility, eligibility.trim());
    assert.deepEqual(node.archetypes, eligibility.startsWith('Support with') ? ['Support'] : eligibility.trim().split(', '));
    assert.equal(node.requiresExistingHeal, eligibility.includes('existing heal'));
    assert.equal(node.perRankPercent, Number(percent));
    assert.equal(node.rankTwoPercent, Number(maximum));
    assert.equal(node.perRankText, '+' + percent + '% ' + effect.trim());
    assert.equal(node.rankTwoText, '+' + maximum + '%');
  });
});

test('BR-AR02: exactly six points, bounded ranks, valid default and 141 complete presets', () => {
  assert.equal(data.pointBudget, 6); assert.equal(data.maxRank, 2);
  assert.deepEqual(data.defaultPreset, [2,0,2,0,2,0]);
  assert.ok(rules.complete(data.defaultPreset, data));
  let count = 0;
  for (let value = 0; value < 3 ** data.nodes.length; value++) {
    let encoded = value;
    const ranks = data.nodes.map(() => { const rank = encoded % 3; encoded = Math.floor(encoded / 3); return rank; });
    const total = ranks.reduce((sum, rank) => sum + rank, 0);
    assert.equal(rules.editable(ranks, data), total <= 6);
    assert.equal(rules.complete(ranks, data), total === 6);
    if (rules.complete(ranks, data)) count++;
  }
  assert.equal(count, 141);
  for (const bad of [null, {}, [], [2,0,2,0,2], [3,0,0,0,0,0], [-1,0,0,0,0,0], [1.5,0,0,0,0,0], ['2',0,2,0,2,0], [255,0,0,0,0,0], [2,2,2,2,2,2]]) {
    assert.equal(rules.editable(bad, data), false);
  }
  assert.equal(rules.complete([0,0,0,0,0,0], data), false);
  assert.equal(rules.editable([1,0,0,0,0,0], data), true);
});

test('every manifest art asset exists locally and has a safe filename', () => {
  const all = [...data.nodes.map(n => n.asset), ...data.families.flatMap(f => [f.emblem, f.banner])];
  assert.equal(all.length, 12);
  for (const asset of all) for (const filename of new Set(Object.values(asset))) {
    assert.match(filename, /^[a-z-]+\.(svg|webp)$/);
    assert.ok(fs.statSync(path.join(assets, filename)).size > 0, filename);
  }
});

test('BR-AR03/10 preview: roster includes Nester, Rampart; Renewal only reaches an existing healer', () => {
  assert.equal(data.roster.find(u => u.name === 'Nester').archetype, 'Support');
  assert.equal(data.roster.find(u => u.name === 'Rampart').archetype, 'Tank');
  const renewal = data.nodes.find(n => n.name === 'Renewal');
  assert.deepEqual(data.roster.filter(u => renewal.archetypes.includes(u.archetype) && u.existingHeal).map(u => u.name), ['Bloomkeeper']);
  assert.equal(data.summon.affected, false);
  assert.match(data.summon.text, /Proposed/);
});
