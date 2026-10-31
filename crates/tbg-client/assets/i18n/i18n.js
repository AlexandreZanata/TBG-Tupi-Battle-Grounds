// Client language runtime (P41, D3/D4). No framework: one flat catalog per
// language, property reads only, DOM touched on boot and on switch.
(() => {
  'use strict';
  const MAP = { en: 'en', 'pt-BR': 'pt-BR' };
  const root = () => document.getElementById('tbg-lobby');
  let lang = document.documentElement.lang === 'pt-BR' ? 'pt-BR' : 'en';
  let catalog = {};
  let fallback = {};
  // Async switches may finish out of order; only the newest one applies.
  let generation = 0;
  const formats = {};

  // Standalone pages (ranking/) set `data-i18n-base` through resolve.js;
  // the game serves the catalogs next to its own page.
  const base = () => document.documentElement.dataset?.i18nBase || 'assets/i18n/';
  const url = code => new URL(base() + code + '.json', document.baseURI).toString();
  // A page may name its own `<title>` key (`data-i18n-page-title`).
  const pageTitle = () => t(document.documentElement.dataset?.i18nPageTitle || 'meta.title');

  async function load(code) {
    try {
      const res = await fetch(url(code), { cache: 'no-cache' });
      if (!res.ok) throw new Error('i18n ' + res.status);
      return await res.json();
    } catch (_) {
      if (code !== 'en') {
        try {
          const res = await fetch(url('en'), { cache: 'no-cache' });
          if (res.ok) return await res.json();
        } catch (_e) { /* fall through to the key itself */ }
      }
      return {};
    }
  }

  function lookup(key, table) {
    const value = table[key];
    return typeof value === 'string' ? value : undefined;
  }

  function t(key, args) {
    let text = lookup(key, catalog) ?? lookup(key, fallback) ?? key;
    if (args) {
      for (const name of Object.keys(args)) {
        text = text.split('{' + name + '}').join(String(args[name]));
      }
    }
    return text;
  }

  const has = key => lookup(key, catalog) !== undefined || lookup(key, fallback) !== undefined;

  // pt-BR uses `one` for 0 and 1 (CLDR i = 0,1); English for 1 only.
  // `args` may override `{n}` with the formatted count and add placeholders.
  function plural(key, n, args) {
    const one = lang === 'pt-BR' ? n === 0 || n === 1 : n === 1;
    return t(key + (one ? '.one' : '.other'), Object.assign({ n }, args));
  }

  // Locale-aware number with `minPlaces`..`places` decimals; one formatter
  // per locale and precision, so mixed precisions never share a cache slot.
  function number(value, places = 0, minPlaces = places) {
    const tag = lang === 'pt-BR' ? 'pt-BR' : 'en';
    const id = tag + '|' + minPlaces + '|' + places;
    formats[id] ??= new Intl.NumberFormat(tag, {
      minimumFractionDigits: minPlaces,
      maximumFractionDigits: places,
    });
    return formats[id].format(value);
  }

  // One pass over ~200 nodes; text writes only land when the value differs.
  function apply(scope) {
    const host = scope || document;
    host.querySelectorAll('[data-i18n]').forEach(node => {
      const text = t(node.dataset.i18n);
      if (node.textContent !== text) node.textContent = text;
    });
    for (const [attr, data] of [
      ['aria-label', 'i18nAriaLabel'],
      ['title', 'i18nTitle'],
      ['placeholder', 'i18nPlaceholder'],
      ['alt', 'i18nAlt'],
    ]) {
      host.querySelectorAll('[data-i18n-' + attr + ']').forEach(node => {
        const text = t(node.dataset[data]);
        if (node.getAttribute(attr) !== text) node.setAttribute(attr, text);
      });
    }
  }

  function refreshSelector() {
    document.querySelectorAll('[data-lang]').forEach(button => {
      button.setAttribute('aria-pressed', String(button.dataset.lang === lang));
    });
  }

  async function setLang(code) {
    if (!MAP[code]) return;
    // Every call is the newest choice, even one that keeps the current
    // language while an older switch is still loading.
    const mine = ++generation;
    if (code !== lang) {
      const next = code === 'en' ? fallback : await load(code);
      if (mine !== generation) return;
      lang = code;
      catalog = next;
      try { localStorage.setItem('tbg.lang', code); } catch (_) { /* private mode */ }
      document.documentElement.lang = code;
      apply(document);
      document.title = pageTitle();
    }
    refreshSelector();
    document.dispatchEvent(new CustomEvent('tbg:lang', { detail: { lang } }));
    const lobby = root();
    if (lobby) lobby.dataset.action = 'lang:' + code;
  }

  document.addEventListener('click', event => {
    const button = event.target.closest('[data-lang]');
    if (!button || button.dataset.lang === lang) return;
    event.preventDefault();
    setLang(button.dataset.lang);
  });

  const ready = (async () => {
    fallback = await load('en');
    catalog = lang === 'en' ? fallback : await load(lang);
    apply(document);
    document.title = pageTitle();
    refreshSelector();
  })();

  window.TBG_I18N = { get lang() { return lang; }, t, has, plural, number, apply, ready, setLang };
})();
