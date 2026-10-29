// Battle presentation reads the client projection; purchases remain PlayerIntentions.
(() => {
  'use strict';
  const root = document.getElementById('tbg-battle');
  if (!root) return;
  const set = (selector, text) => root.querySelectorAll(selector).forEach(node => { if (node.textContent !== String(text)) node.textContent = String(text); });
  const number = (key) => Number(root.dataset[key]) || 0;
  const send = action => { if (!root.hidden) root.dataset.action = action; };
  const cards = [...root.querySelectorAll('.battle-card')];
  const percent = (value, max) => Math.max(0, Math.min(100, value / Math.max(1, max) * 100));
  root.querySelector('[data-battle-action="surrender"]').addEventListener('click', () => send('surrender'));
  cards.forEach((card, i) => {
    let held = false, timer;
    card.addEventListener('pointerdown', event => { if (event.button !== 0 || card.disabled) return; held = false; timer = setTimeout(() => { held = true; send('inspect:' + i); }, 500); });
    for (const event of ['pointerup','pointercancel','pointerleave']) card.addEventListener(event, () => clearTimeout(timer));
    card.addEventListener('click', () => { if (!held && card.dataset.available === 'true') send('buy:' + i); held = false; });
    card.addEventListener('contextmenu', event => { event.preventDefault(); clearTimeout(timer); held = true; send('inspect:' + i); });
  });
  document.addEventListener('keydown', event => { if (!root.hidden && event.key === 'Escape') send('close-evolve'); });
  const slug = name => name.toLowerCase().replace(/[^a-z0-9]/g,'');
  function health(selector, hp, max, label) {
    const bar = root.querySelector(selector);
    bar.querySelector('.vitality-fill').style.width = percent(hp,max) + '%';
    bar.setAttribute('aria-valuenow', hp); bar.setAttribute('aria-valuemin','0'); bar.setAttribute('aria-valuemax',max); bar.setAttribute('aria-label',label + ' Stronghold HP');
    bar.querySelector('strong').textContent = hp + ' / ' + max;
  }
  function render() {
    const s = root.dataset;
    set('[data-battle-you]', s.you || 'You'); set('[data-battle-opponent]', s.opponent || 'Opponent');
    root.querySelector('[data-battle-you]').title = s.you || 'You'; root.querySelector('[data-battle-opponent]').title = s.opponent || 'Opponent';
    set('[data-battle-clock]', s.clock || '3:00');
    root.querySelector('[data-battle-ranked]').hidden = s.ranked !== 'true';
    health('.vitality.ally', number('hpOwn'), number('hpMax'), s.you || 'Your');
    health('.vitality.enemy', number('hpEnemy'), number('hpMax'), s.opponent || 'Enemy');
    const energy = number('energy') / Math.max(1,number('energyScale'));
    const cap = number('energyMax') / Math.max(1,number('energyScale'));
    const readable = value => Number.isInteger(value) ? String(value) : value.toFixed(1);
    set('[data-battle-energy]', readable(energy) + ' / ' + readable(cap));
    root.querySelector('[data-battle-energy-fill]').style.width = percent(number('energy'),number('energyMax')) + '%';
    const track = root.querySelector('.energy-track'); track.setAttribute('aria-valuenow', energy); track.setAttribute('aria-valuemax',cap); track.setAttribute('aria-valuemin','0');
    set('[data-battle-army]', s.army || '0 / 8');
    root.style.setProperty('--rotation', percent(number('rotation'),1) + '%');
    const synergy = root.querySelector('[data-battle-synergy]'); synergy.hidden = !s.synergy; set('[data-battle-synergy]',s.synergy || '');
    cards.forEach((card,i) => {
      const [kind,name,role,cost,slots,available,pending] = (root.getAttribute('data-offer-'+i) || '').split('|');
      if (!kind) return;
      const image = card.querySelector('img');
      if (image.dataset.kind !== kind) { image.dataset.kind = kind; image.src = 'assets/battle/portraits/' + slug(kind) + '.webp'; }
      card.querySelector('.champion-name').textContent = name;
      const roleLabel = role === 'Dps' ? 'DPS' : role;
      card.querySelector('.champion-role').textContent = roleLabel + ' · ' + slots + (slots === '1' ? ' slot' : ' slots');
      card.querySelector('.card-cost b').textContent = cost;
      card.querySelector('.card-slots').textContent = '●'.repeat(Math.min(2,Number(slots)||1));
      card.dataset.available = available; card.disabled = false; card.setAttribute('aria-disabled',available === 'true' ? 'false' : 'true'); card.classList.toggle('unaffordable',available !== 'true'); card.classList.toggle('pending',pending === 'true'); card.setAttribute('aria-busy',pending === 'true' ? 'true' : 'false');
      card.setAttribute('aria-label',name + ', '+roleLabel+', '+slots+(slots === '1' ? ' slot, ' : ' slots, ')+cost+' Energy'); card.title = name + ' · '+cost+' Energy. Hold or right-click for details.';
    });
    const surrender = root.querySelector('.battle-surrender'); surrender.dataset.state = s.surrender || 'ready'; surrender.disabled = s.surrender === 'sent';
    set('[data-battle-surrender]',s.surrender === 'sent' ? 'Surrender sent' : s.surrender === 'armed' ? 'Confirm surrender' : 'Surrender');
    surrender.title = s.surrender === 'armed' ? (s.ranked === 'true' ? 'Confirm: this counts as a Ranked loss' : 'Click again to surrender') : 'Click twice to surrender';
  }
  new MutationObserver(render).observe(root,{attributes:true,attributeFilter:['data-hp-own','data-hp-enemy','data-hp-max','data-you','data-opponent','data-ranked','data-clock','data-energy','data-energy-max','data-energy-scale','data-army','data-rotation','data-synergy','data-surrender','data-offer-0','data-offer-1','data-offer-2']});
  root.dataset.ready = 'true';
})();
