// Battle presentation reads the client projection; purchases remain PlayerIntentions.
(() => {
  'use strict';
  const root = document.getElementById('tbg-battle');
  if (!root) return;
  const set = (selector, text) => root.querySelectorAll(selector).forEach(node => { if (node.textContent !== String(text)) node.textContent = String(text); });
  const number = (key) => Number(root.dataset[key]) || 0;
  // Chrome text from the shared catalog; Rust already localized the data fields.
  const I = globalThis.TBG_I18N;
  const t = (key, args) => I.t(key, args);
  const amount = value => Number.isInteger(value) ? I.number(value, 0) : I.number(value, 1);
  const send = action => { if (!root.hidden) root.dataset.action = action; };
  const cards = [...root.querySelectorAll('.battle-card')];
  const tr = key => globalThis.TBG_I18N?.t(key) || key;
  let shopFlight = false, shopObserved = false;
  const shopSend = action => {
    if (shopFlight || root.dataset.shopPending === 'true') return;
    shopFlight = true; shopObserved = false;
    cards.forEach(card => { card.dataset.available = 'false'; card.setAttribute('aria-busy','true'); });
    send(action);
  };
  const locks = cards.map((card, i) => {
    const wrapper = document.createElement('div'); wrapper.className = 'shop-slot';
    card.replaceWith(wrapper); wrapper.append(card);
    const lock = document.createElement('button'); lock.type = 'button'; lock.className = 'shop-lock'; lock.hidden = true;
    const badge = document.createElement('span'); badge.className = 'reserve-charges'; badge.hidden = true;
    wrapper.append(lock, badge);
    lock.addEventListener('click', () => {
      if (!lock.disabled) shopSend(lock.dataset.charges > 0 ? 'release:'+i : 'reserve:'+i+':'+lock.dataset.kind);
    });
    return {lock,badge};
  });
  const reroll = document.createElement('button'); reroll.type = 'button'; reroll.className = 'shop-reroll iron-frame'; reroll.hidden = true;
  reroll.addEventListener('click', () => { if (!reroll.disabled) shopSend('reroll'); });
  root.querySelector('.battle-resources').append(reroll);
  document.addEventListener('keydown', event => {
    if (!root.hidden && !reroll.hidden && !event.repeat && event.code === 'KeyR' && !['INPUT','TEXTAREA'].includes(event.target.tagName) && !event.target.isContentEditable) {
      event.preventDefault(); if (!reroll.disabled) shopSend('reroll');
    }
  });
  const percent = (value, max) => Math.max(0, Math.min(100, value / Math.max(1, max) * 100));
  root.querySelector('[data-battle-action="surrender"]').addEventListener('click', () => send('surrender'));
  root.querySelector('[data-battle-action="close-inspect"]').addEventListener('click', () => send('close-inspect'));
  // A spectator's Leave (P47): Rust stops the watch and returns to the lobby.
  root.querySelector('[data-battle-action="leave-watch"]').addEventListener('click', () => send('leave-watch'));
  // War Council (BR-AG12–14): pick a card, then confirm; the choice is final.
  const council = { round: '', picked: null, sent: '', initialTicks: 0 };
  const augmentArt = globalThis.TbgAugmentArt;
  const factionArt = globalThis.TbgFactionArt;
  const pause = root.querySelector('[data-pause]');
  const confirmButton = root.querySelector('[data-battle-action="confirm"]');
  root.querySelector('[data-battle-action="forgo"]').addEventListener('click', () => { council.sent = 'forgo'; send('forgo'); });
  confirmButton.addEventListener('click', () => { if (council.picked === null) return; council.sent = 'aug'; send('aug:' + council.picked); });
  root.querySelectorAll('[data-council-option]').forEach(card => card.addEventListener('click', () => { if (card.dataset.available === 'true') { council.picked = Number(card.dataset.councilOption); render(); } }));
  cards.forEach((card, i) => {
    let held = false, timer;
    card.addEventListener('pointerdown', event => { if (event.button !== 0 || card.disabled) return; held = false; timer = setTimeout(() => { held = true; send('inspect:' + i); }, 500); });
    for (const event of ['pointerup','pointercancel','pointerleave']) card.addEventListener(event, () => clearTimeout(timer));
    card.addEventListener('click', () => { if (!shopFlight && root.dataset.shopPending !== 'true' && !held && card.dataset.available === 'true') send('buy:' + i); held = false; });
    card.addEventListener('contextmenu', event => { event.preventDefault(); clearTimeout(timer); held = true; send('inspect:' + i); });
  });
  document.addEventListener('keydown', event => { if (!root.hidden && event.key === 'Escape') send('deselect'); });
  const slug = name => name.toLowerCase().replace(/[^a-z0-9]/g,'');
  // Every buyable unit ships a portrait (tests/portraits.test.js). The proxy keeps
  // images for four hours, 404s included, so the URL carries a revision: bump
  // PORTRAIT_REV whenever a portrait file changes (scripts/unit-portraits.cjs).
  const PORTRAIT_REV = 'p25-1';
  const portrait = (image, kind) => {
    if (image.dataset.kind === kind) return;
    image.dataset.kind = kind; image.style.visibility = 'hidden';
    image.onload = () => { image.style.visibility = ''; };
    image.src = 'assets/battle/portraits/' + slug(kind) + '.webp?v=' + PORTRAIT_REV;
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
  const health = (selector, hp, max, name) => bar(root.querySelector(selector), hp, max, t('battle.stronghold_hp', {name}));
  const initial = name => (name.trim()[0] || '?').toUpperCase();
  function clock(text) {
    const box = root.querySelector('.battle-time');
    if (text.startsWith('STARTING')) { box.dataset.phase = 'countdown'; return ['0:00', t('battle.get_ready')]; }
    box.dataset.phase = 'running';
    return [text, t(root.dataset.ranked === 'true' ? 'live.mode.ranked' : 'live.mode.casual')];
  }
  const panel = root.querySelector('[data-inspect-panel]');
  function inspect() {
    const f = (root.dataset.inspect || '').split('|');
    if (f.length < 18) { panel.hidden = true; delete panel.querySelector('.inspect-hp').dataset.hp; return; }
    const [kind,name,form,faction,role,own,hp,max,attackLabel,attack,every,dps,range,speed,splash,onhit,status,synergy,effects = '',ability = '',statusLabel = status] = f;
    panel.hidden = false;
    panel.dataset.team = own === 'true' ? 'ally' : 'enemy';
    const image = panel.querySelector('img');
    portrait(image, kind);
    const put = (key, value) => panel.querySelectorAll('[data-i="'+key+'"]').forEach(n => { if (n.textContent !== value) n.textContent = value; });
    put('name', name); put('form', form); put('owner', t(own === 'true' ? 'inspect.owner.yours' : 'inspect.owner.enemy')); put('faction', faction); put('role', role);
    put('attack-label', attackLabel); put('attack', attack); put('dps', dps); put('every', every); put('range', t('inspect.tiles', {n: range}));
    put('speed', t('inspect.tiles_per_s', {n: speed})); put('splash', splash === '-' ? '-' : t('inspect.tiles', {n: splash})); put('onhit', onhit); put('status', statusLabel); put('synergy', synergy);
    panel.querySelector('[data-i="status"]').dataset.status = status;
    put('ability', ability);
    panel.querySelector('[data-inspect-ability]').hidden = !ability;
    put('effects', effects);
    panel.querySelector('[data-inspect-effects]').hidden = !effects;
    bar(panel.querySelector('.inspect-hp'), Number(hp), Number(max), t('inspect.unit_hp_named', {name}));
  }
  // Fortification (D-46): the tower data file supplies names and effects;
  // `data-fortify` (from Rust) says what each branch offers right now.
  const TS = globalThis.TbgStronghold;
  const fortPanel = root.querySelector('[data-fort-panel]');
  const fortOptions = [...fortPanel.querySelectorAll('[data-fort-branch]')];
  const fort = {data: null, loading: null};
  function fortData() {
    fort.loading ||= fetch('assets/stronghold/strongholds.json')
      .then(response => { if (!response.ok) throw new Error('tower data unavailable'); return response.json(); })
      .then(data => { fort.data = data; TS.sprite(); render(); })
      .catch(() => { fort.loading = null; });
  }
  const towerOf = record => { const [kind = '', tier, branch] = (record || '').split('|'); return {kind, tier: Number(tier) || 0, branch: Number(branch) || 0}; };
  const towerLabel = (tower, def) => (def ? TS.name(def) : tower.kind) + ' T' + tower.tier;
  fortOptions.forEach(button => button.addEventListener('click', () => { if (button.dataset.state === 'open') send('fortify:' + button.dataset.fortBranch); }));
  // The panel opens on a click on your own tower and closes like a unit's panel.
  root.querySelector('[data-fort-toggle]').addEventListener('click', () => send('deselect'));
  function plates(you, enemy) {
    const data = fort.data;
    const mine = data && TS.kind(data, you.kind), theirs = data && TS.kind(data, enemy.kind);
    const label = (side, tower, def) => {
      const node = root.querySelector('[data-plate-tower="' + side + '"]');
      const value = tower.kind ? towerLabel(tower, def) : '';
      if (node.textContent !== value) node.textContent = value;
      node.title = def && tower.tier ? value + ' · ' + TS.branchName(def, tower.branch) : value;
      node.style.setProperty('--tower', def ? def.color : '#c9a86a');
    };
    label('you', you, mine); label('enemy', enemy, theirs);
  }
  function fortOption(button, def, you, state) {
    const b = Number(button.dataset.fortBranch), tier = Math.min(you.tier + 1, 2);
    const step = def.branches[b].steps[tier - 1], cost = TS.cost(fort.data, tier);
    const closed = state === 'closed';
    button.dataset.state = state; button.hidden = state === 'max';
    button.disabled = state !== 'open';
    const branch = TS.branchName(def, b), stepName = TS.stepName(def, b, tier - 1);
    button.querySelector('.fort-branch').textContent = branch;
    button.querySelector('.fort-step').textContent = closed ? t('fortify.closed', {branch: TS.branchName(def, you.branch)}) : t('fortify.step', {tier, step: stepName});
    button.querySelector('.fort-effect').textContent = closed ? '' : TS.effect(fort.data, def, b, tier);
    button.querySelector('.fort-cost b').textContent = closed ? '-' : String(cost);
    button.setAttribute('aria-label', closed ? t('fortify.aria_closed', {branch}) : t('fortify.aria_open', {branch, tier, step: stepName, cost}) + (state === 'short' ? t('battle.not_enough_energy') : ''));
  }
  function fortNote(states, you, def, s) {
    const tier = Math.min(you.tier + 1, 2);
    const branch = TS.branchName(def, you.branch);
    if (you.tier >= 2) return t('fortify.note.full', {branch});
    if (states.includes('pending')) return t('fortify.note.pending');
    if (states.includes('paused')) return t('fortify.note.paused');
    if (states.includes('idle')) return t('fortify.note.idle');
    if (states.includes('short')) {
      const have = (Number(s.energy) || 0) / Math.max(1, number('energyScale'));
      return t('fortify.note.short', {cost: TS.cost(fort.data, tier), have: amount(have)});
    }
    return t(you.tier ? 'fortify.note.next' : 'fortify.note.first', {branch});
  }
  function renderFortify(s) {
    const you = towerOf(s.towerYou), enemy = towerOf(s.towerEnemy);
    if (!fort.data) fortData();
    plates(you, enemy);
    const def = fort.data && TS.kind(fort.data, you.kind);
    fortPanel.hidden = !def || s.towerOpen !== 'true';
    if (!def || fortPanel.hidden) return;
    fortPanel.style.setProperty('--tower', def.color);
    const toggleLabel = t(fortToggle.getAttribute('aria-expanded') === 'true' ? 'fortify.hide' : 'fortify.show');
    if (fortToggle.getAttribute('aria-label') !== toggleLabel) fortToggle.setAttribute('aria-label', toggleLabel);
    fortPanel.querySelector('[data-fort-icon]').setAttribute('href', TS.icon(def.kind));
    set('[data-fort-name]', TS.name(def));
    set('[data-fort-tier]', t('fortify.tier', {tier: you.tier}) + (you.tier ? ' · ' + TS.branchName(def, you.branch) : ''));
    set('[data-fort-hp]', number('hpOwn') + ' / ' + number('hpMaxOwn'));
    set('[data-fort-passive-name]', TS.text(def, 'passive.name'));
    const passive = TS.fill(TS.text(def, 'passive.text'), TS.stats(def, you.tier, you.branch), fort.data);
    set('[data-fort-passive]', passive);
    fortPanel.querySelector('.fort-passive').title = passive; // the panel clamps it to two lines
    const states = (s.fortify || 'idle|idle').split('|');
    fortOptions.forEach((button, b) => fortOption(button, def, you, states[b] || 'idle'));
    set('[data-fort-note]', fortNote(states, you, def, s));
    fortPanel.dataset.tier = String(you.tier);
  }
  function render() {
    const s = root.dataset;
    // Player names are user data; only the missing-name fallback is translated.
    const you = s.you || t('battle.you'), opponent = s.opponent || t('battle.opponent');
    set('[data-battle-you]', you); set('[data-battle-opponent]', opponent);
    root.querySelector('[data-battle-you]').title = you; root.querySelector('[data-battle-opponent]').title = opponent;
    set('[data-crest-you]', initial(you)); set('[data-crest-opponent]', initial(opponent));
    const [time, mode] = clock(s.clock || '3:00');
    set('[data-battle-clock]', time); set('[data-battle-mode]', mode);
    inspect();
    // Each seat has its own max HP: kinds and Fortification differ (BR-SK07).
    health('.vitality.ally', number('hpOwn'), number('hpMaxOwn'), you);
    health('.vitality.enemy', number('hpEnemy'), number('hpMaxEnemy'), opponent);
    renderFortify(s);
    const scale = Math.max(1,number('energyScale'));
    const energy = number('energy') / scale;
    set('[data-battle-energy]', amount(energy));
    const visualMax = 25 * scale;
    root.querySelector('[data-battle-energy-fill]').style.width = percent(number('energy'),visualMax) + '%';
    const track = root.querySelector('.energy-track'); track.setAttribute('aria-valuenow', energy); track.setAttribute('aria-valuemax','25'); track.setAttribute('aria-valuemin','0');
    // Interest (BR-E05): each full 5 Energy held adds 25 % to the regeneration.
    const boost = number('regen') / 10000;
    set('[data-battle-interest]', '×' + I.number(boost, 2));
    set('[data-battle-interest-in]', Math.min(5, Math.floor(energy / 5)) + '/5');
    const ownAugments = root.dataset.augmentsYou || ''; const enemyAugments = root.dataset.augmentsEnemy || '';
    const youAug = root.querySelector('[data-augments-you]'); youAug.hidden = !ownAugments; set('[data-augments-you]', ownAugments);
    const enemyAug = root.querySelector('[data-augments-enemy]'); enemyAug.hidden = !enemyAugments; set('[data-augments-enemy]', enemyAugments);
    renderCouncil(s);
    set('[data-battle-army]', s.army || '0');
    const synergy = root.querySelector('[data-battle-synergy]'); synergy.hidden = !s.synergy; set('[data-battle-synergy]',s.synergy || '');
    if (root.dataset.shopPending === 'true') { shopFlight = true; shopObserved = true; }
    else if (shopObserved || root.dataset.council !== 'closed') { shopFlight = false; shopObserved = false; }
    const active = cards.reduce((n,_,i) => n + (Number((root.getAttribute('data-offer-'+i)||'').split('|')[7]) > 0 ? 1 : 0), 0);
    const enabled = root.dataset.rerollCost !== undefined && root.dataset.rerollCost !== '';
    const pausedShop = root.dataset.council !== 'closed';
    reroll.hidden = !enabled;
    reroll.textContent = tr('shop.reroll') + ' · ' + Number(root.dataset.rerollCost || 0) / (number('energyScale') || 600) + ' Energy [R]';
    reroll.disabled = shopFlight || pausedShop || number('energy') < number('rerollCost');
    cards.forEach((card,i) => {
      const [kind,name,role,cost,faction,available,pending,charges,held,roleLabel = role,factionLabel = faction] = (root.getAttribute('data-offer-'+i) || '').split('|');
      if (!kind) return;
      const {lock,badge} = locks[i];
      lock.hidden = !enabled; lock.dataset.kind = kind; lock.dataset.charges = charges || '0';
      lock.textContent = Number(charges) > 0 ? '◆' : '◇';
      lock.disabled = shopFlight || pausedShop || (Number(charges) === 0 && (held === 'true' || active >= number('reserveLimit')));
      lock.title = held === 'true' && Number(charges) === 0 ? tr('shop.held') : active >= number('reserveLimit') && Number(charges) === 0 ? tr('shop.limit') : tr(Number(charges) > 0 ? 'shop.release' : 'shop.reserve');
      lock.setAttribute('aria-label',lock.title+' '+name);
      badge.hidden = Number(charges) === 0;
      badge.textContent = tr('shop.reserved') + ' ' + '●'.repeat(Number(charges));
      card.classList.toggle('reserved', Number(charges) > 0);
      factionArt?.decorate(card, faction, render);
      const image = card.querySelector('img');
      portrait(image, kind);
      card.querySelector('.champion-name').textContent = name;
      card.querySelector('.champion-role').textContent = roleLabel + ' · ' + factionLabel;
      card.querySelector('.card-cost b').textContent = cost;
      card.querySelector('.card-slots').textContent = '';
      card.dataset.available = shopFlight ? 'false' : available; card.disabled = false; card.setAttribute('aria-disabled',available === 'true' ? 'false' : 'true'); card.classList.toggle('unaffordable',available !== 'true'); card.classList.toggle('pending',pending === 'true'); card.setAttribute('aria-busy',pending === 'true' ? 'true' : 'false');
      card.setAttribute('aria-label', t('battle.card_label', {name, role: roleLabel, faction: factionLabel, cost})); card.title = t('battle.card_title', {name, cost});
    });
    function renderCouncil(s) {
    const parts = (s.council || 'closed').split('|');
    if (parts.length < 4 || parts[0] === 'closed') { pause.hidden = true; council.round = ''; council.picked = null; council.sent = ''; augmentArt.close(); return; }
    if (council.round !== parts[0]) { council.round = parts[0]; council.picked = null; council.sent = ''; council.initialTicks = Number(parts[1]); }
    pause.hidden = false;
    const ids = parts[3].split(',').map(Number);
    const artData = augmentArt.open(ids, render);
    const ticks = Number(parts[1]);
    const seconds = Math.ceil(ticks / (artData ? artData.tickHz : 20));
    const totalTicks = artData ? artData.roundTicks : council.initialTicks;
    root.querySelector('[data-council-progress]').style.width = percent(ticks, totalTicks) + '%';
    const timeTrack = pause.querySelector('.council-timer-track');
    timeTrack.setAttribute('aria-valuenow', String(ticks));
    timeTrack.setAttribute('aria-valuemax', String(totalTicks));
    timeTrack.setAttribute('aria-valuemin', '0');
    set('[data-council-timer]', Math.floor(seconds / 60) + ':' + String(seconds % 60).padStart(2,'0'));
    root.querySelector('.council-clock').classList.toggle('urgent', seconds <= 5);
    const resolved = parts[2] === 'true';
    const defs = (s.councilOptions || '').split('|');
    const scale = Math.max(1, number('energyScale'));
    pause.querySelectorAll('[data-council-option]').forEach((card, i) => {
      // `id:name:text:cost`; a text may itself contain ':'.
      augmentArt.decorate(card, ids[i]);
      const f = (defs[i] || '').split(':'); const name = f[1], cost = f.length > 3 ? f[f.length - 1] : '', text = f.slice(2, -1).join(':');
      card.querySelector('.council-name').textContent = name || '—';
      card.querySelector('.council-text').textContent = text || '';
      const affordable = Number(cost) * scale <= Number(s.energy);
      card.querySelector('.council-cost').textContent = cost;
      const available = !resolved && affordable;
      if (!available && council.picked === i && !resolved) council.picked = null;
      card.dataset.available = available ? 'true' : 'false';
      card.disabled = !available && !(resolved && council.picked === i);
      card.classList.toggle('selected', council.picked === i);
      // Short of Energy: grey, like an unaffordable Shop card.
      card.classList.toggle('unaffordable', !resolved && !affordable);
      card.setAttribute('aria-pressed', council.picked === i ? 'true' : 'false');
      card.setAttribute('aria-label', t('council.option_label', {name: name || t('council.augment'), text: text || '', cost: cost || '0'}) + (!resolved && !affordable ? t('battle.not_enough_energy') : ''));
    });
    confirmButton.disabled = resolved || council.picked === null;
    pause.querySelector('.council-forgo').disabled = resolved;
    pause.querySelector('.council-actions').hidden = resolved;
    pause.querySelector('[data-council-wait]').hidden = !resolved;
    set('[data-council-wait-title]', t(council.sent === 'forgo' ? 'council.forgone' : 'council.selected'));
  }
  const surrender = root.querySelector('.battle-surrender'); surrender.dataset.state = s.surrender || 'ready'; surrender.disabled = s.surrender === 'sent';
    set('[data-battle-surrender]', t(s.surrender === 'sent' ? 'battle.surrender_leaving' : s.surrender === 'armed' ? 'battle.surrender_confirm' : 'battle.surrender'));
    surrender.title = t(s.surrender === 'armed' ? (s.ranked === 'true' ? 'battle.surrender_ranked' : 'battle.surrender_again') : 'battle.surrender_twice');
  }
  new MutationObserver(render).observe(root,{attributes:true,attributeFilter:['data-reroll-cost','data-shop-pending','data-reserve-limit','data-hp-own','data-hp-enemy','data-hp-max-own','data-hp-max-enemy','data-tower-you','data-tower-enemy','data-tower-open','data-fortify','data-you','data-opponent','data-ranked','data-clock','data-energy','data-energy-scale','data-army','data-synergy','data-surrender','data-offer-0','data-offer-1','data-offer-2','data-offer-3','data-offer-4','data-inspect','data-regen','data-augments-you','data-augments-enemy','data-council','data-council-options']});
  document.addEventListener('tbg:lang', render);
  root.dataset.ready = 'true';
})();
