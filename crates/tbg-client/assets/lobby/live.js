//! Lobby live surfaces (P47): header online chip, Live battles panel and its See-all dialog. Read-only API, no account needed; text goes through textContent only.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  if (!root) return;
  const D = globalThis.TbgLiveData;
  const T = (key, args) => window.TBG_I18N ? window.TBG_I18N.t(key, args) : key;
  // The API is cheap, but one request per visible lobby every 5 s is the budget (P47 section 10).
  const POLL_MS = 5000;
  const q = selector => root.querySelector(selector);
  const chip = q('[data-live-chip]'), title = q('[data-live-title]'), rows = q('[data-live-rows]');
  const status = q('[data-live-status]'), play = q('[data-live-play]'), seeAll = q('[data-live-see-all]');
  const allRows = q('[data-live-all]'), allEmpty = q('[data-live-all-empty]');
  const accounts = q('[data-live-accounts]'), accountsTitle = q('[data-live-accounts-title]'), accountsStatus = q('[data-live-accounts-status]');
  const copyStatus = q('[data-live-copy-status]'), manual = q('[data-live-link]');
  let battles = null, accountList = null, battlesFailed = false, accountsFailed = false;
  let timer = 0, inFlight = false, ready = !window.TBG_I18N;

  const el = (tag, className, text) => {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = String(text);
    return node;
  };

  // Ratings are numbers; a guest seat has no rating and says so in words.
  function seat(player, side) {
    const block = el('span', 'live-seat live-seat-' + side);
    block.append(el('strong', '', player.name), player.rating === null ? el('small', 'live-guest', T('live.guest')) : el('small', 'live-rating', player.rating));
    return block;
  }

  function bar(bp, side) {
    const track = el('span', 'live-bar live-bar-' + side), fill = el('i');
    fill.style.width = D.percent(bp) + '%';
    track.append(fill);
    return track;
  }

  function battleRow(match, featured, withCopy) {
    const item = el('li', 'live-row' + (featured ? ' is-featured' : ''));
    const names = el('div', 'live-names');
    const arenaKey = D.arenaKey(match.arena);
    const badge = el('span', 'live-badge live-' + match.mode, T('live.mode.' + match.mode));
    names.append(seat(match.players[0], 'a'), seat(match.players[1], 'b'), badge);
    const meta = el('span', 'live-meta', [arenaKey ? T(arenaKey) : '', D.clock(match.elapsed_s), T('spectate.watching_badge', {count: match.spectators})].filter(Boolean).join(' · '));
    const hp = el('span', 'live-hp');
    hp.append(bar(match.hp_bp[0], 'a'), bar(match.hp_bp[1], 'b'));
    const actions = el('span', 'live-actions');
    const watch = el('button', 'live-watch', T('live.watch'));
    watch.dataset.liveWatch = match.id;
    actions.append(watch);
    if (withCopy) {
      const copy = el('button', 'live-copy', T('spectate.copy_link'));
      copy.dataset.liveCopy = match.id;
      actions.append(copy);
    }
    item.append(names, meta, hp, actions);
    return item;
  }

  function accountRow(row) {
    const item = el('li', 'live-account');
    const label = Object.hasOwn(D.STATUSES, row.status) ? T(D.STATUSES[row.status]) : '';
    item.append(el('strong', '', row.name), el('small', 'live-rating', row.rating ?? ''), el('span', 'live-state', label));
    return item;
  }

  // Header chip: a placeholder until the first good answer, never a fake zero.
  function renderChip() {
    chip.textContent = battles ? T('live.chip', {online: battles.online.players, battle: battles.online.in_battle}) : T('live.chip_loading');
  }

  function renderPanel() {
    title.textContent = battles ? T('live.panel.title', {n: battles.matches.length}) : T('live.dialog.title');
    rows.replaceChildren();
    play.hidden = true;
    status.textContent = '';
    if (!battles) {
      status.textContent = battlesFailed ? T('live.unavailable') : T('live.loading');
      return;
    }
    if (!battles.matches.length) {
      status.textContent = T('live.none');
      play.hidden = false;
      return;
    }
    D.featuredFirst(battles.matches, battles.featured).slice(0, 4).forEach(match => {
      rows.append(battleRow(match, match.id === battles.featured, false));
    });
  }

  function renderBattleList() {
    allRows.replaceChildren();
    if (!battles) return;
    const matches = D.featuredFirst(battles.matches, battles.featured);
    matches.forEach(match => allRows.append(battleRow(match, match.id === battles.featured, true)));
    allEmpty.hidden = matches.length > 0;
  }

  function renderAccounts() {
    accounts.replaceChildren();
    accountsTitle.textContent = T('live.accounts', {count: accountList ? accountList.total : '…'});
    if (!accountList) {
      accountsStatus.textContent = accountsFailed ? T('live.unavailable') : T('live.loading');
      return;
    }
    accountsStatus.textContent = accountList.rows.length ? '' : T('live.accounts_none');
    accountList.rows.forEach(row => accounts.append(accountRow(row)));
  }

  function render() {
    renderChip();
    renderPanel();
    renderBattleList();
  }

  // Keep the last good render on any failure; a failure before the first success shows the placeholder.
  async function fetchJson(url, parse) {
    try {
      const response = await fetch(url, {headers: {Accept: 'application/json'}, cache: 'no-store'});
      return response.ok ? parse(await response.json()) : null;
    } catch (_) {
      return null;
    }
  }

  async function poll() {
    if (inFlight) return;
    inFlight = true;
    const body = await fetchJson('/api/live', D.parseLive);
    inFlight = false;
    battlesFailed = !body && !battles;
    if (body) battles = body;
    render();
  }

  const visible = () => !root.hidden && document.visibilityState !== 'hidden';

  // Polling runs only while the lobby is on screen and the tab is visible; coming back fetches at once.
  function sync() {
    if (!ready) return;
    if (!visible()) {
      clearInterval(timer);
      timer = 0;
      return;
    }
    if (!timer) {
      timer = setInterval(poll, POLL_MS);
      poll();
    }
  }

  async function loadAccounts() {
    const body = await fetchJson('/api/online?offset=0&limit=100', D.parseOnline);
    accountsFailed = !body && !accountList;
    if (body) accountList = body;
    renderAccounts();
  }

  // Same URL with ?watch= added; other parameters (such as ?lang=) stay.
  function watchMatch(id) {
    if (!D.isId(id)) return;
    const url = new URL(location.href);
    url.searchParams.set('watch', id);
    location.assign(url.href);
  }

  // Clipboard first; when it is refused, the link is selected in a field so the player can copy it.
  async function copyLink(id) {
    if (!D.isId(id)) return;
    const link = D.watchLink(id, location.origin, location.pathname);
    manual.value = link;
    let copied = false;
    try {
      await navigator.clipboard.writeText(link);
      copied = true;
    } catch (_) {
      manual.hidden = false;
      manual.select();
      try { copied = document.execCommand('copy'); } catch (_e) { copied = false; }
    }
    manual.hidden = copied;
    if (!copied) manual.select();
    copyStatus.textContent = copied ? T('live.copied') : T('live.copy_manual');
  }

  root.addEventListener('click', event => {
    const watch = event.target.closest('[data-live-watch]');
    if (watch) return watchMatch(watch.dataset.liveWatch);
    const copy = event.target.closest('[data-live-copy]');
    if (copy) copyLink(copy.dataset.liveCopy);
  });
  // Quick match is the existing Casual button: the same click path the player would use.
  play.addEventListener('click', () => q('button[data-action="quick"]')?.click());
  seeAll.addEventListener('click', () => {
    copyStatus.textContent = '';
    manual.hidden = true;
    loadAccounts();
  });
  document.addEventListener('visibilitychange', sync);
  document.addEventListener('tbg:lang', () => { render(); renderAccounts(); });
  new MutationObserver(sync).observe(root, {attributes: true, attributeFilter: ['hidden']});
  if (window.TBG_I18N) {
    window.TBG_I18N.ready.then(() => {
      ready = true;
      render();
      renderAccounts();
      sync();
    });
  } else {
    render();
    sync();
  }
})();
