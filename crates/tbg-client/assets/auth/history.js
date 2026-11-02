// Bounded same-origin bearer requests; refresh stays in the existing auth coordinator.
export async function historyRequest(raw, callback) {
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 2000);
  try {
    const q = JSON.parse(raw), url = new URL(q.url);
    if (url.origin !== location.origin || !/^\/api\/(me|players)\//.test(url.pathname)) throw Error();
    const headers = {'Content-Type':'application/json','X-TBG-Auth':'1','Cache-Control':'no-store'};
    if (q.access) headers.Authorization = `Bearer ${q.access}`;
    if (q.slot) headers['X-TBG-Slot'] = q.slot;
    if (q.revision !== null) headers['If-Match'] = `"${q.revision}"`;
    const response = await fetch(url, {method:q.method, headers, body:q.body === null ? undefined : JSON.stringify(q.body), credentials:'omit', cache:'no-store', redirect:'error', signal:controller.signal});
    const reader = response.body.getReader(), parts = []; let size = 0;
    while (true) { const {done,value} = await reader.read(); if (done) break; size += value.byteLength; if (size > 262144) { await reader.cancel(); throw Error(); } parts.push(value); }
    const data = new Uint8Array(size); let at = 0;
    for (const part of parts) { data.set(part,at); at += part.byteLength; }
    callback(JSON.stringify({status:response.status,value:JSON.parse(new TextDecoder().decode(data))}));
  } catch { callback('{}'); }
  finally { clearTimeout(timeout); }
}
export function historyTime(seconds, locale) {
  const date = new Date(seconds * 1000);
  const fields = Object.fromEntries(new Intl.DateTimeFormat(locale, {year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',hourCycle:'h23'}).formatToParts(date).map(p => [p.type,p.value]));
  const offset = -date.getTimezoneOffset(), absolute = Math.abs(offset);
  const pad = n => String(n).padStart(2,'0');
  const zone = `${offset < 0 ? '-' : '+'}${pad(Math.floor(absolute/60))}:${pad(absolute%60)}`;
  const day = locale === 'pt-BR' ? `${fields.day}/${fields.month}/${fields.year}` : `${fields.month}/${fields.day}/${fields.year}`;
  return `${day} ${fields.hour}:${fields.minute} ${zone}`;
}
