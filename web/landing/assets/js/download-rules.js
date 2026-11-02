// Pure download eligibility and OS hints; unsupported or incompatible artifacts stay unavailable.
const PREFIX = 'https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/';
const SUFFIXES = { appimage: '.AppImage', tarball: '.tar.gz', installer: '-setup.exe', zip: '.zip' };
const KINDS = { linux: ['appimage', 'tarball'], windows: ['installer', 'zip'] };

export function detectedOS(platform = '', agent = '') {
  const value = `${platform} ${agent}`;
  // Android user agents also name Linux, so they are matched first.
  if (/Android/i.test(value)) return 'android';
  if (/iPhone|iPad|iPod/i.test(value)) return null;
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
  const kinds = new Set();
  for (const asset of item.assets) {
    if (!KINDS[os].includes(asset?.kind) || kinds.has(asset.kind)
      || !/^[0-9a-f]{64}$/.test(asset.sha256 ?? '') || !Number.isSafeInteger(asset.bytes) || asset.bytes <= 0) return false;
    const expected = PREFIX + 'v' + item.version + '/';
    const filename = `tupi-battle-grounds-${item.version}-${os}-x86_64${SUFFIXES[asset.kind]}`;
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
