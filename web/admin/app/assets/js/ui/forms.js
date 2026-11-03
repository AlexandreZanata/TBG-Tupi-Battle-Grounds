// Text-only widgets and explicit, reasoned confirmations for console actions.
import {el} from '../../shared/dom.js';
import {t} from '../i18n.js';
export const button=(key,run,kind='')=>{const b=el('button',{type:'button',class:`btn ${kind}`,text:t(key)});b.addEventListener('click',run);return b;};
export function field(key,name,type='text',value=''){const input=el(type==='textarea'?'textarea':'input',{name,type:type==='textarea'?null:type,value:type==='textarea'?null:value,autocomplete:'off',required:true,maxlength:type==='textarea'?500:254});if(type==='textarea')input.textContent=value;return el('label',{class:'field'},el('span',{text:t(key)}),input);}
export function form(fields,key,submit){const f=el('form',{class:'form'},...fields,el('button',{type:'submit',class:'btn',text:t(key)}));f.addEventListener('submit',async e=>{e.preventDefault();const controls=[...f.querySelectorAll('button')];controls.forEach(b=>b.disabled=true);try{await submit(Object.fromEntries(new FormData(f)));}finally{controls.forEach(b=>b.disabled=false);}});return f;}
export function select(key,name,values){const s=el('select',{name});for(const value of values)s.append(el('option',{value,text:t(`reason.${value}`)}));return el('label',{class:'field'},el('span',{text:t(key)}),s);}
export function card(title,...nodes){return el('section',{class:'panel'},el('h2',{text:title}),...nodes);}
export function facts(value){const list=el('dl',{class:'facts'});for(const [key,data]of Object.entries(value||{})){list.append(el('dt',{text:t(`field.${key}`)}),el('dd',{text:typeof data==='object'?JSON.stringify(data):String(data??'—')}));}return list;}
export function confirmAction(title,run){const dialog=el('dialog',{},el('h2',{text:title}),el('p',{text:t('confirm.impact')}));dialog.append(button('cancel',()=>dialog.close()),button('confirm',async()=>{dialog.close();await run();},'danger'));dialog.addEventListener('close',()=>dialog.remove());document.body.append(dialog);dialog.showModal();}
export const reasons=['cheating','abusive_chat','offensive_name','offensive_emoji','harassment','ban_evasion','impersonation','other'];
