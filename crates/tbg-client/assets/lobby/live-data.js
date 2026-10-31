//! Shape checks and pure formatting for the lobby live surfaces (P47). No DOM, no network: live.js renders what this returns.
(() => {
  'use strict';
  // Server data is untrusted: a malformed row is dropped, a malformed payload is null (keep the last good render).
  const MODES = ['casual', 'ranked'];
  const STATUSES = {idle: 'live.status.idle', queued: 'live.status.queued', playing: 'live.status.playing', watching: 'live.status.watching'};
  const ARENAS = ['forest', 'tundra', 'volcanic', 'ruins', 'storm'];
  const MAX_MATCHES = 50;
  const isId = value => typeof value === 'string' && /^[0-9a-f]{16}$/.test(value);
  const count = value => Number.isInteger(value) && value >= 0;

  function parsePlayer(raw) {
    if (!raw || typeof raw.name !== 'string' || raw.name === '') return null;
    if (raw.rating !== null && !count(raw.rating)) return null;
    return {name: raw.name, rating: raw.rating};
  }

  function parseMatch(raw) {
    if (!raw || !isId(raw.id) || !MODES.includes(raw.mode) || !Array.isArray(raw.players) || raw.players.length !== 2) return null;
    const players = raw.players.map(parsePlayer);
    const hp = raw.hp_bp;
    if (players.includes(null) || !Array.isArray(hp) || hp.length !== 2 || !hp.every(count)) return null;
    return {
      id: raw.id,
      mode: raw.mode,
      arena: typeof raw.arena === 'string' ? raw.arena : '',
      players,
      elapsed_s: count(raw.elapsed_s) ? raw.elapsed_s : 0,
      hp_bp: hp,
      spectators: count(raw.spectators) ? raw.spectators : 0,
    };
  }

  // GET /api/live. Returns null when the shape is wrong; otherwise the valid rows only.
  function parseLive(body) {
    const online = body && body.online;
    if (!online || !['players', 'accounts', 'in_battle', 'watching'].every(key => count(online[key]))) return null;
    if (!Array.isArray(body.matches)) return null;
    const matches = body.matches.slice(0, MAX_MATCHES).map(parseMatch).filter(Boolean);
    const featured = isId(body.featured) ? body.featured : null;
    return {online, featured, matches};
  }

  // GET /api/online. Guests never appear here; the server total still counts what it lists.
  function parseOnline(body) {
    if (!body || !count(body.total) || !Array.isArray(body.rows)) return null;
    const rows = body.rows
      .filter(row => row && typeof row.name === 'string' && row.name !== '' && (row.rating === null || count(row.rating)))
      .map(row => ({name: row.name, rating: row.rating, status: typeof row.status === 'string' ? row.status : ''}));
    return {total: body.total, rows};
  }

  // The featured battle leads; the server already sends it first, this keeps the rule local and explicit.
  function featuredFirst(matches, featured) {
    const lead = matches.filter(match => match.id === featured);
    return [...lead, ...matches.filter(match => match.id !== featured)];
  }

  // Match time as m:ss; the server counts whole seconds.
  function clock(seconds) {
    const total = Math.max(0, Math.floor(seconds));
    return Math.floor(total / 60) + ':' + String(total % 60).padStart(2, '0');
  }

  // HP is basis points (0..10000); the bar width is a percentage.
  const percent = bp => Math.min(10000, Math.max(0, bp)) / 100;

  // Same string the Copy watch link button puts on the clipboard.
  const watchLink = (id, origin, pathname) => origin + pathname + '?watch=' + id;

  const arenaKey = slug => (ARENAS.includes(slug) ? 'arenas.' + slug + '.name' : '');

  globalThis.TbgLiveData = {parseLive, parseOnline, featuredFirst, clock, percent, watchLink, isId, arenaKey, STATUSES};
})();
