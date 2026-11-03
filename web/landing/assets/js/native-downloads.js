// Render platform cards and accessible native-download dialogs from verified metadata.
import { el, clear } from '../shared/dom.js';
import { t } from './i18n.js';
import { available, detectedOS } from './download-rules.js';

const PLATFORMS = ['windows', 'linux', 'android'];
// Every format comes from the same source-bound catalog, in recommended-first order.
const ORDER = { windows: ['installer', 'zip'], linux: ['appimage', 'tarball'], android: ['apk'] };
const FORMATS = { installer: '.exe', zip: '.zip', appimage: '.AppImage', tarball: '.tar.gz', apk: '.apk' };
const labels = { installer: 'downloads.install', zip: 'downloads.portable', appimage: 'downloads.appimage', tarball: 'downloads.tarball', apk: 'downloads.apk' };
const descriptions = { installer: 'downloads.windows.install', zip: 'downloads.windows.portable', appimage: 'downloads.linux.appimage', tarball: 'downloads.linux.tarball', apk: 'downloads.android.apk' };
// Text shown while a platform has no verified package.
const PENDING = {
  windows: { status: 'state.preparing', short: 'downloads.pending', long: 'downloads.pending' },
  linux: { status: 'state.preparing', short: 'downloads.pending', long: 'downloads.pending' },
  android: { status: 'state.preparing', short: 'android.pending.short', long: 'android.pending' },
};

let current = null;
let opened = null;
let initialized = false;
const platformName = navigator.userAgentData?.platform ?? navigator.platform;
// Android only highlights its availability card; detectedOS stays desktop-only.
const choice = /Android/i.test(`${platformName} ${navigator.userAgent}`) ? 'android' : detectedOS(platformName, navigator.userAgent);

function option(asset, recommended) {
  const size = t('linux.size', { size: (asset.bytes / 1048576).toFixed(1) });
  return el('article', { class: `native-option${recommended ? ' is-best' : ''}` },
    el('div', { class: 'native-option-body' },
      el('h3', {},
        el('span', { text: t(labels[asset.kind]) }),
        recommended && el('span', { class: 'badge badge-ok', text: t('downloads.best') })),
      el('p', { text: t(descriptions[asset.kind]) }),
      el('p', { class: 'native-option-meta' },
        el('span', { class: 'native-format', text: FORMATS[asset.kind] }), el('span', { text: size }))),
    el('a', { class: `btn ${recommended ? 'btn-play' : 'btn-gold'} native-option-cta`, href: asset.url, rel: 'noopener noreferrer' },
      el('span', { text: t('downloads.get') })),
    el('details', { class: 'native-checksum' },
      el('summary', { text: t('downloads.checksum') }), el('code', { text: asset.sha256 })));
}

function notes(release) {
  const items = [];
  if (opened === 'windows') items.push(el('p', { class: 'native-callout native-callout-warn', text: t('downloads.unsigned') }));
  if (opened === 'android') items.push(el('p', { class: 'native-callout', text: t('downloads.android.beta') }));
  else if (release.visual_acceptance !== 'passed') items.push(el('p', { class: 'native-callout', text: t('downloads.pixels.pending') }));
  const tag = opened === 'android' ? release.tag : 'v' + release.version;
  items.push(el('a', { class: 'native-notes-link', href: `https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/tag/${tag}`, rel: 'noopener noreferrer', text: t('downloads.notes') }));
  return el('div', { class: 'native-notes' }, items);
}

function renderHeading(ready) {
  document.querySelector('#native-dialog-title').textContent = t(`${opened}.title`);
  document.querySelector('[data-native-icon]')?.setAttribute('src', `/assets/icons/${opened}.svg`);
  const meta = document.querySelector('[data-native-meta]');
  if (!meta) return;
  if (!ready) {
    clear(meta).append(el('span', { class: 'badge badge-warn', text: t(PENDING[opened].status) }));
    return;
  }
  clear(meta).append(
    el('span', { class: 'native-version', text: t('downloads.version', { version: current[opened].version }) }),
    el('span', { class: 'badge badge-info', text: t('downloads.beta') }));
}

function renderDialog() {
  const content = document.querySelector('[data-native-content]');
  if (!content || !opened) return;
  const ready = available(current, opened);
  renderHeading(ready);
  clear(content).append(el('p', { class: 'native-requirements', text: t(`${opened}.requirements`) }));
  if (!ready) {
    content.append(el('p', { class: 'native-pending', text: t(PENDING[opened].long) }));
    return;
  }
  const release = current[opened];
  content.append(
    el('div', { class: 'native-options' },
      ORDER[opened].map((kind, index) => option(release.assets.find(asset => asset.kind === kind), index === 0))),
    notes(release));
}

function initialize() {
  if (initialized) return;
  initialized = true;
  const dialog = document.querySelector('[data-native-dialog]');
  for (const button of document.querySelectorAll('[data-open-downloads]')) {
    button.addEventListener('click', () => {
      opened = button.dataset.openDownloads;
      renderDialog();
      dialog.showModal();
    });
  }
  document.querySelector('[data-close-downloads]')?.addEventListener('click', () => dialog.close());
  dialog?.addEventListener('click', event => {
    if (event.target !== dialog) return;
    const rect = dialog.getBoundingClientRect();
    if (event.clientX < rect.left || event.clientX > rect.right || event.clientY < rect.top || event.clientY > rect.bottom) dialog.close();
  });
}

function renderCard(os) {
  const ready = available(current, os);
  const status = document.querySelector(`[data-platform-status="${os}"]`);
  if (status) {
    status.textContent = t(ready ? 'state.available' : PENDING[os].status);
    status.classList.toggle('badge-ok', ready);
    status.classList.toggle('badge-warn', !ready);
  }
  const version = document.querySelector(`[data-platform-version="${os}"]`);
  if (version) version.textContent = ready ? t('downloads.version', { version: current[os].version }) : t(PENDING[os].short);
  document.querySelector(`[data-platform="${os}"]`)?.classList.toggle('is-recommended', choice === os);
  const badge = document.querySelector(`[data-recommended="${os}"]`);
  if (badge) badge.hidden = choice !== os;
}

export function renderNative(config) {
  initialize();
  current = config;
  const note = document.querySelector('[data-os-note]');
  if (note) note.textContent = t(`downloads.detect.${choice ?? 'unknown'}`);
  for (const os of PLATFORMS) renderCard(os);
  renderDialog();
}
