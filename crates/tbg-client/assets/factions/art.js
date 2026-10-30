//! Shop Faction frame projection: each card's mold, its measured text rows and the Faction emblem in the top orb.
(() => {
  'use strict';
  const names=new Set(['Humans','Undead','Machines','Plants']);
  const base='assets/factions/';
  let data,loading,failed=false;
  function validate(value){
    if(value?.schema!==1||!Array.isArray(value.factions)||value.factions.length!==names.size)throw new Error('Invalid Faction art');
    const seen=new Set();
    for(const f of value.factions){
      if(!names.has(f.name)||seen.has(f.name)||!/^#[0-9a-f]{6}$/i.test(f.color)||!/^[a-z-]+\.(svg|webp)$/.test(f.frame?.display)||!/^[a-z-]+\.(svg|webp)$/.test(f.emblem?.display))throw new Error('Invalid Faction frame');
      if(!['portraitTop','portraitHeight','nameY','roleY','costY','orbY','orbSize'].every(k=>Number.isFinite(f.layout?.[k])&&f.layout[k]>=0&&f.layout[k]<=100))throw new Error('Invalid Faction layout');
      seen.add(f.name);
    }
    return value;
  }
  function load(update){
    if(data||loading||failed)return;
    loading=fetch(base+'factions.json?v=factions-1')
      .then(r=>{if(!r.ok)throw new Error('Faction art unavailable');return r.json();})
      .then(value=>{
        data=validate(value);
        // Frames and emblems are cosmetic: the emblem names the Faction, it adds no Synergy UI.
        for(const f of data.factions)for(const file of [f.frame.display,f.emblem.display]){const image=new Image();image.decoding='async';image.src=base+file;}
        update();
      })
      .catch(()=>{failed=true;})
      .finally(()=>{loading=null;});
  }
  function decorate(card,faction,update){
    const image=card.querySelector('.champion-frame');
    const emblem=card.querySelector('.champion-emblem');
    if(!names.has(faction)){image.hidden=true;if(emblem)emblem.hidden=true;delete card.dataset.faction;return;}
    if(!data){load(update);return;}
    const f=data.factions.find(row=>row.name===faction);
    card.dataset.faction=faction;
    card.style.setProperty('--faction-accent',f.color);
    // The frame coordinates remain visual metadata, separate from simulation values.
    card.style.setProperty('--frame-portrait-top',f.layout.portraitTop+'%');
    card.style.setProperty('--frame-portrait-height',f.layout.portraitHeight+'%');
    card.style.setProperty('--frame-name-y',f.layout.nameY+'%');
    card.style.setProperty('--frame-role-y',f.layout.roleY+'%');
    card.style.setProperty('--frame-cost-y',f.layout.costY+'%');
    card.style.setProperty('--frame-orb-y',f.layout.orbY+'%');
    card.style.setProperty('--frame-orb-size',f.layout.orbSize+'%');
    if(image.dataset.art!==f.frame.display){image.dataset.art=f.frame.display;image.decoding='async';image.src=base+f.frame.display;}
    image.hidden=false;
    if(emblem){
      if(emblem.dataset.art!==f.emblem.display){emblem.dataset.art=f.emblem.display;emblem.decoding='async';emblem.src=base+f.emblem.display;}
      emblem.hidden=false;
    }
  }
  globalThis.TbgFactionArt=Object.freeze({decorate});
})();
