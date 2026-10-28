// Desktop shell projection. Only textContent receives account or API data.
(() => {
  'use strict';
  const root = document.getElementById('tbg-lobby');
  if (!root) return;
  const set = (selector, text) => root.querySelectorAll(selector).forEach(node => {
    if (node.textContent !== String(text)) node.textContent = String(text);
  });
  function render() {
    const name = root.dataset.player || '';
    root.querySelectorAll('[data-member]').forEach(node => { node.hidden = !name; });
    root.querySelectorAll('[data-guest]').forEach(node => { node.hidden = Boolean(name); });
    set('[data-player-name]', name || 'Guest');
    set('[data-rating]', root.dataset.rating || '—');
    set('[data-sound]', root.dataset.muted === 'true' ? 'OFF' : 'ON');
    set('[data-ranked-hint]', name ? 'Ranked · Compete for rating' : 'Locked · Create account to play Ranked');
    root.querySelector('[data-ranked]').dataset.action = name ? 'ranked' : 'register';
  }
  new MutationObserver(render).observe(root, {attributes:true, attributeFilter:['data-player','data-rating','data-muted','data-screen']});
  root.dataset.ready = 'true';
  render();
})();
