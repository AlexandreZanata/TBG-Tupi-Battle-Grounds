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
  root.querySelector('[data-battle-action="close-inspect"]').addEventListener('click', () => send('close-inspect'));
  // War Council (BR-AG12–14): pick a card, then confirm; the choice is final.
  const council = { round: '', picked: null, sent: '' };
  const pause = root.querySelector('[data-pause]');
  const confirmButton = root.querySelector('[data-battle-action="confirm"]');
  root.querySelector('[data-battle-action="forgo"]').addEventListener('click', () => { council.sent = 'forgo'; send('forgo'); });
  confirmButton.addEventListener('click', () => { if (council.picked === null) return; council.sent = 'aug'; send('aug:' + council.picked); });
  root.querySelectorAll('[data-council-option]').forEach(card => card.addEventListener('click', () => { if (card.dataset.available === 'true') { council.picked = Number(card.dataset.councilOption); render(); } }));
  cards.forEach((card, i) => {
    let held = false, timer;
    card.addEventListener('pointerdown', event => { if (event.button !== 0 || card.disabled) return; held = false; timer = setTimeout(() => { held = true; send('inspect:' + i); }, 500); });
    for (const event of ['pointerup','pointercancel','pointerleave']) card.addEventListener(event, () => clearTimeout(timer));
    card.addEventListener('click', () => { if (!held && card.dataset.available === 'true') send('buy:' + i); held = false; });
    card.addEventListener('contextmenu', event => { event.preventDefault(); clearTimeout(timer); held = true; send('inspect:' + i); });
  });
  document.addEventListener('keydown', event => { if (!root.hidden && event.key === 'Escape') send('deselect'); });
  const slug = name => name.toLowerCase().replace(/[^a-z0-9]/g,'');
  // Units without a portrait yet (P16-B Nester, Rampart, Beetling) show none.
  const portrait = (image, kind) => {
    if (image.dataset.kind === kind) return;
    image.dataset.kind = kind; image.style.visibility = 'hidden';
    image.onload = () => { image.style.visibility = ''; };
    image.src = 'assets/battle/portraits/' + slug(kind) + '.webp';
  };
  // Fill moves at once; the pale trail follows after a beat, so a hit reads
  // as a chunk that drains away. A drop also flashes the bar.
  function bar(node, hp, max, label) {
    const width = percent(hp, max) + '%';
    const before = Number(node.dataset.hp);
    node.querySelector('.vitality-fill').style.width = width;
    node.querySelector('.vitality-trail').style.width = width;
    if (before > hp) { node.classList.remove('hurt'); void node.offsetWidth; node.classList.add('hurt'); }
    node.dataset.hp = hp;
    node.setAttribute('aria-valuenow', hp); node.setAttribute('aria-valuemin','0'); node.setAttribute('aria-valuemax',max); node.setAttribute('aria-label',label);
    const text = hp + ' / ' + max;
    if (node.querySelector('strong').textContent !== text) node.querySelector('strong').textContent = text;
  }
  const health = (selector, hp, max, label) => bar(root.querySelector(selector), hp, max, label + ' Stronghold HP');
  const initial = name => (name.trim()[0] || '?').toUpperCase();
  function clock(text) {
    const box = root.querySelector('.battle-time');
    if (text.startsWith('STARTING')) { box.dataset.phase = 'countdown'; return ['0:00', 'Get ready']; }
    box.dataset.phase = 'running';
    return [text, root.dataset.ranked === 'true' ? 'Ranked' : 'Casual'];
  }
  const panel = root.querySelector('[data-inspect-panel]');
  function inspect() {
    const f = (root.dataset.inspect || '').split('|');
    if (f.length < 18) { panel.hidden = true; delete panel.querySelector('.inspect-hp').dataset.hp; return; }
    const [kind,name,form,faction,role,own,hp,max,attackLabel,attack,every,dps,range,speed,splash,onhit,status,synergy] = f;
    panel.hidden = false;
    panel.dataset.team = own === 'true' ? 'ally' : 'enemy';
    const image = panel.querySelector('img');
    portrait(image, kind);
    const put = (key, value) => panel.querySelectorAll('[data-i="'+key+'"]').forEach(n => { if (n.textContent !== value) n.textContent = value; });
    put('name', name); put('form', form); put('owner', own === 'true' ? 'Yours' : 'Enemy'); put('faction', faction); put('role', role);
    put('attack-label', attackLabel); put('attack', attack); put('dps', dps); put('every', every); put('range', range + ' tiles');
    put('speed', speed + ' tiles/s'); put('splash', splash === '-' ? '-' : splash + ' tiles'); put('onhit', onhit); put('status', status); put('synergy', synergy);
    panel.querySelector('[data-i="status"]').dataset.status = status;
    bar(panel.querySelector('.inspect-hp'), Number(hp), Number(max), name + ' HP');
  }
  function render() {
    const s = root.dataset;
    set('[data-battle-you]', s.you || 'You'); set('[data-battle-opponent]', s.opponent || 'Opponent');
    root.querySelector('[data-battle-you]').title = s.you || 'You'; root.querySelector('[data-battle-opponent]').title = s.opponent || 'Opponent';
    set('[data-crest-you]', initial(s.you || 'You')); set('[data-crest-opponent]', initial(s.opponent || 'Opponent'));
    const [time, mode] = clock(s.clock || '3:00');
    set('[data-battle-clock]', time); set('[data-battle-mode]', mode);
    inspect();
    health('.vitality.ally', number('hpOwn'), number('hpMax'), s.you || 'Your');
    health('.vitality.enemy', number('hpEnemy'), number('hpMax'), s.opponent || 'Enemy');
    const scale = Math.max(1,number('energyScale'));
    const energy = number('energy') / scale;
    const readable = value => Number.isInteger(value) ? String(value) : value.toFixed(1);
    set('[data-battle-energy]', readable(energy));
    const visualMax = 25 * scale;
    root.querySelector('[data-battle-energy-fill]').style.width = percent(number('energy'),visualMax) + '%';
    const track = root.querySelector('.energy-track'); track.setAttribute('aria-valuenow', energy); track.setAttribute('aria-valuemax','25'); track.setAttribute('aria-valuemin','0');
    set('[data-battle-interest]', '+' + (number('interest') / scale).toFixed(2));
    const interestSeconds = Math.ceil(number('interestIn') / 20);
    set('[data-battle-interest-in]', Math.floor(interestSeconds / 60) + ':' + String(interestSeconds % 60).padStart(2,'0'));
    const ownAugments = root.dataset.augmentsYou || ''; const enemyAugments = root.dataset.augmentsEnemy || '';
    const youAug = root.querySelector('[data-augments-you]'); youAug.hidden = !ownAugments; set('[data-augments-you]', ownAugments);
    const enemyAug = root.querySelector('[data-augments-enemy]'); enemyAug.hidden = !enemyAugments; set('[data-augments-enemy]', enemyAugments);
    renderCouncil(s);
    set('[data-battle-army]', s.army || '0');
    const synergy = root.querySelector('[data-battle-synergy]'); synergy.hidden = !s.synergy; set('[data-battle-synergy]',s.synergy || '');
    cards.forEach((card,i) => {
      const [kind,name,role,cost,faction,available,pending] = (root.getAttribute('data-offer-'+i) || '').split('|');
      if (!kind) return;
      const image = card.querySelector('img');
      portrait(image, kind);
      card.querySelector('.champion-name').textContent = name;
      const roleLabel = role === 'Dps' ? 'DPS' : role;
      card.querySelector('.champion-role').textContent = roleLabel + ' · ' + faction;
      card.querySelector('.card-cost b').textContent = cost;
      card.querySelector('.card-slots').textContent = '';
      card.dataset.available = available; card.disabled = false; card.setAttribute('aria-disabled',available === 'true' ? 'false' : 'true'); card.classList.toggle('unaffordable',available !== 'true'); card.classList.toggle('pending',pending === 'true'); card.setAttribute('aria-busy',pending === 'true' ? 'true' : 'false');
      card.setAttribute('aria-label',name + ', '+roleLabel+', '+faction+', '+cost+' Energy'); card.title = name + ' · '+cost+' Energy. Hold or right-click for details.';
    });
    function renderCouncil(s) {
    const parts = (s.council || 'closed').split('|');
    if (parts.length < 4 || parts[0] === 'closed') { pause.hidden = true; council.round = ''; council.picked = null; council.sent = ''; return; }
    if (council.round !== parts[0]) { council.round = parts[0]; council.picked = null; council.sent = ''; }
    pause.hidden = false;
    const seconds = Math.ceil(Number(parts[1]) / 20);
    set('[data-council-timer]', Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2,'0'));
    root.querySelector('.council-clock').classList.toggle('urgent', seconds <= 10);
    const resolved = parts[2] === 'true';
    const defs = (s.councilOptions || '').split('|');
    const scale = Math.max(1, number('energyScale'));
    pause.querySelectorAll('[data-council-option]').forEach((card, i) => {
      // `id:name:text:cost`; a text may itself contain ':'.
      const f = (defs[i] || '').split(':'); const name = f[1], cost = f.length > 3 ? f[f.length - 1] : '', text = f.slice(2, -1).join(':');
      card.querySelector('.council-name').textContent = name || '—';
      card.querySelector('.council-text').textContent = text || '';
      const affordable = Number(cost) * scale <= Number(s.energy);
      card.querySelector('.council-cost').textContent = cost ? cost + ' Energy' + (affordable || resolved ? '' : ' · not enough') : '';
      const available = !resolved && affordable;
      if (!available && council.picked === i && !resolved) council.picked = null;
      card.dataset.available = available ? 'true' : 'false';
      card.disabled = !available && !(resolved && council.picked === i);
      card.classList.toggle('selected', council.picked === i);
      // Short of Energy: grey, like an unaffordable Shop card.
      card.classList.toggle('unaffordable', !resolved && !affordable);
      card.setAttribute('aria-pressed', council.picked === i ? 'true' : 'false');
      card.setAttribute('aria-label', (name || 'Augment') + ', ' + (text || '') + ', ' + (cost || '0') + ' Energy');
    });
    confirmButton.disabled = resolved || council.picked === null;
    pause.querySelector('.council-forgo').disabled = resolved;
    pause.querySelector('.council-actions').hidden = resolved;
    pause.querySelector('[data-council-wait]').hidden = !resolved;
    set('[data-council-wait-title]', council.sent === 'forgo' ? 'Round forgone.' : 'Augment selected!');
  }
  const surrender = root.querySelector('.battle-surrender'); surrender.dataset.state = s.surrender || 'ready'; surrender.disabled = s.surrender === 'sent';
    set('[data-battle-surrender]',s.surrender === 'sent' ? 'Leaving...' : s.surrender === 'armed' ? 'Confirm?' : 'Surrender');
    surrender.title = s.surrender === 'armed' ? (s.ranked === 'true' ? 'Confirm: this counts as a Ranked loss' : 'Click again to surrender') : 'Click twice to surrender';
  }
  new MutationObserver(render).observe(root,{attributes:true,attributeFilter:['data-hp-own','data-hp-enemy','data-hp-max','data-you','data-opponent','data-ranked','data-clock','data-energy','data-energy-scale','data-army','data-synergy','data-surrender','data-offer-0','data-offer-1','data-offer-2','data-offer-3','data-offer-4','data-inspect','data-interest','data-interest-in','data-augments-you','data-augments-enemy','data-council','data-council-options']});
  root.dataset.ready = 'true';
})();
