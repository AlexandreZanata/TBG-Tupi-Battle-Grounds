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
  function render(){
    const key=label.textContent==='Paste your account key';
    form.dataset.entry=key?'key':'name';
    title.textContent=key?'Welcome back':'Create your account';
    help.textContent=key?'Use your saved account key to return to your ranked record.':'Choose your player name and claim your place in the Grounds.';
    submit.textContent=key?'Sign in':'Create account';
    input.type=key?'password':'text';
    reveal.hidden=!key;reveal.textContent='Show key';reveal.setAttribute('aria-pressed','false');
  }
  reveal.addEventListener('click',()=>{
    const shown=input.type==='password';input.type=shown?'text':'password';
    reveal.textContent=shown?'Hide key':'Show key';reveal.setAttribute('aria-pressed',String(shown));
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
  new MutationObserver(render).observe(form,{attributes:true,attributeFilter:['hidden']});
  render();
})();
