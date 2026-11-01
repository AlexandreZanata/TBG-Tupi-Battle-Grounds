// P64 localization contracts: the real i18n.js runtime (switch ordering,
// number precision, plurals), API code labels and literal key references.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {boot, catalogs} = require('./i18n-runtime.js');
const D = require('../assets/ranking/data.js');
const client = path.resolve(__dirname, '..');

test('T04: a slow older language load never replaces the newest choice', async () => {
  const {i18n, document, events} = await boot('en', {'pt-BR': 40});
  const slow = i18n.setLang('pt-BR');
  await i18n.setLang('en');
  await slow;
  assert.equal(i18n.lang, 'en');
  assert.equal(document.documentElement.lang, 'en');
  assert.equal(i18n.t('result.victory'), 'VICTORY');
  await i18n.setLang('pt-BR');
  assert.equal(i18n.t('result.victory'), 'VITÓRIA');
  assert.equal(events.at(-1).lang, 'pt-BR');
});

test('T06: numbers keep each precision in one session; plurals follow CLDR for 0/1/many', async () => {
  const {i18n} = await boot('pt-BR');
  assert.equal(i18n.number(1.5, 1), '1,5');
  assert.equal(i18n.number(1.5, 2), '1,50');
  assert.equal(i18n.number(0.25, 2, 0), '0,25');
  assert.equal(i18n.number(12, 2, 0), '12');
  assert.equal(i18n.number(1234, 0), '1.234');
  assert.deepEqual([0, 1, 2].map(n => i18n.plural('ranking.web.matches', n)), ['0 partida ranqueada', '1 partida ranqueada', '2 partidas ranqueadas']);
  const en = (await boot('en')).i18n;
  assert.deepEqual([0, 1, 2].map(n => en.plural('ranking.web.matches', n)), ['0 ranked matches', '1 ranked match', '2 ranked matches']);
  assert.equal(en.number(1.5, 2), '1.50');
});

test('T05: API codes get localized labels with safe fallbacks; data failures carry codes', () => {
  for (const lang of ['en', 'pt-BR']) {
    const c = catalogs[lang];
    for (const f of ['humans', 'undead', 'machines', 'plants']) assert.ok(c['faction.' + f], lang + ' ' + f);
    for (const r of ['w', 'l', 'd', 'unknown']) assert.ok(c['ranking.result.' + r], lang + ' ' + r);
    for (const r of ['stronghold_destroyed', 'time_up', 'forfeit', 'unknown']) assert.ok(c['ranking.reason.' + r], lang + ' ' + r);
    for (const e of ['rate_limited', 'unavailable', 'offline', 'incomplete']) assert.ok(c['ranking.error.' + e], lang + ' ' + e);
    for (const s of ['idle', 'playing', 'queued', 'watching']) assert.ok(c['live.status.' + s], lang + ' ' + s);
  }
  assert.throws(() => D.board({rows: [{rank: 1, name: '<b>x</b>'}], total: 1}), error => error.code === 'incomplete');
  assert.throws(() => D.profile({name: 'Valid_1', rating: 1, games: 0, wins: 0, losses: 0, draws: 0, win_rate: 0, rank: null, recent: [{opponent: 'Other', result: 'X', delta: 0}]}), error => error.code === 'incomplete');
});

// T08: every complete key literal a script or page references exists in
// both catalogs. Dynamic families (`'arenas.' + id`) are not literals.
test('T08: referenced literal keys exist in both catalogs', () => {
  const files = [];
  const walk = dir => {
    for (const entry of fs.readdirSync(dir, {withFileTypes: true})) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (/\.(js|html)$/.test(entry.name)) files.push(full);
    }
  };
  walk(path.join(client, 'assets'));
  walk(path.join(client, 'ranking'));
  files.push(path.join(client, 'index.html'));
  const missing = [];
  for (const file of files) {
    const source = fs.readFileSync(file, 'utf8');
    const keys = [
      ...[...source.matchAll(/\bt\('([a-z0-9_.]+)'/g)].map(m => m[1]),
      ...[...source.matchAll(/data-i18n(?:-[a-z-]+)?="([a-z0-9_.]+)"/g)].map(m => m[1]),
    ].filter(key => !key.endsWith('.'));
    for (const key of keys) {
      if (!(key in catalogs.en) || !(key in catalogs['pt-BR'])) missing.push(path.relative(client, file) + ': ' + key);
    }
  }
  assert.deepEqual(missing, []);
});
