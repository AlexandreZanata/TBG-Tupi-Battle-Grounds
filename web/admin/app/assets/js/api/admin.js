// Same-origin operator API; secrets live only in HttpOnly cookies or transient form values.
let csrf = '';
export class ApiError extends Error { constructor(status, code) { super(code); this.status=status; this.code=code; } }
async function read(response,max=131072) {
  const reader=response.body.getReader(),parts=[];let size=0;
  while(true){const {done,value}=await reader.read();if(done)break;size+=value.byteLength;if(size>max){await reader.cancel();throw new ApiError(503,'response_limit');}parts.push(value);}
  const bytes=new Uint8Array(size);let at=0;for(const part of parts){bytes.set(part,at);at+=part.length;}return bytes;
}
export async function request(path,{method='GET',body=null}={}) {
  if(!/^[a-z][a-z0-9/-]*(?:\?[^#]*)?$/.test(path)&&!/^accounts\/[A-Za-z][A-Za-z0-9_]{2,15}(?:\/[a-z-]+)?$/.test(path)&&!/^chat\/terms\/[^/#]+$/.test(path)&&!/^email-blocks\/[^/#]+$/.test(path))throw new ApiError(400,'invalid_route');
  const headers={'Accept':'application/json','Content-Type':'application/json'};
  if(method!=='GET')headers['X-TBG-CSRF']=csrf;
  const controller=new AbortController(),timer=setTimeout(()=>controller.abort(),5000);
  try {
    const response=await fetch(`/admin-api/v1/${path}`,{method,headers,body:method==='GET'?undefined:JSON.stringify(body||{}),credentials:'same-origin',cache:'no-store',redirect:'error',signal:controller.signal});
    const bytes=await read(response);const value=JSON.parse(new TextDecoder().decode(bytes));if(typeof value.csrf==='string')csrf=value.csrf;
    if(!response.ok)throw new ApiError(response.status,value.error||'unavailable');return value;
  }finally{clearTimeout(timer);}
}
export async function sheet(id) {
  if(!Number.isSafeInteger(id)||id<1)throw new ApiError(400,'invalid_asset');
  const response=await fetch(`/admin-api/v1/emoji/${id}/sheet.png`,{credentials:'same-origin',cache:'no-store',redirect:'error'});
  if(!response.ok)throw new ApiError(response.status,'sheet_unavailable');const bytes=await read(response,262144);
  if(bytes.length<24||bytes[0]!==137||String.fromCharCode(...bytes.slice(1,4))!=='PNG')throw new ApiError(422,'sheet_invalid');
  const data=new DataView(bytes.buffer);if(data.getUint32(16)>768||data.getUint32(20)>512)throw new ApiError(422,'sheet_dimensions');return createImageBitmap(new Blob([bytes],{type:'image/png'}));
}
