// Client contracts: GAME-DESIGN §9 is authoritative for node data.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const client = path.resolve(__dirname, '..');
const assets = path.join(client, 'assets/arcana');
const data = JSON.parse(fs.readFileSync(path.join(assets, 'arcana.json'), 'utf8'));
require('../assets/arcana/preset.js');
const rules = globalThis.TbgArcanaPreset;
const byName = new Map(data.nodes.map(node => [node.name, node.index]));

test('BR-AR02: the fifteen-node table, costs and prerequisites match the canon', () => {
  const canon = fs.readFileSync(path.resolve(client, '../../docs/GAME-DESIGN.md'), 'utf8').split('## 9. Arcanas extra')[1];
  const rows = [...canon.matchAll(/^\| (\d+) \| (\w+) \/ ([^|]+?) \| (\d) \| ([^|]+) \|$/gm)];
  assert.equal(rows.length, data.nodes.length);
  assert.equal(data.nodes.length, 15);
  rows.forEach((row, index) => {
    const [, order, family, name, cost, requires] = row;
    const node = data.nodes[index];
    assert.equal(node.index, Number(order));
    assert.equal(node.family, family.toLowerCase());
    assert.equal(node.name, name.trim());
    assert.equal(node.cost, Number(cost));
    const expected = requires.trim() === 'none' ? [] : requires.split(',').map(part => {
      const match = part.trim().match(/^(.+) (\d)$/);
      assert.ok(match, part);
      return [byName.get(match[1]), Number(match[2])];
    });
    assert.deepEqual(node.requires, expected, node.name);
    assert.equal(node.maxRank, node.kind === 'original' ? 2 : 1);
  });
});

test('BR-AR02: twelve points, bounded ranks, valid default and 1842 complete presets', () => {
  assert.equal(data.pointBudget, 12);
  assert.deepEqual(data.defaultPreset, [2,2,2,2,2,2,0,0,0,0,0,0,0,0,0]);
  assert.ok(rules.complete(data.defaultPreset, data));
  const total = data.nodes.reduce((n, node) => n * (node.maxRank + 1), 1);
  let count = 0;
  for (let value = 0; value < total; value++) {
    let encoded = value;
    const ranks = data.nodes.map(node => {
      const rank = encoded % (node.maxRank + 1);
      encoded = Math.floor(encoded / (node.maxRank + 1));
      return rank;
    });
    if (rules.complete(ranks, data)) count++;
  }
  assert.equal(count, 1842);
  for (const bad of [null, {}, [], data.defaultPreset.slice(0, 14), [3,0,0,0,0,0,0,0,0,0,0,0,0,0,0], [-1,0,0,0,0,0,0,0,0,0,0,0,0,0,0], [1.5,0,0,0,0,0,0,0,0,0,0,0,0,0,0], ['2',0,2,0,2,0,0,0,0,0,0,0,0,0,0]]) {
    assert.equal(rules.editable(bad, data), false);
  }
  assert.equal(rules.complete([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], data), false);
  assert.equal(rules.editable([1,0,0,0,0,0,0,0,0,0,0,0,0,0,0], data), true);
});

test('BR-AR03: children need a parent and supremes need the complete family', () => {
  const guard = byName.get('Guard');
  const shieldwall = byName.get('Shieldwall');
  const battleRush = byName.get('Battle Rush');
  const combined = byName.get('Combined Arms');
  const ranks = [...data.defaultPreset];
  ranks[combined] = 1; ranks[shieldwall] = 1; ranks[battleRush] = 1;
  assert.equal(rules.canRemove(ranks, data, guard), false);
  assert.ok(rules.dependents(ranks, data, guard, 1).includes('Combined Arms'));
  const build = [...data.defaultPreset];
  build[byName.get('Renewal')] = 0;
  assert.equal(rules.canAdd(build, data, shieldwall), true);
  build[shieldwall] = 1;
  assert.equal(rules.complete(build, data), false);
  build[battleRush] = 1;
  assert.equal(rules.complete(build, data), true, 'War children fit without the supreme');
  build[combined] = 1;
  assert.equal(rules.complete(build, data), false, 'War now costs ten points');
});

test('BR-AR13: a schema-1 six-node preset migrates deterministically', () => {
  const migrated = rules.migrate([2,0,2,0,2,0], data);
  assert.deepEqual(migrated, data.defaultPreset);
  assert.ok(rules.complete(migrated, data));
  assert.equal(rules.migrate([1,1,1,1,1,1], data).every((rank, i) => rank <= data.nodes[i].maxRank), true);
});

test('every manifest art asset exists locally and has a safe filename', () => {
  const all = [...data.nodes.map(n => n.asset), ...data.families.flatMap(f => [f.emblem, f.banner])];
  assert.equal(all.length, 21);
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

test('P64: Arcana catalog English equals arcana.json; every node, family and copy line is paired', () => {
  const {catalogs} = require('./i18n-runtime.js');
  const en = catalogs.en, pt = catalogs['pt-BR'];
  const slug = id => id.replace(/-/g, '_');
  for (const family of data.families) {
    const id = 'arcana.family.' + slug(family.id) + '.';
    assert.equal(en[id + 'name'], family.name);
    assert.equal(en[id + 'description'], family.description);
    assert.ok(pt[id + 'name'] && pt[id + 'description'], family.id);
  }
  for (const node of data.nodes) {
    const id = 'arcana.node.' + slug(node.id) + '.';
    for (const [field, value] of [['name', node.name], ['eligibility', node.eligibility], ['per_rank', node.perRankText], ['effect', node.effect]]) {
      assert.equal(en[id + field], value, id + field);
      assert.ok(pt[id + field], id + field);
    }
    if (node.rankTwoText) assert.equal(en[id + 'rank_two'], node.rankTwoText);
  }
  for (const [key, value] of Object.entries(data.copy)) assert.equal(en['arcana.copy.' + key], value, key);
  assert.equal(en['arcana.summon'], data.summon.text);
  // Localized requirement text keeps the same ranks the rules compute.
  const ranks = data.nodes.map(() => 0);
  const combined = data.nodes[byName.get('Combined Arms')];
  const describe = (node, rank, now) => pt['arcana.requirement'].replace('{name}', pt['arcana.node.' + slug(node.id) + '.name']).replace('{rank}', rank).replace('{now}', now);
  assert.deepEqual(rules.missing(ranks, data, combined, describe), rules.missing(ranks, data, combined).map((_, i) => describe(data.nodes[combined.requires[i][0]], combined.requires[i][1], 0)));
});
