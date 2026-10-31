import fs from 'node:fs/promises';

export async function connect(port = 9331) {
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  const socket = new WebSocket(targets.find(target => target.type === 'page').webSocketDebuggerUrl);
  await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
  let sequence = 0;
  const pending = new Map();
  const events = [];
  socket.onmessage = ({data}) => {
    const message = JSON.parse(data);
    if (!message.id) { events.push(message); return; }
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    clearTimeout(request.timer);
    if (message.error) request.reject(new Error(JSON.stringify(message.error)));
    else request.resolve(message.result);
  };
  const send = (method, params = {}) => new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => {pending.delete(id);reject(new Error(`Timed out: ${method}`));}, 45000);
    pending.set(id, {resolve, reject, timer});
    socket.send(JSON.stringify({id, method, params}));
  });
  await send('Page.enable');
  await send('Runtime.enable');
  await send('Log.enable');
  return {
    send, events,
    evaluate: async expression => {
      const result = await send('Runtime.evaluate', {expression, returnByValue: true, awaitPromise: true});
      if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
      return result.result.value;
    },
    screenshot: async path => {
      const {data} = await send('Page.captureScreenshot', {format: 'jpeg', quality: 80});
      await fs.writeFile(path, Buffer.from(data, 'base64'));
    },
    close: () => socket.close(),
  };
}
