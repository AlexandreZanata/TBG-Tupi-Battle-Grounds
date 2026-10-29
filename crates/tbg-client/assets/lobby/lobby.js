// Desktop shell projection. Only textContent receives account or API data.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  if (!root) return;
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
      image.src = typeof source === 'string' && source.startsWith('assets/lobby/') ? source : 'assets/lobby/' + image.dataset.art + '.svg';
      image.decoding = 'async';
    });
    root.querySelectorAll('use[data-icon]').forEach(icon => icon.setAttribute('href', 'assets/lobby/icons.svg#' + icon.dataset.icon));
  }
  const send = action => {
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
    const close = {ranking:'close-ranking',key:'close-panel'}[root.dataset.screen];
    if (close) { event.preventDefault(); send(close); }
    if (root.dataset.screen === 'form') { event.preventDefault(); document.getElementById('tbg-form-cancel').click(); }
  });
  async function stats(name) {
    if (statsRequest) statsRequest.abort();
    set('[data-global-rank]', 'Unranked');
    set('[data-wins], [data-losses], [data-win-rate]', '—');
    set('[data-record-note]', name ? 'Your global standing' : 'Play Ranked to build your record');
    set('[data-stats-status]', name ? 'Loading your ranked record…' : 'Create an account to compete');
    if (!name) return;
    const request = new AbortController();
    statsRequest = request;
    try {
      const response = await fetch('/api/players/' + encodeURIComponent(name), {signal:request.signal,headers:{Accept:'application/json'}});
      if (!response.ok) throw new Error('unavailable');
      const player = await response.json();
      if (root.dataset.player !== name) return;
      set('[data-global-rank]', Number.isInteger(player.rank) ? '#' + player.rank + ' Global' : 'Unranked');
      set('[data-wins]', player.wins);
      set('[data-losses]', player.losses);
      set('[data-win-rate]', Number(player.win_rate).toFixed(0) + '%');
      set('[data-record-note]', player.games ? player.games + (player.games === 1 ? ' ranked match' : ' ranked matches') : 'Your first Ranked match awaits');
      set('[data-stats-status]', player.games ? 'Ranked record · ' + player.draws + ' draws' : 'Play Ranked to enter the global ranking');
    } catch (error) {
      if (error.name !== 'AbortError') set('[data-stats-status]', 'Ranked record unavailable · try again later');
    }
  }
  const news = [
    ['Welcome to Tupi Battle Grounds!', 'Real-time 1v1 strategy. Build your BattleLine, evolve your units, and destroy the enemy Stronghold.'],
    ['A rival is waiting', 'Play Casual, invite a friend, or create an account to join Ranked and earn your place in the global ranking.'],
    ['Master your BattleLine', 'Try Practice vs Bot. Spend Energy in the Shop, discover Evolution paths, and build your Synergy.'],
  ];
  let story = 0;
  root.querySelectorAll('[data-news]').forEach(button => button.addEventListener('click', () => {
    story = (story + Number(button.dataset.news) + news.length) % news.length;
    set('[data-news-title]', news[story][0]); set('[data-news-body]', news[story][1]); set('[data-news-page]', (story + 1) + ' / ' + news.length);
  }));
  function render() {
    const name = root.dataset.player || '';
    root.querySelectorAll('[data-member]').forEach(node => { node.hidden = !name; });
    root.querySelectorAll('[data-guest]').forEach(node => { node.hidden = Boolean(name); });
    set('[data-player-name]', name || 'Guest');
    set('[data-rating]', root.dataset.rating || '—');
    set('[data-sound]', root.dataset.muted === 'true' ? 'OFF' : 'ON');
    set('[data-ranked-hint]', name ? 'Ranked · Compete for rating' : 'Locked · Create account to play Ranked');
    root.querySelector('[data-ranked]').dataset.action = name ? 'ranked' : 'register';
    if (root.hidden) dialogs.forEach(dialog => dialog.close());
    if (root.dataset.screen === 'lobby') requestAnimationFrame(loadArt);
    if (name !== previousPlayer || root.dataset.rating !== previousRating || (root.dataset.screen === 'lobby' && previousScreen !== 'lobby')) stats(name);
    previousPlayer = name; previousRating = root.dataset.rating; previousScreen = root.dataset.screen;
  }
  new MutationObserver(render).observe(root, {attributes:true, attributeFilter:['data-player','data-rating','data-muted','data-screen']});
  root.dataset.ready = 'true';
  render();
})();
