// Desktop shell projection. Only textContent receives account or API data.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  if (!root) return;
  // P41: every player-facing string comes from the catalog; the key itself is
  // the last-resort fallback so a missing catalog never throws.
  const T = (key, args) => window.TBG_I18N ? window.TBG_I18N.t(key, args) : key;
  const set = (selector, text) => root.querySelectorAll(selector).forEach(node => {
    if (node.textContent !== String(text)) node.textContent = String(text);
  });
  const dialogs = [...root.querySelectorAll('dialog')];
  let previousScreen = '', previousPlayer = '', previousRating = '', statsRequest, artStarted = false;
  async function loadArt() {
    if (artStarted) return;
    artStarted = true;
    // This manifest lets the owner substitute WebP illustrations without changing code.
    let sources = {};
    try { const response = await fetch('assets/lobby/art.json'); if (response.ok) sources = await response.json(); } catch (_) { /* SVG defaults work offline. */ }
    root.querySelectorAll('[data-art]').forEach(image => {
      const source = sources[image.dataset.art];
      image.src = typeof source === 'string' && source.startsWith('assets/lobby/') ? source : image.dataset.art === 'profile' ? 'assets/lobby/profile-v2.webp' : 'assets/lobby/' + image.dataset.art + '.svg';
      image.decoding = 'async';
    });
    root.querySelectorAll('use[data-icon]').forEach(icon => icon.setAttribute('href', 'assets/lobby/icons.svg#' + icon.dataset.icon));
  }
  const send = action => {
    if (action === 'ranking' && globalThis.TbgRanking) { globalThis.TbgRanking.open(); return; }
    if (action === 'menu' && globalThis.TbgRanking) { globalThis.TbgRanking.close(); return; }
    if (!['sound','copy-key','download-key','logout'].includes(action)) globalThis.TbgRanking?.close();
    root.dataset.action = action;
    if (action !== 'sound' && action !== 'copy-key' && action !== 'download-key' && action !== 'logout') dialogs.forEach(dialog => dialog.close());
  };
  root.addEventListener('click', event => {
    const button = event.target.closest('button');
    if (!button || button.disabled) return;
    if (button.dataset.action) send(button.dataset.action);
    if (button.dataset.dialog) document.getElementById('lobby-' + button.dataset.dialog).showModal();
    if (button.hasAttribute('data-close')) button.closest('dialog').close();
  });
  document.getElementById('lobby-join').addEventListener('submit', event => {
    event.preventDefault();
    const code = document.getElementById('lobby-room').value.trim().toUpperCase();
    if (/^[A-Z0-9]{5}$/.test(code)) send('join:' + code);
  });
  document.addEventListener('keydown', event => {
    if (event.key !== 'Escape' || root.dataset.screen === 'off') return;
    const open = dialogs.find(dialog => dialog.open);
    if (open) { event.preventDefault(); open.close(); return; }
    if (root.dataset.lobbyView === 'ranking') { event.preventDefault(); globalThis.TbgRanking.close(); return; }
    const close = {ranking:'close-ranking',key:'close-panel'}[root.dataset.screen];
    if (close) { event.preventDefault(); send(close); }
    if (root.dataset.screen === 'form') { event.preventDefault(); document.getElementById('tbg-form-cancel').click(); }
  });
  async function stats(name) {
    if (statsRequest) statsRequest.abort();
    set('[data-global-rank]', T('lobby.profile.unranked'));
    set('[data-wins], [data-losses], [data-win-rate]', '—');
    set('[data-record-note]', name ? T('lobby.profile.ranked_note_member') : T('lobby.profile.ranked_note_guest'));
    set('[data-stats-status]', name ? T('lobby.profile.stats_loading') : T('lobby.profile.stats_guest'));
    if (!name) return;
    const request = new AbortController();
    statsRequest = request;
    try {
      const response = await fetch('/api/players/' + encodeURIComponent(name), {signal:request.signal,headers:{Accept:'application/json'}});
      if (!response.ok) throw new Error('unavailable');
      const player = await response.json();
      if (root.dataset.player !== name) return;
      set('[data-global-rank]', Number.isInteger(player.rank) ? T('lobby.profile.global', {rank: player.rank}) : T('lobby.profile.unranked'));
      set('[data-wins]', player.wins);
      set('[data-losses]', player.losses);
      set('[data-win-rate]', Number(player.win_rate).toFixed(0) + '%');
      set('[data-record-note]', player.games ? window.TBG_I18N.plural('lobby.profile.ranked_matches', player.games) : T('lobby.profile.first_match'));
      set('[data-stats-status]', player.games ? T('lobby.profile.stats_record', {draws: player.draws}) : T('lobby.profile.stats_enter'));
    } catch (error) {
      if (error.name !== 'AbortError') set('[data-stats-status]', T('lobby.profile.stats_unavailable'));
    }
  }
  // Story index doubles as the catalog id (0-based indexed family); resolved
  // at render time so a language switch re-renders.
  const NEWS_COUNT = 3;
  const newsTitle = i => T('lobby.news.' + i + '.title');
  const newsBody = i => T('lobby.news.' + i + '.body');
  let story = 0;
  root.querySelectorAll('[data-news]').forEach(button => button.addEventListener('click', () => {
    story = (story + Number(button.dataset.news) + NEWS_COUNT) % NEWS_COUNT;
    set('[data-news-title]', newsTitle(story)); set('[data-news-body]', newsBody(story)); set('[data-news-page]', (story + 1) + ' / ' + NEWS_COUNT);
  }));
  let initialPanelOpened = false;
  function initialPanel() {
    if (initialPanelOpened || root.hidden || root.dataset.screen !== 'lobby') return;
    initialPanelOpened = true;
    const panel = new URLSearchParams(location.search).get('panel');
    if (panel === 'ranking') { globalThis.TbgRanking?.open(); return; }
    const selector = {arcanas:'[data-arcana-open]',stronghold:'[data-tower-open]',settings:'[data-dialog="settings"]'}[panel];
    if (selector) root.querySelector(selector).click();
  }
  function render() {
    const name = root.dataset.player || '';
    root.querySelectorAll('[data-member]').forEach(node => { node.hidden = !name; });
    root.querySelectorAll('[data-guest]').forEach(node => { node.hidden = Boolean(name); });
    set('[data-player-name]', name || T('lobby.guest.name'));
    set('[data-rating]', root.dataset.rating || '—');
    set('[data-sound]', root.dataset.muted === 'true' ? T('lobby.sound.off') : T('lobby.sound.on'));
    set('[data-ranked-hint]', name ? T('lobby.mode.ranked_hint_member') : T('lobby.mode.ranked_hint_guest'));
    root.querySelector('[data-ranked]').dataset.action = name ? 'ranked' : 'register';
    set('[data-news-title]', newsTitle(story)); set('[data-news-body]', newsBody(story));
    if (root.hidden) { dialogs.forEach(dialog => dialog.close()); globalThis.TbgRanking?.close(); }
    if (root.dataset.screen === 'lobby') { requestAnimationFrame(loadArt); requestAnimationFrame(initialPanel); }
    if (name !== previousPlayer || root.dataset.rating !== previousRating || (root.dataset.screen === 'lobby' && previousScreen !== 'lobby')) stats(name);
    previousPlayer = name; previousRating = root.dataset.rating; previousScreen = root.dataset.screen;
  }
  new MutationObserver(render).observe(root,{attributes:true, attributeFilter:['data-player','data-rating','data-muted','data-screen']});
  // Invariant 4: the lobby stays hidden until its text is in the chosen
  // language; a switch re-renders through the same path.
  if (window.TBG_I18N) {
    document.addEventListener('tbg:lang', render);
    window.TBG_I18N.ready.then(() => {
      render();
      root.dataset.ready = 'true';
    });
  } else {
    render();
    root.dataset.ready = 'true';
  }
})();
