// Verify the exact browser helper shipped by Rust, including rejected Promises.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import vm from 'node:vm';

const config = await readFile(new URL('../crates/tbg-client/src/config.rs', import.meta.url), 'utf8');
const source = config.match(/inline_js = r#"([\s\S]*?)"#/)[1].replaceAll('export function', 'function');
const url = 'https://example.test/?challenge=' + 'ab'.repeat(16);
const cases = [
  { name: 'no share API', copies: 1 },
  { name: 'unsupported data', canShare: () => false, share: () => assert.fail('unsupported data'), copies: 1 },
  { name: 'share without canShare', share: () => Promise.resolve(), copies: 0 },
  { name: 'dismissed share sheet', share: () => Promise.reject({ name: 'AbortError' }), copies: 0 },
  { name: 'share denied', share: () => Promise.reject({ name: 'NotAllowedError' }), copies: 1 },
  { name: 'synchronous share failure', share: () => { throw new Error('unsupported'); }, copies: 1 },
  { name: 'clipboard denied', clipboardDenied: true, copies: 1 },
];
for (const scenario of cases) {
  const copied = [];
  const navigator = {
    clipboard: { writeText(text) {
      copied.push(text);
      return scenario.clipboardDenied ? Promise.reject(new Error('denied')) : Promise.resolve();
    } },
    ...(scenario.share ? { share: scenario.share } : {}),
    ...(scenario.canShare ? { canShare: scenario.canShare } : {}),
  };
  const context = vm.createContext({ navigator });
  vm.runInContext(source, context);
  context.tbgShareText('Join my TBG challenge', url);
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(copied.length, scenario.copies, scenario.name);
  assert.ok(copied.every(text => text === url), scenario.name);
}
// Let Node report any unhandled rejection before declaring success.
await new Promise(resolve => setImmediate(resolve));
console.log(`${cases.length} share/clipboard scenarios passed; no unhandled rejections`);
