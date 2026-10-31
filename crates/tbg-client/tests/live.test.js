// Lobby live surfaces (P47): header chip, Live battles panel, See-all dialog.
// Shapes come from GET /api/live and GET /api/online; the copy is checked in
// both catalogs. The DOM is a small fake, so these run under plain node.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assets = path.resolve(__dirname, '../assets');
const read = file => fs.readFileSync(path.join(assets, file), 'utf8');
const liveSource = read('lobby/live.js');
const dataSource = read('lobby/live-data.js');
const lobbySource = read('lobby/lobby.js');
const editorSource = read('arcana/editor.js');
const pickerSource = read('stronghold/picker.js');
const hubSource = read('arenas/hub.js');
const html = fs.readFileSync(path.resolve(__dirname, '../index.html'), 'utf8');
const css = read('lobby/live.css');
const catalogs = {en: JSON.parse(read('i18n/en.json')), 'pt-BR': JSON.parse(read('i18n/pt-BR.json'))};

const ID = {a: '0123456789abcdef', b: 'fedcba9876543210', c: '1111222233334444', d: 'aaaabbbbccccdddd', e: '0000111122223333', f: '9999888877776666'};
const match = (id, extra = {}) => ({
  id, mode: 'casual', arena: 'forest', elapsed_s: 75, hp_bp: [10000, 4000], spectators: 2,
  players: [{name: 'Ana', rating: 1200, kind: 'classic', tier: 1}, {name: 'Bruno', rating: null, kind: 'bastion', tier: 2}],
  ...extra,
});
const live = (matches, featured = null, online = {players: 9, accounts: 4, in_battle: 2, watching: 1}) => ({online, featured, matches});
const okJson = body => async () => ({ok: true, json: async () => body});
const failing = async () => ({ok: false, json: async () => ({})});
const flush = () => new Promise(resolve => setImmediate(resolve));

// Every selector live.js queries, so the fake root can refuse unknown ones.
const selectorsOf = source => [...new Set([...source.matchAll(/\bq\('([^']+)'\)/g)].map(m => m[1]))];

function node() {
  const listeners = {};
  return {
    className: '', textContent: '', hidden: false, dataset: {}, style: {}, value: '', children: [], listeners,
    append(...items) { this.children.push(...items); },
    replaceChildren(...items) { this.children = items; },
    addEventListener(type, fn) { (listeners[type] ||= []).push(fn); },
    select() { this.selected = true; },
    click() { this.clicks = (this.clicks || 0) + 1; },
  };
}
const textOf = n => [n.textContent, ...n.children.map(textOf)].filter(Boolean).join(' ');
const fire = (target, type, event = {}) => (target.listeners[type] || []).forEach(fn => fn(event));
// A click whose closest(selector) finds `dataset` for the matching selector only.
const clickOn = (selector, dataset) => ({target: {closest: sel => (sel === selector ? {dataset} : null)}});

function setup({hidden = false, lang = 'en', reply = failing, href = 'https://tbg.test/index.html?lang=pt-BR'} = {}) {
  const catalog = catalogs[lang];
  const t = (key, args) => Object.entries(args || {}).reduce((text, [k, v]) => text.split('{' + k + '}').join(String(v)), catalog[key] ?? key);
  const nodes = Object.fromEntries(selectorsOf(liveSource).map(sel => [sel, node()]));
  const root = node();
  root.hidden = hidden;
  root.querySelector = sel => {
    if (!(sel in nodes)) throw new Error('live.js queries an unknown selector: ' + sel);
    return nodes[sel];
  };
  const io = {calls: [], assigned: null, observers: [], timers: new Map(), nextId: 1, reply, execResult: false, clipboardOk: true, copied: []};
  const doc = {
    visibilityState: 'visible', listeners: {},
    getElementById: id => (id === 'tbg-lobby' ? root : null),
    createElement: () => node(),
    addEventListener(type, fn) { (this.listeners[type] ||= []).push(fn); },
    fire(type) { (this.listeners[type] || []).forEach(fn => fn()); },
    execCommand: () => io.execResult,
  };
  const context = {
    document: doc, URL,
    location: {href, origin: new URL(href).origin, pathname: new URL(href).pathname, assign(url) { io.assigned = url; }},
    navigator: {clipboard: {writeText: async text => { if (!io.clipboardOk) throw new Error('denied'); io.copied.push(text); }}},
    fetch: async url => { io.calls.push(url); return io.reply(url); },
    setInterval: (fn, ms) => { const id = io.nextId++; io.timers.set(id, {fn, ms}); return id; },
    clearInterval: id => { io.timers.delete(id); },
    MutationObserver: class { constructor(fn) { this.fn = fn; io.observers.push(fn); } observe() {} },
    TBG_I18N: {t, ready: Promise.resolve(), lang},
  };
  context.window = context;
  vm.createContext(context);
  vm.runInContext(dataSource, context);
  vm.runInContext(liveSource, context);
  return {
    io, nodes, root, doc,
    sync: () => io.observers.forEach(fn => fn()),
    tick: () => [...io.timers.values()].forEach(timer => timer.fn()),
    chip: () => nodes['[data-live-chip]'].textContent,
    title: () => nodes['[data-live-title]'].textContent,
    status: () => nodes['[data-live-status]'].textContent,
    rows: () => nodes['[data-live-rows]'].children,
    text: textOf,
  };
}

test('shape checks keep valid battles, drop malformed rows, and reject a bad payload', () => {
  const context = {};
  vm.runInNewContext(dataSource, context);
  const D = context.TbgLiveData;
  const good = match(ID.a);
  const parsed = D.parseLive(live([good, match('nothex'), match(ID.b, {mode: 'practice'}), match(ID.c, {hp_bp: [1]})], ID.a));
  assert.deepEqual(parsed.matches.map(m => m.id), [ID.a]);
  assert.equal(parsed.featured, ID.a);
  assert.equal(D.parseLive(live([], 'XYZ')).featured, null, 'a featured id that is not 16 hex is ignored');
  assert.equal(D.parseLive({matches: []}), null, 'no online block means a bad payload');
  assert.equal(D.parseLive({online: {players: -1, accounts: 0, in_battle: 0, watching: 0}, matches: []}), null);
  const online = D.parseOnline({total: 3, rows: [{name: 'Ana', rating: 1200, status: 'playing'}, {name: '', rating: 5, status: 'idle'}, null]});
  assert.equal(online.total, 3);
  assert.deepEqual(online.rows.map(row => row.name), ['Ana']);
  assert.equal(D.clock(75), '1:15');
  assert.equal(D.clock(0), '0:00');
  assert.equal(D.percent(12345), 100, 'HP basis points clamp to 0..10000');
});

test('the chip is a placeholder until the first good answer, never a zero', async () => {
  const h = setup({reply: failing});
  await flush();
  assert.equal(h.chip(), '… online');
  assert.doesNotMatch(h.chip(), /0/);
  h.io.reply = okJson(live([match(ID.a)], ID.a, {players: 7, accounts: 3, in_battle: 2, watching: 0}));
  h.tick();
  await flush();
  assert.equal(h.chip(), '7 online · 2 in battle');
});

test('a failed or malformed answer keeps the last good render', async () => {
  const h = setup({reply: okJson(live([match(ID.a)], ID.a, {players: 7, accounts: 3, in_battle: 2, watching: 0}))});
  await flush();
  assert.equal(h.chip(), '7 online · 2 in battle');
  h.io.reply = failing;
  h.tick();
  await flush();
  assert.equal(h.chip(), '7 online · 2 in battle');
  h.io.reply = async () => ({ok: true, json: async () => ({online: 'broken', matches: 'no'})});
  h.tick();
  await flush();
  assert.equal(h.chip(), '7 online · 2 in battle');
  assert.equal(h.rows().length, 1);
});

test('the featured battle leads, then at most three more rows', async () => {
  const all = [match(ID.a), match(ID.b), match(ID.c), match(ID.d), match(ID.e), match(ID.f)];
  const h = setup({reply: okJson(live(all, ID.b))});
  await flush();
  assert.equal(h.title(), 'Live battles · 6');
  assert.equal(h.rows().length, 4);
  assert.equal(h.rows()[0].className, 'live-row is-featured');
  assert.deepEqual(h.rows().map(row => row.children[3].children[0].dataset.liveWatch), [ID.b, ID.a, ID.c, ID.d]);
  assert.match(h.text(h.rows()[0]), /Ana 1200 Bruno Guest/);
  assert.match(h.text(h.rows()[0]), /Verdant Forest · 1:15 · 2 watching/, 'arena, match time and spectators');
  assert.match(h.text(h.rows()[0]), /Casual/);
});

test('empty state says so and Play Casual presses the existing Casual button', async () => {
  const h = setup({reply: okJson(live([]))});
  await flush();
  assert.equal(h.status(), 'No battles right now');
  assert.equal(h.nodes['[data-live-play]'].hidden, false);
  h.nodes['[data-live-play]'].listeners.click[0]();
  assert.equal(h.nodes['button[data-action="quick"]'].clicks, 1);
});

test('polling runs only while the lobby is shown and the tab is visible', async () => {
  const h = setup({hidden: true, reply: okJson(live([]))});
  await flush();
  assert.deepEqual(h.io.calls, []);
  assert.equal(h.io.timers.size, 0);
  h.root.hidden = false;
  h.sync();
  await flush();
  assert.deepEqual(h.io.calls, ['/api/live']);
  assert.deepEqual([...h.io.timers.values()].map(timer => timer.ms), [5000]);
  h.tick();
  await flush();
  assert.equal(h.io.calls.length, 2);
  h.doc.visibilityState = 'hidden';
  h.doc.fire('visibilitychange');
  assert.equal(h.io.timers.size, 0, 'hidden tab clears the timer');
  h.tick();
  await flush();
  assert.equal(h.io.calls.length, 2, 'no request while hidden');
  h.doc.visibilityState = 'visible';
  h.doc.fire('visibilitychange');
  await flush();
  assert.equal(h.io.calls.length, 3, 'coming back fetches at once');
  h.root.hidden = true;
  h.sync();
  assert.equal(h.io.timers.size, 0, 'leaving the lobby stops polling');
});

test('Watch navigates the same tab with ?watch= and keeps other parameters', async () => {
  const h = setup({reply: okJson(live([match(ID.b)], ID.b))});
  await flush();
  fire(h.root, 'click', clickOn('[data-live-watch]', {liveWatch: ID.b}));
  assert.equal(h.io.assigned, 'https://tbg.test/index.html?lang=pt-BR&watch=' + ID.b);
  h.io.assigned = null;
  fire(h.root, 'click', clickOn('[data-live-watch]', {liveWatch: 'not-an-id'}));
  assert.equal(h.io.assigned, null, 'an id that is not 16 hex never reaches the URL');
});

test('Copy watch link copies the watch URL, with a selected field when the clipboard is refused', async () => {
  const h = setup({reply: okJson(live([match(ID.a)], ID.a))});
  await flush();
  const link = 'https://tbg.test/index.html?watch=' + ID.a;
  fire(h.root, 'click', clickOn('[data-live-copy]', {liveCopy: ID.a}));
  await flush();
  assert.deepEqual(h.io.copied, [link]);
  assert.equal(h.nodes['[data-live-copy-status]'].textContent, 'Link copied');
  h.io.clipboardOk = false;
  h.io.execResult = false;
  fire(h.root, 'click', clickOn('[data-live-copy]', {liveCopy: ID.a}));
  await flush();
  const field = h.nodes['[data-live-link]'];
  assert.equal(field.value, link);
  assert.equal(field.hidden, false);
  assert.equal(field.selected, true);
  assert.equal(h.nodes['[data-live-copy-status]'].textContent, 'Select the link and copy it');
});

test('See all loads the online accounts only when the dialog opens, and shows their status', async () => {
  const h = setup({reply: async url => (url.startsWith('/api/online') ? {ok: true, json: async () => ({total: 3, rows: [
    {name: 'Ana', rating: 1200, status: 'playing'}, {name: 'Caio', rating: 900, status: 'watching'}, {name: '', rating: 1, status: 'idle'}]})} : okJson(live([]))())});
  await flush();
  assert.equal(h.io.calls.some(url => url.startsWith('/api/online')), false, 'no accounts request at boot');
  h.nodes['[data-live-see-all]'].listeners.click[0]();
  await flush();
  assert.deepEqual(h.io.calls.filter(url => url.startsWith('/api/online')), ['/api/online?offset=0&limit=100']);
  assert.equal(h.nodes['[data-live-accounts-title]'].textContent, 'Accounts online · 3');
  const accounts = h.nodes['[data-live-accounts]'].children;
  assert.equal(accounts.length, 2, 'a nameless row is dropped');
  assert.equal(accounts[0].children[2].textContent, 'In battle');
  assert.equal(accounts[1].children[2].textContent, 'Watching');
});

test('the copy is the same text in both languages and every key exists in both catalogs', () => {
  const used = new Set();
  for (const source of [liveSource, dataSource, html]) {
    for (const m of source.matchAll(/['"](live\.[\w.]+)['"]/g)) used.add(m[1]);
    for (const m of source.matchAll(/data-i18n(?:-aria-label)?="(live\.[\w.]+)"/g)) used.add(m[1]);
  }
  used.add('live.mode.casual').add('live.mode.ranked');
  for (const key of ['spectate.watching_badge', 'spectate.copy_link']) used.add(key);
  for (const key of used) {
    if (key.endsWith('.')) continue;
    for (const [lang, catalog] of Object.entries(catalogs)) assert.ok(key in catalog, lang + ' misses ' + key);
  }
  for (const [lang, catalog] of Object.entries(catalogs)) {
    assert.equal(Object.keys(catalog).filter(key => key.startsWith('live.')).length, Object.keys(catalogs.en).filter(key => key.startsWith('live.')).length, lang);
  }
});

test('live.js queries only selectors that exist in index.html', () => {
  for (const selector of selectorsOf(liveSource)) {
    for (const name of selector.match(/data-[\w-]+/g)) assert.match(html, new RegExp('\\s' + name + '(?=[\\s>=])'), selector);
    const value = selector.match(/="([^"]+)"/);
    if (value) assert.ok(html.includes(`data-action="${value[1]}"`), selector);
  }
});

test('live data-* names never collide with lobby.js, Arcana, the picker or the Arenas page', () => {
  const toAttribute = name => 'data-' + name.replace(/[A-Z]/g, letter => '-' + letter.toLowerCase());
  const liveNames = new Set([
    ...[...liveSource.matchAll(/\bdata-live-[\w-]+/g)].map(m => m[0]),
    ...[...liveSource.matchAll(/dataset\.(live\w+)/g)].map(m => toAttribute(m[1])),
  ]);
  assert.ok(liveNames.has('data-live-chip') && liveNames.has('data-live-watch'));
  const written = source => new Set([...source.matchAll(/\.dataset\.(\w+)\s*=/g)].map(m => toAttribute(m[1])));
  const foreign = new Set([...written(editorSource), ...written(pickerSource), ...written(hubSource), ...written(lobbySource)]);
  const projected = new Set([...lobbySource.matchAll(/\bset\(\s*'([^']+)'/g)].flatMap(m => m[1].match(/\[data-[\w-]+\]/g) || []).map(s => s.slice(1, -1)));
  assert.deepEqual([...liveNames].filter(name => foreign.has(name) || projected.has(name)), []);
});

test('live code builds the DOM with textContent only and styles through the stylesheet', () => {
  for (const source of [liveSource, dataSource]) assert.doesNotMatch(source, /innerHTML|outerHTML|insertAdjacentHTML|setAttribute\(\s*'style'/);
  const live = html.slice(html.indexOf('data-live-chip'), html.indexOf('id="lobby-settings"'));
  assert.doesNotMatch(live, /style=/);
  assert.match(css, /\.live-panel\{position:absolute;right:2%;top:17%;bottom:33%;width:23%/, 'panel keeps the news panel edges');
});
