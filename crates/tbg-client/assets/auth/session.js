// One memory-only coordinator is shared by the Bevy and HTML entry points.
const symbol = Symbol.for('org.tupi.TBG.auth.v1');
const state = globalThis[symbol] ||= {epoch:0, grant:null, access:null, device:null, slot:null};
const prefix = 'tbg.auth.v1.';
const hex = bytes => [...crypto.getRandomValues(new Uint8Array(bytes))].map(v=>v.toString(16).padStart(2,'0')).join('');
const failure = error => {throw new Error(error);};
const read = key => {try{return localStorage.getItem(prefix+key);}catch{return null;}};
const write = (key,value) => {try{if(value===null)localStorage.removeItem(prefix+key);else localStorage.setItem(prefix+key,value);return localStorage.getItem(prefix+key)===value;}catch{return false;}};
const legacy = () => {try{return localStorage.getItem('tbg_account_key');}catch{return null;}};
const supported = () => !!navigator.locks?.request && write('available','1') && isSecureContext;

async function post(operation,body,access,credentials) {
  const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),5000);
  try {
    const headers={'Content-Type':'application/json','X-TBG-Auth':'1','Cache-Control':'no-store'};
    if(access)headers.Authorization=`Bearer ${access}`;
    // P66: auth follows the release generation that holds this tab's socket.
    if(/^[a-z]$/.test(state.slot||''))headers['X-TBG-Slot']=state.slot;
    const response=await fetch(`/api/auth/browser/${operation}`,{method:'POST',headers,body:JSON.stringify(body),credentials,cache:'no-store',redirect:'error',signal:controller.signal});
    if(response.status===404||response.status>=300&&response.status<400)failure('incompatible');
    if(!response.body)failure('incompatible');
    const reader=response.body.getReader();let size=0,chunks=[];
    while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>4096){await reader.cancel();failure('incompatible');}chunks.push(value);}
    const bytes=new Uint8Array(size);let offset=0;for(const chunk of chunks){bytes.set(chunk,offset);offset+=chunk.length;}
    let value;try{value=JSON.parse(new TextDecoder().decode(bytes));}catch{failure('incompatible');}
    if(response.status!==200)failure(typeof value.error==='string'?value.error:'incompatible');
    return value;
  } catch(error) {
    const known=['incompatible','auth_required','name_taken','name_invalid','rate_limited','conflict','recovery_required','in_match','capacity','family_limit','unavailable','storage_unavailable'];
    failure(known.includes(error.message)?error.message:'unavailable');
  } finally {clearTimeout(timer);}
}

async function retry(operation,body,access,credentials) {
  try{return await post(operation,body,access,credentials);}catch(error){
    if(error.message==='unavailable'&&['refresh','register','confirm'].includes(operation))return post(operation,body,access,credentials);
    throw error;
  }
}
function notify() {try{const channel=new BroadcastChannel(prefix+'changed');channel.postMessage({changed:true});channel.close();}catch{/* Nonsecret advisory only; locks enforce correctness. */}}

async function logout(credentials,access) {
  const durable=credentials!=='omit';
  const marked=!durable||write('logout_pending','1');
  try {
    await post('logout',{},access,credentials);
    if(durable&&!write('logout_pending',null))failure('unavailable');
    state.grant=null;state.access=null;state.recoveryKey=null;notify();return {kind:'logged_out'};
  }catch(error){
    // An unrecognized cookie cannot restore an account; a lost successful reply is safe to retry.
    if(error.message==='auth_required'&&marked){if(durable)write('logout_pending',null);state.grant=null;state.access=null;state.recoveryKey=null;return {kind:'logged_out'};}
    return {kind:'logout_pending',error:error.message};
  }
}

async function run(request) {
  const durable=supported(),credentials=durable?'same-origin':'omit';
  const current=()=>state.epoch===request.epoch;
  const operation=request.kind;
  if(request.origin!==location.origin)failure('invalid_origin');
  if(operation==='recovery-key') {
    const name=state.grant?.profile?.name;
    if(!name||!state.access)failure('auth_required');
    if(request.input) {
      if(!request.input.toLowerCase().startsWith(name.toLowerCase()+'-'))failure('auth_required');
      state.recoveryKey=request.input;
    }
    return {kind:'recovery_key',key:state.recoveryKey||null};
  }
  if(read('logout_pending')==='1'&&operation!=='logout') {
    const result=await logout('same-origin',state.access);
    if(result.kind!=='logged_out')return result;
    if(operation==='restore')return {kind:'guest',legacy:legacy()};
  }
  if(operation==='logout')return logout(credentials,state.access);
  if(operation==='backup') {
    if(!durable||!state.grant) return {kind:'backup',migrated:false};
    const proof=await post('confirm',{},null,credentials);
    if(proof.session_id!==state.grant.session_id)failure('conflict');
    if(!current())failure('cancelled');
    const key=legacy();
    if(key&&key.toLowerCase().startsWith(state.grant.profile.name.toLowerCase()+'-')) {
      try{if(localStorage.getItem('tbg_account_key')===key)localStorage.removeItem('tbg_account_key');if(localStorage.getItem('tbg_account_key')===key)failure('unavailable');}catch{failure('unavailable');}
    }
    return {kind:'backup',migrated:true};
  }
  if(operation==='restore'&&!durable)return {kind:'guest',legacy:legacy()};
  if(['login','register'].includes(operation)) {
    // Explicit account switches finish revocation before a new cookie is installed.
    if(durable||state.access){const out=await logout(credentials,state.access);if(out.kind!=='logged_out')return out;}
  }
  state.device ||= read('device')||hex(16);
  if(durable&&!write('device',state.device))failure('unavailable');
  let grant;
  if(operation==='restore'||operation==='refresh') {
    if(!durable)failure('recovery_required');
    const id=read('operation')||hex(16);
    if(!write('operation',id))failure('unavailable');
    try{grant=await retry('refresh',{operation_id:id},null,credentials);}catch(error){if(operation==='restore'&&error.message==='auth_required')return {kind:'guest',legacy:legacy()};throw error;}
    write('operation',null);
  }else{
    const body={device_id:state.device,persistent:!!request.remember};
    if(operation==='login')body.account_key=request.input;
    else if(operation==='register'){body.name=request.input;body.registration_retry_key=hex(32);}
    else failure('incompatible');
    grant=await retry(operation,body,null,credentials);
  }
  if(!current()) {
    const out=await logout(credentials,grant.access_token);
    if(out.kind!=='logged_out'){write('logout_pending','1');failure('revocation_pending');}
    failure('cancelled');
  }
  let secure=false,warning=durable?null:'storage_unavailable';
  if(durable) {
    try{const proof=await retry('confirm',{},null,credentials);secure=proof.session_id===grant.session_id;if(!secure)warning='conflict';}catch(error){warning=error.message==='auth_required'?'storage_unavailable':error.message;}
    // A transport outage does not prove cookies are blocked or require logout.
    if(!secure&&warning==='storage_unavailable')write('logout_pending','1');
  }
  state.grant={...grant};delete state.grant.account_key;state.access=grant.access_token;
  notify();
  return {kind:'signed_in',grant,secure,...(warning?{warning}:{}),device:state.device};
}

export function startAuth(raw,callback) {
  let request;try{request=JSON.parse(raw);}catch{callback(JSON.stringify({error:'incompatible'}));return;}
  state.epoch=Math.max(state.epoch,request.epoch);
  state.slot=typeof request.slot==='string'&&/^[a-z]$/.test(request.slot)?request.slot:null;
  const execute=()=>run(request).catch(async error=>{
    if(['login','register'].includes(request.kind)&&error.message==='unavailable') {write('logout_pending','1');return {kind:'logout_pending',error:'unavailable'};}
    if(request.epoch!==state.epoch&&['login','register','refresh'].includes(request.kind)) {
      const out=await logout(supported()?'same-origin':'omit',state.access);
      if(out.kind!=='logged_out')failure('revocation_pending');
      failure('cancelled');
    }
    throw error;
  });
  const work=supported()?navigator.locks.request('org.tupi.TBG.auth.origin.v1',{mode:'exclusive'},execute):execute();
  Promise.resolve(work).then(value=>callback(JSON.stringify(value)),error=>callback(JSON.stringify({error:error.message||'unavailable'})));
}


export function invalidateEpoch(epoch) {state.epoch=Math.max(state.epoch,epoch);}
