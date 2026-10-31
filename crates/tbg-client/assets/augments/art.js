//! Lazy Augment illustration/frame loading for the four authoritative round options.
(() => {
  'use strict';
  let data, loading, failed = '', active = '';
  const loaded = new Set();
  const source = filename => 'assets/augments/' + filename;
  function preload(ids) {
    const filenames = new Set();
    for (const id of ids) {
      const def = data.augments[id];
      if (!def || def.id !== id) continue;
      const rarity = data.rarities.find(r => r.name === def.rarity);
      filenames.add(def.asset.display);
      if (rarity) filenames.add(rarity.frame.display);
    }
    for (const filename of filenames) {
      if (loaded.has(filename)) continue;
      loaded.add(filename);
      const image = new Image(); image.decoding = 'async'; image.src = source(filename);
    }
  }
  function open(ids, update) {
    const key = ids.join(','); active = key;
    if (data) { preload(ids); return data; }
    if (loading || failed === key) return null;
    loading = fetch(source('augments.json?art=20261030'))
      .then(response => { if (!response.ok) throw new Error('Augment art unavailable'); return response.json(); })
      .then(value => { data = value; if (active === key) { preload(ids); update(); } })
      .catch(() => { failed = key; })
      .finally(() => { loading = null; });
    return null;
  }
  function picture(image, filename) {
    if (image.dataset.art === filename) return;
    image.dataset.art = filename; image.decoding = 'async'; image.src = source(filename);
  }
  function decorate(card, id) {
    const def = data && data.augments[id];
    if (!def || def.id !== id) return;
    const rarity = data.rarities.find(r => r.name === def.rarity);
    if (!rarity) return;
    card.dataset.rarity = def.rarity.toLowerCase();
    card.style.setProperty('--rarity', rarity.color);
    // `def.rarity` keys the frame art; only the shown label is translated.
    card.querySelector('.council-rarity').textContent = globalThis.TBG_I18N.t('rarity.' + def.rarity.toLowerCase());
    picture(card.querySelector('.council-illustration'), def.asset.display);
    picture(card.querySelector('.council-frame'), rarity.frame.display);
  }
  globalThis.TbgAugmentArt = Object.freeze({open, decorate, close:() => { active = ''; }});
})();
