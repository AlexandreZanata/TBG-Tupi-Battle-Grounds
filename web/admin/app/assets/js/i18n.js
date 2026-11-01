// Admin preview language state: t() lookups over STRINGS and change notifications.
import { LANGS, initialLang, saveLang } from '../shared/lang.js';
import { STRINGS } from './strings.js';

let current = initialLang();
const listeners = new Set();

export const lang = () => current;

export function t(key, vars = {}) {
  const template = STRINGS[current][key] ?? STRINGS['pt-BR'][key] ?? key;
  return template.replace(/\{(\w+)\}/g, (match, name) => (name in vars ? String(vars[name]) : match));
}

export function setLang(next) {
  if (!LANGS.includes(next) || next === current) return;
  current = next;
  saveLang(next);
  document.documentElement.lang = next;
  for (const listener of listeners) listener(next);
}

export function onLangChange(listener) {
  listeners.add(listener);
}

document.documentElement.lang = current;
