// Loads the real assets/i18n/i18n.js into Node with a minimal document and a
// fetch that serves the shipped catalogs, optionally delayed per language.
// Shared by the localization tests (P64); not a test file itself.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

const dir = path.resolve(__dirname, '../assets/i18n');
const catalogs = {
  en: JSON.parse(fs.readFileSync(path.join(dir, 'en.json'), 'utf8')),
  'pt-BR': JSON.parse(fs.readFileSync(path.join(dir, 'pt-BR.json'), 'utf8')),
};

async function boot(lang = 'en', delays = {}) {
  const events = [];
  const document = {
    documentElement: { lang },
    baseURI: 'https://tbg.test/',
    title: '',
    getElementById: () => null,
    querySelectorAll: () => [],
    addEventListener: () => {},
    dispatchEvent: event => events.push(event.detail),
  };
  const fetch = async url => {
    const code = /i18n\/(.+)\.json/.exec(url)[1];
    await new Promise(resolve => setTimeout(resolve, delays[code] || 0));
    return { ok: true, json: async () => catalogs[code] };
  };
  const context = { document, fetch, URL, Intl, setTimeout, localStorage: { setItem() {} }, CustomEvent: class { constructor(type, init) { this.detail = init.detail; } } };
  context.window = context; context.globalThis = context;
  vm.runInNewContext(fs.readFileSync(path.join(dir, 'i18n.js'), 'utf8'), context);
  await context.TBG_I18N.ready;
  return { i18n: context.TBG_I18N, document, events };
}

module.exports = { boot, catalogs };
