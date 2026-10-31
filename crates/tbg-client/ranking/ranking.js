//! Public player profiles: read-only API fields become text nodes and encoded links.
//! Labels come from the shared catalog (P64); API codes (Faction, W/L/D,
//! reason) stay machine values and only their presentation is translated.
'use strict';

const NAME_RE = /^[A-Za-z][A-Za-z0-9_]{2,15}$/;
const FACTIONS = ['Humans', 'Undead', 'Machines', 'Plants'];
const REASONS = { StrongholdDestroyed: 'stronghold_destroyed', TimeUp: 'time_up', Forfeit: 'forfeit' };
const RESULT_CLASS = { W: 'win', L: 'loss', D: '' };
const I = () => globalThis.TBG_I18N;
const t = (key, args) => I().t(key, args);

function el(tag, text, cls) {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = String(text);
  if (cls) e.className = cls;
  return e;
}

// The status is a key plus arguments so a language switch can re-read it.
let status = ['ranking.web.loading_profile', {}, false];
function setStatus(key, args, error) {
  status = [key, args || {}, Boolean(error)];
  const s = document.getElementById('status');
  s.textContent = key ? t(key, args) : '';
  s.classList.toggle('error', Boolean(error));
}

function failure(code) {
  const error = new Error('profile request ' + code);
  error.code = code;
  return error;
}

// null for 404; throws a stable failure code for everything else.
async function getJson(path) {
  let res;
  try {
    res = await fetch(path, { headers: { Accept: 'application/json' } });
  } catch (_) {
    throw failure('offline');
  }
  if (res.status === 404) return null;
  if (res.status === 429) throw failure('rate_limited');
  if (!res.ok) throw failure('unavailable');
  return res.json();
}

function playerLink(name) {
  const a = el('a', name);
  a.href = 'player.html?name=' + encodeURIComponent(name);
  return a;
}

const pct = (n) => I().number(Number(n), 1) + '%';
const record = (p) => p.wins + '/' + p.losses + '/' + p.draws;
const factionName = (f) => FACTIONS.includes(f) ? t('faction.' + f.toLowerCase()) : t('ranking.reason.unknown');

function fillCard(p) {
  document.getElementById('title').textContent = p.name;
  document.title = t('profile.page_title', { name: p.name });
  const set = (id, text) => { document.getElementById(id).textContent = String(text); };
  set('rank', p.rank === null ? t('profile.unranked') : '#' + p.rank);
  set('rating', p.rating);
  set('peak', p.peak_rating);
  set('record', record(p));
  set('winrate', pct(p.win_rate));
  set('joined', date(p.joined));
  document.getElementById('card').hidden = false;
}

function fillFactions(p) {
  const list = document.getElementById('factions');
  list.replaceChildren();
  if (p.games === 0) return;
  for (const f of FACTIONS) {
    const share = Number(p.factions[f]) || 0;
    const li = el('li');
    const label = el(f === p.favorite_faction ? 'strong' : 'span', factionName(f));
    const bar = el('div', undefined, 'bar');
    bar.setAttribute('aria-hidden', 'true');
    // The canonical Faction name stays the CSS hook for the bar colour.
    const fill = el('span', undefined, f);
    fill.style.width = Math.max(0, Math.min(100, share)) + '%';
    bar.append(fill);
    li.append(label, bar, el('span', pct(share), 'num'));
    list.append(li);
  }
  document.getElementById('factions-box').hidden = false;
}

function duration(s) {
  return Math.floor(s / 60) + ':' + String(s % 60).padStart(2, '0');
}

// `YYYY-MM-DD` from the API, shown in the page language; any other shape
// is shown as given rather than guessed.
function date(value) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(value));
  if (!match) return String(value);
  const day = new Date(Date.UTC(Number(match[1]), Number(match[2]) - 1, Number(match[3])));
  return new Intl.DateTimeFormat(I().lang, { dateStyle: 'medium', timeZone: 'UTC' }).format(day);
}

function fillRecent(p) {
  const body = document.querySelector('#recent tbody');
  body.replaceChildren();
  if (p.recent.length === 0) return;
  for (const m of p.recent) {
    const known = m.result in RESULT_CLASS;
    const word = t(known ? 'ranking.result.' + m.result.toLowerCase() : 'ranking.result.unknown');
    const opp = el('td', undefined, 'name');
    opp.append(playerLink(m.opponent));
    const delta = m.delta > 0 ? '+' + m.delta : m.delta < 0 ? '−' + -m.delta : '0';
    const tr = el('tr');
    // Unknown or historical codes get a safe label, never the raw code.
    tr.append(el('td', date(m.date)), opp, el('td', word, known ? RESULT_CLASS[m.result] : ''), el('td', delta, 'num'),
      el('td', t('ranking.reason.' + (REASONS[m.reason] || 'unknown'))), el('td', duration(m.duration_s), 'num'));
    body.append(tr);
  }
  document.getElementById('recent-box').hidden = false;
}

let shown = null;
function render(p) {
  shown = p;
  fillCard(p);
  fillFactions(p);
  fillRecent(p);
}

async function player() {
  await I().ready;
  const name = new URLSearchParams(location.search).get('name') || '';
  const notFound = () => name ? setStatus('profile.not_found', { name }, true) : setStatus('profile.no_name', {}, true);
  if (!NAME_RE.test(name)) return notFound();
  let p;
  try {
    p = await getJson('/api/players/' + encodeURIComponent(name));
  } catch (e) {
    return setStatus('ranking.error.' + e.code, {}, true);
  }
  if (!p) return notFound();
  setStatus(p.games === 0 ? 'profile.no_matches' : '');
  render(p);
}

document.addEventListener('tbg:lang', () => {
  setStatus(...status);
  if (shown) render(shown);
});

player();
