// Safe DOM construction: text only through textContent, never HTML strings (CSP default-src 'self').

/**
 * Create an element. `props` sets attributes (`class`, `aria-*`, `data-*`, …)
 * except `text`, which becomes textContent; children are nodes or strings.
 */
export function el(tag, props = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (value === undefined || value === null || value === false) continue;
    if (key === 'text') node.textContent = value;
    else node.setAttribute(key, value === true ? '' : String(value));
  }
  for (const child of children.flat()) {
    if (child === undefined || child === null || child === false) continue;
    node.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return node;
}

export function clear(node) {
  node.replaceChildren();
  return node;
}

/** Fetch same-origin JSON with a size guard; rejects on HTTP errors. */
export async function fetchJson(url, { maxBytes = 512 * 1024, signal } = {}) {
  const response = await fetch(url, { credentials: 'same-origin', signal });
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  const body = await response.text();
  if (body.length > maxBytes) throw new Error(`${url}: response too large`);
  return JSON.parse(body);
}
