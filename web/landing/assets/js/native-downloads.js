// Render platform cards and accessible native-download dialogs from verified metadata.
import { el, clear } from '../shared/dom.js';
import { t } from './i18n.js';
import { available, detectedOS } from './download-rules.js';

let current = null;
let opened = null;
let initialized = false;
const choice = detectedOS(navigator.userAgentData?.platform ?? navigator.platform, navigator.userAgent);
const labels = { installer: 'downloads.install', zip: 'downloads.portable', appimage: 'downloads.appimage', tarball: 'downloads.tarball' };
const descriptions = { installer: 'downloads.windows.install', zip: 'downloads.windows.portable', appimage: 'downloads.linux.appimage', tarball: 'downloads.linux.tarball' };

function option(asset) {
  const details = el('details', { class: 'native-checksum' },
    el('summary', { text: t('downloads.checksum') }), el('code', { text: asset.sha256 }));
  return el('article', { class: 'native-option' },
    el('h3', { text: t(labels[asset.kind]) }), el('p', { text: t(descriptions[asset.kind]) }),
    el('a', { class: 'btn btn-gold', href: asset.url, rel: 'noopener noreferrer' },
      el('span', { text: t(labels[asset.kind]) }), el('small', { text: t('linux.size', { size: (asset.bytes / 1048576).toFixed(1) }) })), details);
}

function renderDialog() {
  const dialog = document.querySelector('[data-native-dialog]');
  const content = document.querySelector('[data-native-content]');
  if (!dialog || !content || !opened) return;
  document.querySelector('#native-dialog-title').textContent = t(`${opened}.title`);
  clear(content);
  const icon = el('img', { class: 'platform-icon', src: `/assets/icons/${opened}.svg`, width: 56, height: 56, alt: '' });
  content.append(icon);
  if (!available(current, opened)) {
    content.append(el('p', { class: 'state-note', text: t('downloads.pending') }));
    return;
  }
  const release = current[opened];
  content.append(el('div', { class: 'native-release-meta' },
    el('strong', { text: t('downloads.version', { version: release.version }) }),
    el('span', { class: 'badge badge-info', text: t('downloads.beta') })));
  content.append(el('p', { class: 'fine-print', text: t(`${opened}.requirements`) }));
  const order = opened === 'windows' ? ['installer', 'zip'] : ['appimage', 'tarball'];
  content.append(el('div', { class: 'native-options' }, order.map(kind => option(release.assets.find(asset => asset.kind === kind)))));
  if (opened === 'windows') content.append(el('p', { class: 'fine-print', text: t('downloads.unsigned') }));
  if (release.visual_acceptance !== 'passed') content.append(el('p', { class: 'fine-print', text: t('downloads.pixels.pending') }));
  content.append(el('a', { href: `https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/tag/v${release.version}`, rel: 'noopener noreferrer', text: t('downloads.notes') }));
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

export function renderNative(config) {
  initialize();
  current = config;
  const note = document.querySelector('[data-os-note]');
  if (note) note.textContent = t(`downloads.detect.${choice ?? 'unknown'}`);
  for (const os of ['windows', 'linux']) {
    const ready = available(config, os);
    const status = document.querySelector(`[data-platform-status="${os}"]`);
    if (status) {
      status.textContent = t(ready ? 'state.available' : 'state.preparing');
      status.classList.toggle('badge-ok', ready);
      status.classList.toggle('badge-warn', !ready);
    }
    const version = document.querySelector(`[data-platform-version="${os}"]`);
    if (version) version.textContent = ready ? t('downloads.version', { version: config[os].version }) : t('downloads.pending');
    document.querySelector(`[data-platform="${os}"]`)?.classList.toggle('is-recommended', choice === os);
    const badge = document.querySelector(`[data-recommended="${os}"]`);
    if (badge) badge.hidden = choice !== os;
  }
  renderDialog();
}
