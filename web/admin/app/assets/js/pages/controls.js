// Explicit audited configuration actions, with ordinary text rendering.
import {el} from '../../shared/dom.js';
import {t} from '../i18n.js';
import {request} from '../api/admin.js';
import {button,card,field,form,select,reasons,facts} from '../ui/forms.js';
export async function controls(root,ctx,area) {
  const path=area==='terms'?'chat/terms':area;
  const data=await request(path);root.replaceChildren();
  if(area==='settings') {
    for(const row of data.items||[]) {
      const toggle=body=>ctx.run(async()=>{
        await request(`settings/${row.key}`,{method:'PUT',body:{...body,value:row.value?0:1}});
        await controls(root,ctx,area);
      });
      root.append(card(t(`setting.${row.key}`),el('p',{text:t(row.value?'enabled':'disabled')}),form([select('reason','reason',reasons)],row.value?'disable':'enable',toggle)));
    }
    return;
  }
  const fieldName=area==='terms'?'term':'email';
  const add=body=>ctx.run(async()=>{await request(path,{method:'POST',body});await controls(root,ctx,area);});
  root.append(form([field(fieldName,fieldName,fieldName==='email'?'email':'text'),select('reason','reason',reasons)],'add',add));
  for(const row of data.items||[]) {
    const remove=body=>ctx.run(async()=>{await request(`${path}/${encodeURIComponent(row.value)}`,{method:'DELETE',body});await controls(root,ctx,area);});
    root.append(card(row.value,form([select('reason','reason',reasons)],'remove',remove)));
  }
}
export async function operators(root,ctx) {
  const data=await request('operators');
  const invite=body=>ctx.run(async()=>{await request('operators',{method:'POST',body:{...body,role:'moderator'}});await operators(root,ctx);});
  root.replaceChildren(form([field('email','email','email')],'invite',invite));
  for(const row of data.items||[]) {
    const item=card(row.email_masked,facts({role:row.role,status:row.status}));
    if(row.role!=='owner') {
      const action=row.status==='disabled'?'enable':'disable';
      const update=body=>ctx.run(async()=>{await request(`operators/${row.id}/${action}`,{method:'POST',body});await operators(root,ctx);});
      item.append(form([select('reason','reason',reasons)],action,update));
    }
    root.append(item);
  }
}
export async function audit(root,ctx,cursor='') {
  const data=await request(`audit${cursor?`?cursor=${cursor}`:''}`);root.replaceChildren();
  for(const row of data.items||[])root.append(card(`${row.id} · ${row.action}`,facts(row)));
  if(data.cursor)root.append(button('more',()=>ctx.run(()=>audit(root,ctx,data.cursor))));
}
export async function read(root,ctx,area) {
  const data=await request(area);root.replaceChildren(card(t(`nav.${area}`),facts(data)));
}
