// Same-origin HTTP reads with a timeout and a size limit. Public data only: no credentials are sent.

export class HttpError extends Error {
  constructor(status, path) {
    super(`${path}: HTTP ${status}`);
    this.status = status;
  }
}

async function read(path, { timeoutMs = 5000, maxBytes = 64 * 1024, accept = 'application/json' } = {}) {
  // Absolute or protocol-relative URLs would leave the origin; the admin never calls third parties.
  if (!path.startsWith('/') || path.startsWith('//')) throw new Error(`same-origin path required: ${path}`);
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(path, { credentials: 'omit', signal: controller.signal, headers: { Accept: accept } });
    if (!response.ok) throw new HttpError(response.status, path);
    const body = await response.text();
    if (body.length > maxBytes) throw new Error(`${path}: response too large`);
    return body;
  } finally {
    clearTimeout(timer);
  }
}

export async function getJson(path, options) {
  return JSON.parse(await read(path, options));
}

export async function getText(path, options) {
  return read(path, { ...options, accept: 'text/plain' });
}
