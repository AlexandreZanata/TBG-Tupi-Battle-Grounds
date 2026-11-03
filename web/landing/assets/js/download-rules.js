// Pure download eligibility and OS hints; unsupported or incompatible artifacts stay unavailable.
const PREFIX = 'https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/';
const SUFFIXES = { appimage: '.AppImage', tarball: '.tar.gz', installer: '-setup.exe', zip: '.zip', apk: '.apk' };
const KINDS = { linux: ['appimage', 'tarball'], windows: ['installer', 'zip'], android: ['apk'] };
const ANDROID_CERTIFICATE = '5b0762f8270e29e73466305224b3315ffc69bea77cc8c828b7a5b1999696f168';

export function detectedOS(platform = '', agent = '') {
  const value = `${platform} ${agent}`;
  if (/Android|iPhone|iPad|iPod/i.test(value)) return null;
  if (/Windows|Win32|Win64/i.test(value)) return 'windows';
  if (/Linux|X11/i.test(value)) return 'linux';
  return null;
}

export function available(config, os) {
  const item = config?.[os];
  const server = config?.server;
  if (!KINDS[os] || item?.state !== 'available' || !/^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?$/.test(item.version ?? '')) return false;
  if (!Number.isInteger(server?.protocol) || item.protocol !== server.protocol || item.contract !== server.contract) return false;
  if (!/^w\d+-r\d+-[0-9a-f]{16}$/.test(server.contract ?? '') || !server.contract.startsWith(`w${server.protocol}-`) || !Array.isArray(item.assets)) return false;
  if (os === 'android' && (item.channel !== 'beta' || item.abi !== 'arm64-v8a'
    || item.package !== 'io.github.alexandrezanata.tbg.prototype' || item.debuggable !== false
    || item.certificate_sha256 !== ANDROID_CERTIFICATE || item.min_sdk !== 31 || item.target_sdk !== 36
    || !Number.isInteger(item.version_code) || item.version_code <= 54
    || !item.version.endsWith(`-android.${item.version_code}`) || item.tag !== 'android-v' + item.version)) return false;
  const kinds = new Set();
  for (const asset of item.assets) {
    if (!KINDS[os].includes(asset?.kind) || kinds.has(asset.kind)
      || !/^[0-9a-f]{64}$/.test(asset.sha256 ?? '') || !Number.isSafeInteger(asset.bytes) || asset.bytes <= 0) return false;
    const expected = PREFIX + (os === 'android' ? 'android-v' : 'v') + item.version + '/';
    const architecture = os === 'android' ? 'arm64-v8a' : 'x86_64';
    const filename = `tupi-battle-grounds-${item.version}-${os}-${architecture}${SUFFIXES[asset.kind]}`;
    if (asset.name !== filename || asset.url !== expected + filename) return false;
    kinds.add(asset.kind);
  }
  return KINDS[os].every(kind => kinds.has(kind));
}

export function mergeCatalog(base, catalog) {
  if (catalog?.schema !== 1 || !Number.isInteger(catalog.server?.protocol)
    || !/^w\d+-r\d+-[0-9a-f]{16}$/.test(catalog.server?.contract ?? '')) return base;
  const result = { ...base, server: catalog.server };
  for (const os of Object.keys(KINDS)) result[os] = catalog[os] ?? { state: 'pending' };
  return result;
}
