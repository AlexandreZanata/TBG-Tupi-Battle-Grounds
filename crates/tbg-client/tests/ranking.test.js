//! Public ranked record contracts, paging boundaries and integrated artwork.
'use strict';
const assert=require('node:assert/strict');
const {test}=require('node:test');
const fs=require('node:fs');
const path=require('node:path');
const D=require('../assets/ranking/data.js');
const row=(rank,name='Player_'+rank)=>({rank,name,rating:1000,games:4,wins:2,losses:1,draws:1,win_rate:50});
test('BR-R07: server rank order survives presentation, including equal Rating',()=>{
 const rows=[row(1,'Zulu'),row(2,'Alpha')];
 assert.deepEqual(D.board({rows,total:2}).rows,rows);
 assert.throws(()=>D.board({rows:[row(0)],total:1}));
 assert.throws(()=>D.board({rows:[row(1,'<img onerror=x>')],total:1}));
 assert.throws(()=>D.board({rows:[{...row(1),rating:NaN}],total:1}));
});
test('BR-P05: public records keep W/L/D and signed deltas without private fields',()=>{
 const p=D.profile({...row(1),secret:'ignored',recent:[{opponent:'Other',result:'W',delta:15},{opponent:'Third',result:'L',delta:-12},{opponent:'Fourth',result:'D',delta:0}]});
 assert.equal(p.secret,undefined);assert.deepEqual(p.recent.map(m=>D.delta(m.delta)),['+15','−12','0']);
 assert.equal(D.profile({...row(1),rank:null,recent:[]}).rank,null);
 assert.throws(()=>D.profile({...row(1),recent:[{opponent:'Other',result:'X',delta:0}]}));
 assert.throws(()=>D.profile({...row(1),win_rate:101,recent:[]}));
});
test('ten-row paging clamps external offsets and exposes usable first/last controls',()=>{
 for(const [raw,want]of[[-1,0],['oops',0],[19,10],[Infinity,0],[999999,100000]])assert.equal(D.offset(raw),want);
 assert.equal(D.last(0),0);assert.equal(D.last(10),0);assert.equal(D.last(11),10);
 assert.deepEqual(D.pages(20,100).numbered,[0,10,20,30,40]);
 assert.deepEqual(D.pages(90,100).numbered,[50,60,70,80,90]);
 assert.equal(D.pages(90,100).next,90);
});
test('all six original ranking assets are wired into both HTML hosts and stay in budget',()=>{
 const base=path.join(__dirname,'../assets/ranking');const data=JSON.parse(fs.readFileSync(path.join(base,'ranking.json')));
 assert.equal(data.assets.length,6);let bytes=0;
 for(const a of data.assets){assert.match(a.asset.display,/^[a-z-]+\.webp$/);const b=fs.readFileSync(path.join(base,a.asset.display));assert.equal(b.toString('ascii',8,12),'WEBP');bytes+=b.length;}
 assert.ok(bytes<=data.artBudgetBytes);
 const panel=fs.readFileSync(path.join(base,'panel.html'),'utf8');
 for(const a of data.assets)assert.ok(panel.includes('data-ranking-art="'+a.id+'"'));
 const web=fs.readFileSync(path.join(__dirname,'../ranking/index.html'),'utf8');
 assert.ok(web.includes('ranking/controller.js'));assert.ok(!/wasm|data-trunk/.test(web));
 const game=fs.readFileSync(path.join(__dirname,'../index.html'),'utf8');assert.ok(game.includes('id="tbg-ranking"'));
});
