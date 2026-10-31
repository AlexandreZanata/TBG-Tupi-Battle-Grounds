//! Every buyable unit ships a Shop portrait: 400x400 WebP within the file budget.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '../../..');
const portraits = path.resolve(__dirname, '../assets/battle/portraits');
const BUDGET = 40 * 1024;
// Same rule as assets/battle/hud.js `slug`.
const slug = name => name.toLowerCase().replace(/[^a-z0-9]/g, '');

// The contract follows ALL_KINDS and the authoritative summon classification.
function buyable() {
  const source = fs.readFileSync(path.join(root, 'crates/tbg-core/src/types.rs'), 'utf8');
  const all = source.match(/ALL_KINDS: \[UnitKind; \d+\] = \[([\s\S]*?)\];/);
  const summons = source.match(/pub fn is_summon\(self\) -> bool \{\s*matches!\(self,([\s\S]*?)\)\s*\}/);
  assert.ok(all, 'ALL_KINDS is declared in tbg-core types.rs');
  assert.ok(summons, 'the summon classification is declared in tbg-core types.rs');
  const summoned = new Set([...summons[1].matchAll(/UnitKind::(\w+)/g)].map(m => m[1]));
  return [...all[1].matchAll(/UnitKind::(\w+)/g)].map(m => m[1]).filter(kind => !summoned.has(kind));
}

// Reads the canvas size from the RIFF header: VP8X, lossy VP8 or lossless VP8L.
function webpSize(buffer) {
  assert.equal(buffer.toString('latin1', 0, 4), 'RIFF');
  assert.equal(buffer.toString('latin1', 8, 12), 'WEBP');
  const chunk = buffer.toString('latin1', 12, 16);
  if (chunk === 'VP8X') return {width: 1 + buffer.readUIntLE(24, 3), height: 1 + buffer.readUIntLE(27, 3)};
  if (chunk === 'VP8 ') return {width: buffer.readUInt16LE(26) & 0x3fff, height: buffer.readUInt16LE(28) & 0x3fff};
  if (chunk === 'VP8L') {
    const bits = buffer.readUInt32LE(21);
    return {width: (bits & 0x3fff) + 1, height: ((bits >> 14) & 0x3fff) + 1};
  }
  assert.fail('unknown WebP chunk ' + chunk);
}

test('every buyable unit has a 400x400 Shop portrait within 40 KB', () => {
  const kinds = buyable();
  assert.ok(kinds.length > 0, 'the buyable roster must not be empty');
  for (const kind of kinds) {
    const file = path.join(portraits, slug(kind) + '.webp');
    assert.ok(fs.existsSync(file), kind + ': missing ' + path.relative(root, file));
    const buffer = fs.readFileSync(file);
    assert.deepEqual(webpSize(buffer), {width: 400, height: 400}, kind);
    assert.ok(buffer.length <= BUDGET, kind + ': ' + buffer.length + ' B > ' + BUDGET + ' B');
  }
});
