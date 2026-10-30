//! Art manifests ship complete, budgeted sets; Augment pictures load only for an open round.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const zlib = require('node:zlib');
const assets = path.resolve(__dirname, '../assets');
const read = name => JSON.parse(fs.readFileSync(path.join(assets, name), 'utf8'));
const augments = read('augments/augments.json');

test('all 81 art entries have unique, safe local files within each complete set budget', () => {
  const arcana = read('arcana/arcana.json'), towers = read('stronghold/strongholds.json');
  const sets = [
    ['arcana', arcana, [...arcana.families.flatMap(f => [f.emblem, f.banner]), ...arcana.nodes.map(n => n.asset)], 21],
    ['augments', augments, [...augments.rarities.map(r => r.frame), ...augments.augments.map(a => a.asset)], 44],
    ['stronghold', towers, towers.kinds.flatMap(k => [k.portrait, k.banner, ...k.branches.map(b => b.asset)]), 16]
  ];
  for (const [folder, data, entries, count] of sets) {
    assert.equal(entries.length, count);
    assert.equal(new Set(entries.map(a => a.svg)).size, count);
    for (const entry of entries) for (const filename of new Set(Object.values(entry))) {
      assert.match(filename, /^[a-z0-9-]+\.(svg|webp)$/);
      assert.ok(fs.statSync(path.join(assets, folder, filename)).size > 0, filename);
    }
    let bytes = 0;
    for (const file of fs.readdirSync(path.join(assets, folder))) {
      if (!/\.(svg|webp)$/.test(file)) continue;
      const buffer = fs.readFileSync(path.join(assets, folder, file));
      bytes += file.endsWith('.svg') ? zlib.gzipSync(buffer, {level:9}).length : buffer.length;
    }
    assert.ok(bytes <= data.artBudgetBytes, folder + ': ' + bytes + ' > ' + data.artBudgetBytes);
  }
});

test('BR-AG12: opening a round preloads exactly its four illustrations and required frames', async () => {
  const sources = [], state = {fetches:0, updates:0};
  const context = {
    Image:class {set src(value){ sources.push(value); }},
    fetch:async () => {state.fetches++; return {ok:true,json:async () => augments};}
  };
  vm.runInNewContext(fs.readFileSync(path.join(assets,'augments/art.js'),'utf8'),context);
  assert.equal(state.fetches,0); assert.deepEqual(sources,[]);
  const loader=context.TbgAugmentArt;
  assert.equal(loader.open([0,5,20,30],()=>state.updates++),null);
  await new Promise(setImmediate);
  assert.equal(state.fetches,1); assert.equal(state.updates,1);
  const expected=[0,5,20,30].map(id=>'assets/augments/'+augments.augments[id].asset.display);
  expected.push(...augments.rarities.map(r=>'assets/augments/'+r.frame.display));
  assert.deepEqual(new Set(sources),new Set(expected));
  loader.open([0,5,20,30],()=>{}); assert.equal(sources.length,8);
  loader.close(); assert.equal(state.fetches,1);
});

test('canonical rarity names and their visual palettes are preserved', () => {
  assert.deepEqual(augments.rarities.map(r=>r.name),['Bronze','Silver','Gold','Diamond']);
  assert.equal(augments.augments.length,40);
  assert.ok(augments.augments.every((a,id)=>a.id===id));
  const html=fs.readFileSync(path.resolve(__dirname,'../index.html'),'utf8');
  assert.match(html,/Choose an Augment/);
  assert.doesNotMatch(html,/Reroll/);
  assert.match(html,/data-battle-action="forgo"/);
});
