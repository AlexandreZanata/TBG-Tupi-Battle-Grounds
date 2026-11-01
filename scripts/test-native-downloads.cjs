// Download contracts: recommend the detected desktop OS while keeping all choices accessible.
const assert = require('node:assert/strict');
const fs = require('node:fs');
(async () => {
  const code = fs.readFileSync('web/landing/assets/js/download-rules.js', 'utf8');
  const { available, detectedOS, mergeCatalog } = await import('data:text/javascript;base64,' + Buffer.from(code).toString('base64'));
  assert.equal(detectedOS('Win32', 'Windows NT 10.0'), 'windows');
  assert.equal(detectedOS('Linux x86_64', 'Firefox'), 'linux');
  assert.equal(detectedOS('Linux', 'Android'), null);
  assert.equal(detectedOS('MacIntel', 'Safari'), null);
  const prefix = 'https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/v0.1.2-rc.1/';
  const suffix = {appimage: '.AppImage', tarball: '.tar.gz', installer: '-setup.exe', zip: '.zip'};
  const item = (os, kinds) => ({ state: 'available', version: '0.1.2-rc.1', protocol: 31, contract: 'w31-r6-097385fee9a10b0f',
    assets: kinds.map(kind => {const name = `tupi-battle-grounds-0.1.2-rc.1-${os}-x86_64${suffix[kind]}`; return {kind, name, bytes: 1000, sha256: 'a'.repeat(64), url: prefix + name};}) });
  const value = {server: {protocol: 31, contract: 'w31-r6-097385fee9a10b0f'}, linux: item('linux', ['appimage', 'tarball']), windows: item('windows', ['installer', 'zip'])};
  assert(available(value, 'linux')); assert(available(value, 'windows'));
  for (const os of ['linux', 'windows']) {
    const changed = () => structuredClone(value);
    let v = changed(); v[os].protocol = 30; assert(!available(v, os));
    v = changed(); v[os].contract = 'w31-r2-0000000000000000'; assert(!available(v, os));
    v = changed(); v[os].assets.pop(); assert(!available(v, os));
    v = changed(); v[os].assets[1] = v[os].assets[0]; assert(!available(v, os));
    v = changed(); v[os].assets[0].url = 'https://evil.example/game.exe'; assert(!available(v, os));
    v = changed(); v[os].assets[0].url = prefix + '../wrong.exe'; assert(!available(v, os));
    v = changed(); v[os].assets[0].url = prefix + 'other.exe'; assert(!available(v, os));
    v = changed(); v[os].assets[0].bytes = -1; assert(!available(v, os));
    v = changed(); v[os].assets[0].sha256 = 'bad'; assert(!available(v, os));
  }
  const next = mergeCatalog(value, {schema: 1, server: {protocol: 32, contract: 'w32-r6-097385fee9a10b0f'}, linux: {state:'pending'}, windows: {state:'pending'}});
  assert(!available(next, 'linux')); assert(!available(next, 'windows'));
  assert.equal(mergeCatalog(value, {schema: 9}), value);
  console.log('Native download rules passed: OS hints, complete packages, origin/digest and live-contract rejection');
})().catch(error => {console.error(error); process.exit(1);});
