//! Shop Faction frame projection; the separate Synergy emblems stay unloaded until a future UI uses them.
(() => {
  'use strict';
  const names=new Set(['Humans','Undead','Machines','Plants']);
  const base='assets/factions/';
  let data,loading,failed=false;
  function validate(value){
    if(value?.schema!==1||!Array.isArray(value.factions)||value.factions.length!==names.size)throw new Error('Invalid Faction art');
    const seen=new Set();
    for(const f of value.factions){
      if(!names.has(f.name)||seen.has(f.name)||!/^#[0-9a-f]{6}$/i.test(f.color)||!/^[a-z-]+\.(svg|webp)$/.test(f.frame?.display))throw new Error('Invalid Faction frame');
      if(!['portraitTop','portraitHeight','nameY','roleY','costY'].every(k=>Number.isFinite(f.layout?.[k])&&f.layout[k]>=0&&f.layout[k]<=100))throw new Error('Invalid Faction layout');
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
        // Frames only: these cosmetic emblems have no Synergy gameplay or HUD yet.
        for(const f of data.factions){const image=new Image();image.decoding='async';image.src=base+f.frame.display;}
        update();
      })
      .catch(()=>{failed=true;})
      .finally(()=>{loading=null;});
  }
  function decorate(card,faction,update){
    const image=card.querySelector('.champion-frame');
    if(!names.has(faction)){image.hidden=true;delete card.dataset.faction;return;}
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
    if(image.dataset.art!==f.frame.display){image.dataset.art=f.frame.display;image.decoding='async';image.src=base+f.frame.display;}
    image.hidden=false;
  }
  globalThis.TbgFactionArt=Object.freeze({decorate});
})();
