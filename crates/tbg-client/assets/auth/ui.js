// Public auth state only: credentials never enter this DOM or its event channel.
let listener=null,signature='';
export function bindAuthUi(callback) {
  listener=callback;
  const remember=document.getElementById('tbg-auth-remember');
  remember?.addEventListener('change',()=>listener?.(`remember:${remember.checked}`));
}
export function showAuthUi(raw) {
  const view=JSON.parse(raw),lobby=document.getElementById('tbg-lobby'),form=document.getElementById('tbg-form');
  let root=document.getElementById('tbg-auth-status');
  if(!root){root=document.createElement('aside');root.id='tbg-auth-status';root.className='auth-status metal-panel';root.setAttribute('role','status');root.setAttribute('aria-live','polite');document.body.append(root);}
  root.hidden=!view.visible||!lobby||lobby.hidden||(form&&!form.hidden);
  const checkbox=document.getElementById('tbg-auth-remember');
  if(checkbox)checkbox.checked=view.remember;
  const option=document.getElementById('tbg-auth-remember-option');
  if(option)option.hidden=!['key','name'].includes(form?.dataset.kind);
  if(signature===raw)return;signature=raw;
  root.replaceChildren();
  for(const text of [view.message,view.error]){if(!text)continue;const line=document.createElement('p');line.textContent=text;root.append(line);}
  for(const item of view.actions){const button=document.createElement('button');button.type='button';button.textContent=item.label;button.addEventListener('click',()=>listener?.(item.action));root.append(button);}
}
