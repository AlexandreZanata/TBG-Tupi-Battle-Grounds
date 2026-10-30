// Stronghold picker and Fortification panel contracts (D-46). The numbers
// themselves are pinned to tbg-core by the Rust test
// `ui::stronghold_data_tests`; these check what the web copy derives.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const assets = path.resolve(__dirname, '../assets');
const data = JSON.parse(fs.readFileSync(path.join(assets, 'stronghold/strongholds.json'), 'utf8'));
require('../assets/stronghold/describe.js');
const tower = globalThis.TbgStronghold;
const byKind = name => tower.kind(data, name);

test('BR-SK01: four selectable kinds, Bastion by default, Classic never offered', () => {
  assert.deepEqual(data.kinds.map(def => def.kind), ['Treasury', 'Bastion', 'Artillery', 'Command']);
  assert.equal(data.defaultKind, 'Bastion');
  assert.equal(data.storageKey, 'tbg.stronghold');
  assert.equal(byKind('Classic'), undefined);
});

test('prose holds no hand-written numbers: every figure comes from tested stats', () => {
  for (const def of data.kinds) {
    for (const line of [def.summary, def.passive.text, ...def.advantages, ...def.disadvantages]) {
      assert.doesNotMatch(line.replace(/\{\w+\}/g, ''), /\d/, def.kind + ': ' + line);
      assert.doesNotMatch(tower.fill(line, def.stats, data), /\{/, def.kind + ' placeholder left: ' + line);
    }
  }
});

test('superlatives in the copy hold for the data', () => {
  const hp = data.kinds.map(def => def.stats.maxHp), damage = data.kinds.map(def => def.stats.damage);
  assert.equal(byKind('Bastion').stats.maxHp, Math.max(...hp));
  assert.equal(byKind('Artillery').stats.maxHp, Math.min(...hp));
  assert.equal(byKind('Artillery').stats.damage, Math.max(...damage));
  assert.equal(byKind('Bastion').stats.damage, Math.min(...damage));
});

test('BR-SK02/SK04/SK05: derived effects, passives and costs read as the rules say', () => {
  assert.equal(tower.cost(data, 1), 3);
  assert.equal(tower.cost(data, 2), 4);
  const treasury = byKind('Treasury');
  assert.match(tower.fill(treasury.passive.text, treasury.stats, data), /pays you 0\.15 Energy, at most 0\.6 Energy per 10 s window/);
  assert.equal(tower.effect(data, treasury, 0, 1), '+12% max HP → ' + treasury.branches[0].steps[0].stats.maxHp);
  assert.equal(tower.effect(data, treasury, 0, 2), '+12% more max HP → ' + treasury.branches[0].steps[1].stats.maxHp);
  assert.equal(tower.effect(data, treasury, 1, 2), 'Spoils 0.25 Energy per kill, cap 0.9 Energy per window');
  const artillery = byKind('Artillery');
  assert.equal(tower.effect(data, artillery, 1, 1), 'Attacks every 0.85 s (was 1.00 s)');
  assert.equal(tower.effect(data, artillery, 1, 2), 'Attacks every 0.70 s (was 0.85 s)');
  const bastion = byKind('Bastion');
  assert.equal(tower.effect(data, bastion, 1, 1), '+1 tile range → 7 tiles');
  const command = byKind('Command');
  assert.match(tower.fill(command.passive.text, command.stats, data), /within 4 tiles of this tower deal \+5% damage/);
  assert.equal(tower.effect(data, command, 0, 2), 'War Banner +11% damage');
  assert.equal(tower.effect(data, command, 1, 2), 'War Banner radius 6 tiles');
  for (const def of data.kinds) for (const branch of [0, 1]) for (const tier of [1, 2]) {
    assert.notEqual(tower.effect(data, def, branch, tier), '', def.kind + ' ' + branch + ' ' + tier);
  }
});

test('every kind has its tower drawing and the HUD understands every offer state', () => {
  const sprite = fs.readFileSync(path.join(assets, 'stronghold/towers.svg'), 'utf8');
  for (const def of data.kinds) assert.ok(sprite.includes('id="' + tower.icon(def.kind).slice(1) + '"'), def.kind);
  const css = fs.readFileSync(path.join(assets, 'battle/fortify.css'), 'utf8');
  for (const state of ['short', 'closed', 'paused', 'idle', 'pending']) assert.ok(css.includes('[data-state=' + state + ']'), state);
});
