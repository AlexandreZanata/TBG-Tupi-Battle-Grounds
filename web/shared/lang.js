// Language choice shared by both sites: ?lang=, then the saved preference, then the browser.
// Only the language code is stored; storage failures (private mode, blocked) are ignored.

export const LANGS = ['pt-BR', 'en'];
const KEY = 'tbg.web.lang';

function normalize(value) {
  if (!value) return null;
  const lower = String(value).toLowerCase();
  if (lower.startsWith('pt')) return 'pt-BR';
  if (lower.startsWith('en')) return 'en';
  return null;
}

function stored() {
  try {
    return normalize(localStorage.getItem(KEY));
  } catch {
    return null;
  }
}

export function initialLang(fallback = 'pt-BR') {
  const fromUrl = normalize(new URLSearchParams(location.search).get('lang'));
  if (fromUrl) return fromUrl;
  const saved = stored();
  if (saved) return saved;
  for (const candidate of navigator.languages || [navigator.language]) {
    const lang = normalize(candidate);
    if (lang) return lang;
  }
  return fallback;
}

export function saveLang(lang) {
  try {
    localStorage.setItem(KEY, lang);
  } catch {
    // The choice still applies to this visit.
  }
}

/** Pick the string for `lang` from a { en, 'pt-BR' } record. */
export function pick(record, lang) {
  if (record === null || record === undefined) return '';
  if (typeof record === 'string') return record;
  return record[lang] ?? record['pt-BR'] ?? record.en ?? '';
}
