//! Present the existing Rust account form without reading keys or sending account actions.
(() => {
  'use strict';
  const form=document.getElementById('tbg-form');
  if(!form)return;
  const label=document.getElementById('tbg-form-label');
  const title=document.getElementById('account-title');
  const help=document.getElementById('account-help');
  const input=document.getElementById('tbg-input');
  const reveal=form.querySelector('[data-account-reveal]');
  const submit=document.getElementById('tbg-form-ok');
  const t=key=>globalThis.TBG_I18N.t(key);
  // Rust marks the form kind (`data-kind`); the label text is localized.
  function render(){
    const key=form.dataset.kind==='key';
    form.dataset.entry=key?'key':'name';
    title.textContent=t(key?'account.form.welcome':'account.form.create');
    help.textContent=t(key?'account.form.key_help':'account.form.name_help');
    submit.textContent=t(key?'account.form.sign_in':'lobby.account.create');
    input.type=key?'password':'text';
    reveal.hidden=!key;reveal.textContent=t('account.form.show_key');reveal.setAttribute('aria-pressed','false');
  }
  reveal.addEventListener('click',()=>{
    const shown=input.type==='password';input.type=shown?'text':'password';
    reveal.textContent=t(shown?'account.form.hide_key':'account.form.show_key');reveal.setAttribute('aria-pressed',String(shown));
  });
  form.addEventListener('keydown',event=>{
    if(event.key!=='Tab')return;
    const controls=[...form.querySelectorAll('input,button')].filter(n=>!n.hidden&&!n.disabled);
    const first=controls[0],last=controls[controls.length-1];
    if(event.shiftKey&&document.activeElement===first){event.preventDefault();last.focus();}
    else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}
  });
  // Rust replaces the label and toggles hidden on every open; reset visibility each time.
  new MutationObserver(render).observe(label,{childList:true,characterData:true,subtree:true});
  new MutationObserver(render).observe(form,{attributes:true,attributeFilter:['hidden','data-kind']});
  document.addEventListener('tbg:lang',render);
  render();
})();
