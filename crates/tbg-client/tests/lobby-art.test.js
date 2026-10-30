//! Illustrated lobby assets are complete, local and inside their transport budget.
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const zlib=require('node:zlib');
const folder=path.resolve(__dirname,'../assets/lobby');

test('the four feature illustrations, logo, player crest and status background are integrated',()=>{
  const catalog=JSON.parse(fs.readFileSync(path.join(folder,'presentation.json'),'utf8'));
  const mapping=JSON.parse(fs.readFileSync(path.join(folder,'art.json'),'utf8'));
  assert.equal(catalog.schema,1);assert.equal(catalog.assets.length,7);
  let total=0;
  for(const a of catalog.assets){
    for(const f of new Set(Object.values(a.asset))){
      assert.match(f,/^[a-z0-9-]+\.(svg|webp)$/);
      const bytes=fs.readFileSync(path.join(folder,f));
      assert.ok(bytes.length>0);total+=f.endsWith('.svg')?zlib.gzipSync(bytes).length:bytes.length;
    }
    const key=a.id==='crest-v2'?'emblem':a.id.replace(/-v2$/,'');
    assert.equal(mapping[key],'assets/lobby/'+a.asset.display);
  }
  assert.ok(total<=catalog.artBudgetBytes);
  const html=fs.readFileSync(path.resolve(__dirname,'../index.html'),'utf8');
  assert.match(html,/class="profile-backdrop" data-art="profile"/);
  assert.match(html,/class="player-crest" data-art="emblem"/);
});
