// Page language: applies /assets/i18n/<lang>.json to [data-i18n] text and [data-i18n-attr] attributes.
// The HTML carries the pt-BR text as the no-JS fallback; web/scripts/check.py keeps it equal to pt-BR.json.
import { fetchJson } from '../shared/dom.js';
import { LANGS, initialLang, saveLang } from '../shared/lang.js';

let current = 'pt-BR';
let dict = {};
const listeners = new Set();

export const lang = () => current;

/** Translate `key`, replacing {name} placeholders from `vars`. Unknown keys return the key. */
export function t(key, vars = {}) {
  const template = dict[key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, name) => (name in vars ? String(vars[name]) : match));
}

export function onLangChange(listener) {
  listeners.add(listener);
}

function applyStatic(root = document) {
  for (const node of root.querySelectorAll('[data-i18n]')) {
    const value = dict[node.dataset.i18n];
    if (value !== undefined) node.textContent = value;
  }
  for (const node of root.querySelectorAll('[data-i18n-attr]')) {
    for (const pair of node.dataset.i18nAttr.split(';')) {
      const [attr, key] = pair.split('=');
      if (dict[key] !== undefined) node.setAttribute(attr.trim(), dict[key]);
    }
  }
  document.documentElement.lang = current;
  for (const button of document.querySelectorAll('[data-lang]')) {
    button.setAttribute('aria-pressed', String(button.dataset.lang === current));
  }
}

async function load(next) {
  dict = await fetchJson(`/assets/i18n/${next}.json`);
  current = next;
  applyStatic();
  for (const listener of listeners) listener(current);
}

export async function initI18n() {
  for (const button of document.querySelectorAll('[data-lang]')) {
    button.addEventListener('click', async () => {
      const next = button.dataset.lang;
      if (!LANGS.includes(next) || next === current) return;
      saveLang(next);
      await load(next);
    });
  }
  try {
    await load(initialLang());
  } catch {
    // The authored pt-BR page stays readable without the dictionary.
    current = 'pt-BR';
  }
}
