// Arenas page and battle banner copy (ARENAS-G, D-55). The numbers are
// pinned to tbg-core by the Rust test `ui::arena_data_tests`; these check
// what the web copy derives from them in both languages.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const assets = path.resolve(__dirname, '../assets');
const read = file => JSON.parse(fs.readFileSync(path.join(assets, file), 'utf8'));
const data = read('arenas/arenas.json');
const catalogs = { en: read('i18n/en.json'), 'pt-BR': read('i18n/pt-BR.json') };
require('../assets/arenas/describe.js');
const arena = globalThis.TbgArena;
function speak(lang) {
  globalThis.TBG_I18N = { t: (key, args) => Object.entries(args || {}).reduce((text, [k, v]) => text.split('{' + k + '}').join(v), catalogs[lang][key] ?? key) };
}

test('five arenas, the Forest by default, each with a scene picture', () => {
  assert.deepEqual(data.arenas.map(def => def.slug), ['forest', 'tundra', 'volcanic', 'ruins', 'storm']);
  assert.equal(data.defaultArena, 'forest');
  for (const def of data.arenas) assert.ok(fs.existsSync(path.join(assets, 'arenas', def.image)), def.image);
});

test('each arena states exactly one rule, the Forest none', () => {
  speak('en');
  const lines = Object.fromEntries(data.arenas.map(def => [def.slug, arena.effect(def, data)]));
  assert.equal(lines.forest, 'No modifier · the reference battlefield');
  assert.equal(lines.tundra, 'Troop movement −10%');
  assert.equal(lines.volcanic, 'Troop attack speed +8%');
  assert.equal(lines.ruins, "Each player's first Evolution costs 1 Energy less");
  assert.equal(lines.storm, 'Base Energy regeneration +8%');
});

test('every arena key exists in both languages and no placeholder is left', () => {
  for (const lang of Object.keys(catalogs)) {
    speak(lang);
    for (const def of data.arenas) {
      for (const text of [arena.name(def.slug), arena.theme(def.slug), arena.effect(def, data)]) {
        assert.doesNotMatch(text, /^arenas\.|\{/, lang + ' ' + def.slug + ': ' + text);
      }
    }
  }
});
