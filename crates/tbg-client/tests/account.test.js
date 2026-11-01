//! Account presentation keeps Rust's single form/input and never reads or submits the key.
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const source=fs.readFileSync(path.resolve(__dirname,'../assets/lobby/account.js'),'utf8');

function fixture(){
  const node=()=>({textContent:'',dataset:{},hidden:false,listeners:{},addEventListener(k,f){this.listeners[k]=f;},setAttribute(k,v){this[k]=v;}});
  const names=['tbg-form','tbg-form-label','account-title','account-help','tbg-input','tbg-form-ok'];
  const nodes=Object.fromEntries(names.map(id=>[id,node()]));
  const reveal=node(),observers=[];
  nodes['tbg-form'].querySelector=()=>reveal;
  Object.defineProperty(nodes['tbg-input'],'value',{get(){throw new Error('Presentation must never read credentials');}});
  const {catalogs}=require('./i18n-runtime.js');
  const TBG_I18N={t:key=>catalogs.en[key]??key};
  const context={TBG_I18N,document:{getElementById:id=>nodes[id],addEventListener(){}},MutationObserver:class{constructor(f){this.render=f;}observe(n){observers.push({node:n,render:this.render});}},fetch(){throw new Error('Presentation must not call an account API');}};
  vm.runInNewContext(source,context);
  return {nodes,reveal,update(){observers.forEach(o=>o.render());}};
}

test('existing Name and Key forms gain clear copy, masked keys, and reset reveal state on reopen',()=>{
  const f=fixture();
  assert.equal(f.nodes['tbg-form-ok'].textContent,'Create account');
  assert.equal(f.nodes['tbg-input'].type,'text');assert.equal(f.reveal.hidden,true);
  // P64: Rust marks the kind; a localized label (any language) is never parsed.
  f.nodes['tbg-form-label'].textContent='Cole a chave da sua conta';f.nodes['tbg-form'].dataset.kind='key';f.update();
  assert.equal(f.nodes['account-title'].textContent,'Welcome back');
  assert.equal(f.nodes['tbg-form-ok'].textContent,'Sign in');
  assert.equal(f.nodes['tbg-input'].type,'password');assert.equal(f.reveal.hidden,false);
  f.reveal.listeners.click();assert.equal(f.nodes['tbg-input'].type,'text');assert.equal(f.reveal['aria-pressed'],'true');
  f.update();assert.equal(f.nodes['tbg-input'].type,'password');assert.equal(f.reveal['aria-pressed'],'false');
  f.nodes['tbg-form-label'].textContent='Choose a name (3-16 letters, digits, _)';f.nodes['tbg-form'].dataset.kind='name';f.update();
  assert.equal(f.nodes['tbg-input'].type,'text');assert.equal(f.reveal.hidden,true);
});

test('the redesigned shell retains every Rust account binding exactly once',()=>{
  const html=fs.readFileSync(path.resolve(__dirname,'../index.html'),'utf8');
  for(const id of ['tbg-form','tbg-form-label','tbg-input','tbg-form-error','tbg-form-ok','tbg-form-cancel']){
    assert.equal(html.split('id="'+id+'"').length-1,1,id);
  }
  assert.match(html,/<form id="tbg-form"[^>]+role="dialog"[^>]+hidden>/);
  assert.match(html,/<button id="tbg-form-ok" type="submit">/);
  assert.match(html,/<button id="tbg-form-cancel" type="button">/);
});
