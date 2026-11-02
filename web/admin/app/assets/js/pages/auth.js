// Email claim, key-plus-code sign-in, and one-time AdminKey display.
import {el} from '../../shared/dom.js';
import {t} from '../i18n.js';
import {request} from '../api/admin.js';
import {form,field,button,card} from '../ui/forms.js';
export function auth(root,run,ready) {
  const login=async body=>run(async()=>{
    const reply=await request('session/start',{method:'POST',body});
    const verify=async data=>run(async()=>{
      const session=await request('session/verify',{method:'POST',body:{attempt:reply.attempt,code:data.code}});
      ready(session);
    });
    root.replaceChildren(card(t('login_code'),form([field('code','code')],'verify',verify)));
  });
  const send=async body=>run(async()=>{
    await request('claim/request',{method:'POST',body});
    root.append(el('p',{role:'status',text:t('generic')}));
  });
  const complete=async body=>run(async()=>{
    const result=await request('claim/complete',{method:'POST',body});
    const key=el('textarea',{readonly:true,rows:3,'aria-label':t('new_key')});
    key.textContent=result.admin_key;
    const saved=()=>{key.textContent='';root.replaceChildren();auth(root,run,ready);};
    root.replaceChildren(card(t('new_key'),el('p',{text:t('save_key')}),key,button('saved',saved)));
  });
  root.append(
    card(t('signin'),form([field('admin_key','admin_key','password')],'send_login',login)),
    card(t('claim'),form([field('email','email','email')],'send_claim',send),form([field('email','email','email'),field('code','code')],'claim_complete',complete))
  );
}
export function reauth(root,run,done) {
  const dialog=el('dialog',{},el('h2',{text:t('reauth')}));
  const submit=async body=>run(async()=>{await request('session/reauth',{method:'POST',body});dialog.close();done();});
  dialog.append(form([field('admin_key','admin_key','password')],'confirm',submit),button('cancel',()=>dialog.close()));
  dialog.addEventListener('close',()=>dialog.remove());root.append(dialog);dialog.showModal();
}
