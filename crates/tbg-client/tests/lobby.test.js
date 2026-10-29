// Lobby projection contract. Append-only: change a test here only together
// with the rule it guards.
//
// Regression (2026-10-29, production): when a player was logged in, every
// Arcana node in the workshop showed "#3 Global" and lost its art and text.
// lobby.js writes the ranking into every element matching `[data-rank]`, and
// the Arcana editor also uses `data-rank` (0-2) on each node card inside the
// same `#tbg-lobby` root. Lobby selectors must never match an attribute the
// Arcana editor writes.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const assets = path.resolve(__dirname, '../assets');
const lobby = fs.readFileSync(path.join(assets, 'lobby/lobby.js'), 'utf8');
const editor = fs.readFileSync(path.join(assets, 'arcana/editor.js'), 'utf8');
const toAttribute = name => 'data-' + name.replace(/[A-Z]/g, letter => '-' + letter.toLowerCase());

test('lobby text projection never targets an attribute the Arcana editor writes', () => {
  const written = new Set([...editor.matchAll(/\.dataset\.(\w+)\s*=/g)].map(match => toAttribute(match[1])));
  assert.ok(written.has('data-rank'), 'the editor is expected to write data-rank on each node');
  const projected = [...lobby.matchAll(/\bset\(\s*'([^']+)'/g)].flatMap(match => match[1].match(/\[data-[\w-]+\]/g) || []);
  assert.ok(projected.length > 0, 'lobby.js is expected to project text through set()');
  const clashes = projected.filter(selector => written.has(selector.slice(1, -1)));
  assert.deepEqual(clashes, [], 'lobby.js would overwrite Arcana editor nodes');
});
