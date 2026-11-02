// D-97 browser coordinator regressions; synthetic fetch/cookies are not real-browser acceptance.
const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),vm=require('node:vm'),crypto=require('node:crypto');
const source=fs.readFileSync(require('node:path').join(__dirname,'../assets/auth/session.js'),'utf8').replaceAll('export function ','function ');
function setup({locks=true,fetch}) {
  const values=new Map();let tail=Promise.resolve();
  const context=vm.createContext({crypto:crypto.webcrypto,TextDecoder,Uint8Array,Response,AbortController,setTimeout,clearTimeout,isSecureContext:true,location:{origin:'https://synthetic.example'},navigator:locks?{locks:{request:(_name,_options,run)=>{const next=tail.then(run);tail=next.catch(()=>{});return next;}}}:{},localStorage:{getItem:key=>values.get(key)??null,setItem:(key,value)=>values.set(key,String(value)),removeItem:key=>values.delete(key)},fetch});
  vm.runInContext(source,context);
  return {context,values,run:request=>new Promise(resolve=>context.startAuth(JSON.stringify({origin:'https://synthetic.example',epoch:1,remember:true,...request}),raw=>resolve(JSON.parse(raw))))};
}
const grant=(n=1)=>({access_token:String(n).padStart(64,'0'),session_id:'11'.repeat(16),server_time:1000,access_expires_at:1900,session_expires_at:2593000,persistent:true,profile:{name:'Synthetic',rating:1000,ranked_games:0,wins:0,losses:0,draws:0}});
const response=(value,status=200)=>new Response(JSON.stringify(value),{status,headers:{'Content-Type':'application/json'}});

test('without Web Locks, login is Access-only and cannot read or overwrite cookies',async()=>{
  const calls=[];const app=setup({locks:false,fetch:async(path,options)=>{calls.push([path,options]);return response(grant());}});
  const result=await app.run({kind:'login',input:'Synthetic-0000-0000-0000-0000'});
  assert.equal(result.kind,'signed_in');assert.equal(result.secure,false);
  assert.equal(calls.length,1);assert.equal(calls[0][1].credentials,'omit');assert.equal(calls[0][1].redirect,'error');assert.equal(calls[0][1].cache,'no-store');
  assert.equal(app.values.size,0,'no automatic credential or fallback storage');
});

test('a late login after Guest cancellation revokes its grant and cannot restore identity',async()=>{
  let release;const wait=new Promise(resolve=>release=resolve);let revoked=0;
  const app=setup({fetch:async(path)=>{
    if(path.endsWith('/login')){await wait;return response(grant());}
    if(path.endsWith('/logout')){revoked++;return response({error:'auth_required'},401);}
    return response({session_id:'11'.repeat(16)});
  }});
  const pending=app.run({kind:'login',input:'Synthetic-0000-0000-0000-0000'});
  await new Promise(resolve=>setTimeout(resolve,0));app.context.invalidateEpoch(2);release();
  const result=await pending;assert.equal(result.error,'cancelled');assert.ok(revoked>=1);
  assert.ok(![...app.values.values()].includes(grant().access_token));
});

test('a lost refresh reply retries the same operation before another tab rotates',async()=>{
  let attempts=0,active=0,maxActive=0;const operations=[];
  const app=setup({fetch:async(path,options)=>{
    if(path.endsWith('/refresh')) {
      active++;maxActive=Math.max(maxActive,active);operations.push(JSON.parse(options.body).operation_id);attempts++;
      await new Promise(resolve=>setTimeout(resolve,2));active--;
      if(attempts===1)throw new Error('network lost');return response(grant(attempts));
    }
    return response({session_id:'11'.repeat(16)});
  }});
  const [a,b]=await Promise.all([app.run({kind:'restore'}),app.run({kind:'refresh'})]);
  assert.equal(a.kind,'signed_in');assert.equal(b.kind,'signed_in');assert.equal(maxActive,1);
  assert.equal(operations[0],operations[1]);assert.notEqual(operations[1],operations[2]);
  for(const value of app.values.values())assert.notEqual(value,a.grant.access_token,'Access cannot enter localStorage');
});

test('BR-AS04/11: a confirmation outage preserves login and retries without blaming cookie storage',async()=>{
  let confirmations=0,logouts=0;
  const app=setup({fetch:async(path)=>{
    if(path.endsWith('/logout')){logouts++;return response({});}
    if(path.endsWith('/confirm')){confirmations++;return confirmations<=2?response({error:'unavailable'},503):response({session_id:grant().session_id});}
    return response(grant());
  }});
  const first=await app.run({kind:'login',input:'Synthetic-0000-0000-0000-0000'});
  assert.equal(first.kind,'signed_in');assert.equal(first.secure,false);assert.equal(first.warning,'unavailable');
  assert.equal(app.values.get('tbg.auth.v1.logout_pending'),undefined,'a network failure is not a logout request');
  const recovered=await app.run({kind:'refresh'});
  assert.equal(recovered.kind,'signed_in');assert.equal(recovered.secure,true);assert.equal(recovered.warning,undefined);
  assert.equal(logouts,1,'retry must renew the current family, not sign it out');
});

test('BR-AS11: blocked cookies remain an explicit storage failure',async()=>{
  const app=setup({fetch:async(path)=>path.endsWith('/confirm')?response({error:'auth_required'},401):response(grant())});
  const result=await app.run({kind:'login',input:'Synthetic-0000-0000-0000-0000'});
  assert.equal(result.kind,'signed_in');assert.equal(result.secure,false);assert.equal(result.warning,'storage_unavailable');
  assert.equal(app.values.get('tbg.auth.v1.logout_pending'),'1');
});
