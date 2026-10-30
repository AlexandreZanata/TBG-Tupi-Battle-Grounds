//! Faction frames are cosmetic and do not load the future Synergy emblems in a match.
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const zlib=require('node:zlib');
const folder=path.resolve(__dirname,'../assets/factions');
const manifest=()=>JSON.parse(fs.readFileSync(path.join(folder,'factions.json'),'utf8'));
const source=fs.readFileSync(path.join(folder,'art.js'),'utf8');
const card=()=>{
  const image={dataset:{},hidden:true};
  return {dataset:{},style:{setProperty(){}},querySelector:()=>image};
};

test('all canonical Factions have a separate mold and future emblem within the set budget',()=>{
  const data=manifest();
  assert.deepEqual(data.factions.map(f=>f.name),['Humans','Undead','Machines','Plants']);
  const files=data.factions.flatMap(f=>[...Object.values(f.frame),...Object.values(f.emblem)]);
  assert.equal(new Set(files).size,16);
  for(const file of files){assert.match(file,/^[a-z-]+\.(svg|webp)$/);assert.ok(fs.statSync(path.join(folder,file)).size>0);}
  const total=files.reduce((n,file)=>{const b=fs.readFileSync(path.join(folder,file));return n+(file.endsWith('.svg')?zlib.gzipSync(b).length:b.length);},0);
  assert.ok(total<=data.artBudgetBytes);
});

test('Shop preloads only four frames, retains canonical identity and clears a stale frame',async()=>{
  const sources=[];let requests=0,updates=0;
  const ctx={Image:class{set src(s){sources.push(s);}},fetch:async()=>{requests++;return {ok:true,json:async()=>manifest()};}};
  vm.runInNewContext(source,ctx);
  const node=card(),art=ctx.TbgFactionArt;
  art.decorate(node,'Humans',()=>updates++);
  await new Promise(setImmediate);
  assert.equal(requests,1);assert.equal(updates,1);assert.equal(sources.length,4);
  assert.ok(sources.every(s=>s.includes('/frame-')));
  for(const faction of manifest().factions){art.decorate(node,faction.name,()=>{});assert.equal(node.dataset.faction,faction.name);assert.equal(node.querySelector().src,'assets/factions/'+faction.frame.display);}
  art.decorate(node,'Unknown',()=>{});
  assert.equal(node.querySelector().hidden,true);assert.equal(node.dataset.faction,undefined);
  assert.equal(requests,1);
});

test('invalid remote art paths fail closed without preloading arbitrary content',async()=>{
  const sources=[];let requests=0;
  const data=manifest();data.factions[0].frame.display='https://other.invalid/frame.svg';
  const ctx={Image:class{set src(s){sources.push(s);}},fetch:async()=>{requests++;return {ok:true,json:async()=>data};}};
  vm.runInNewContext(source,ctx);const node=card();
  ctx.TbgFactionArt.decorate(node,'Humans',()=>{});
  await new Promise(setImmediate);
  ctx.TbgFactionArt.decorate(node,'Humans',()=>{});
  assert.equal(requests,1);assert.deepEqual(sources,[]);assert.equal(node.querySelector().hidden,true);
});
