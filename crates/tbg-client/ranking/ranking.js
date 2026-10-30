//! Public player profiles: read-only API fields become text nodes and encoded links.
'use strict';

const NAME_RE = /^[A-Za-z][A-Za-z0-9_]{2,15}$/;
const FACTIONS = ['Humans', 'Undead', 'Machines', 'Plants'];
const REASONS = {
  StrongholdDestroyed: 'Stronghold destroyed',
  TimeUp: 'Time up',
  Forfeit: 'Forfeit',
};
const RESULTS = { W: ['Win', 'win'], L: ['Loss', 'loss'], D: ['Draw', ''] };

function el(tag, text, cls) {
  const e = document.createElement(tag);
  if (text !== undefined) e.textContent = String(text);
  if (cls) e.className = cls;
  return e;
}

function setStatus(text, error) {
  const s = document.getElementById('status');
  s.textContent = text;
  s.classList.toggle('error', Boolean(error));
}

// null for 404; throws a readable message for everything else.
async function getJson(path) {
  let res;
  try {
    res = await fetch(path, { headers: { Accept: 'application/json' } });
  } catch (_) {
    throw new Error('Cannot reach the server. Check your connection and reload.');
  }
  if (res.status === 404) return null;
  if (res.status === 429) throw new Error('Too many requests. Wait a minute and reload.');
  if (!res.ok) throw new Error('The ranking is unavailable right now.');
  return res.json();
}

function playerLink(name) {
  const a = el('a', name);
  a.href = 'player.html?name=' + encodeURIComponent(name);
  return a;
}

const pct = (n) => Number(n).toFixed(1) + '%';
const record = (p) => p.wins + '/' + p.losses + '/' + p.draws;

function fillCard(p) {
  document.getElementById('title').textContent = p.name;
  document.title = p.name + ' — Tupi Battle Grounds';
  const set = (id, text) => { document.getElementById(id).textContent = String(text); };
  set('rank', p.rank === null ? 'Unranked' : '#' + p.rank);
  set('rating', p.rating);
  set('peak', p.peak_rating);
  set('record', record(p));
  set('winrate', pct(p.win_rate));
  set('joined', p.joined);
  document.getElementById('card').hidden = false;
}

function fillFactions(p) {
  if (p.games === 0) return;
  const list = document.getElementById('factions');
  for (const f of FACTIONS) {
    const share = Number(p.factions[f]) || 0;
    const li = el('li');
    const label = el(f === p.favorite_faction ? 'strong' : 'span', f);
    const bar = el('div', undefined, 'bar');
    bar.setAttribute('aria-hidden', 'true');
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

function fillRecent(p) {
  if (p.recent.length === 0) return;
  const body = document.querySelector('#recent tbody');
  for (const m of p.recent) {
    const [word, cls] = RESULTS[m.result] || ['?', ''];
    const opp = el('td', undefined, 'name');
    opp.append(playerLink(m.opponent));
    const delta = m.delta > 0 ? '+' + m.delta : m.delta < 0 ? '−' + -m.delta : '0';
    const tr = el('tr');
    tr.append(el('td', m.date), opp, el('td', word, cls), el('td', delta, 'num'),
      el('td', REASONS[m.reason] || m.reason), el('td', duration(m.duration_s), 'num'));
    body.append(tr);
  }
  document.getElementById('recent-box').hidden = false;
}

async function player() {
  const name = new URLSearchParams(location.search).get('name') || '';
  const notFound = () => setStatus(name ? 'No player named “' + name + '”.' : 'No player name given.', true);
  if (!NAME_RE.test(name)) return notFound();
  let p;
  try {
    p = await getJson('/api/players/' + encodeURIComponent(name));
  } catch (e) {
    return setStatus(e.message, true);
  }
  if (!p) return notFound();
  setStatus(p.games === 0 ? 'No ranked matches yet.' : '');
  fillCard(p);
  fillFactions(p);
  fillRecent(p);
}

player();
