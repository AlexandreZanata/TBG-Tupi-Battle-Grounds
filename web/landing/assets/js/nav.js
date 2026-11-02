// Header behavior: the compact-screen section menu, the scrolled header state and section reveal.
// Without this module the page still works: the menu falls back to a scrollable row and nothing is hidden.

const header = document.querySelector('.site-header');
const toggle = document.querySelector('[data-nav-toggle]');
const nav = document.querySelector('#site-nav');
const compact = window.matchMedia('(max-width: 960px)');

function setOpen(open) {
  header.classList.toggle('nav-open', open);
  toggle.setAttribute('aria-expanded', String(open));
}

function initMenu() {
  if (!header || !toggle || !nav) return;
  header.classList.add('nav-ready');
  toggle.hidden = false;
  toggle.addEventListener('click', () => setOpen(!header.classList.contains('nav-open')));
  nav.addEventListener('click', event => {
    if (event.target.closest('a')) setOpen(false);
  });
  document.addEventListener('keydown', event => {
    if (event.key !== 'Escape' || !header.classList.contains('nav-open')) return;
    setOpen(false);
    toggle.focus();
  });
  document.addEventListener('click', event => {
    if (header.classList.contains('nav-open') && !header.contains(event.target)) setOpen(false);
  });
  compact.addEventListener('change', () => setOpen(false));
}

function initScrolled() {
  if (!header) return;
  const update = () => header.classList.toggle('is-scrolled', window.scrollY > 8);
  update();
  window.addEventListener('scroll', update, { passive: true });
}

// Content is hidden only once the observer exists, so a failure never leaves sections invisible.
function initReveal() {
  if (!('IntersectionObserver' in window) || window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
  const targets = document.querySelectorAll('.section-head, .pillar, .steps li, .shot-grid li, .platform');
  const observer = new IntersectionObserver(entries => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      entry.target.classList.add('is-visible');
      observer.unobserve(entry.target);
    }
  }, { rootMargin: '0px 0px -8% 0px' });
  for (const target of targets) {
    target.classList.add('reveal');
    observer.observe(target);
  }
}

initMenu();
initScrolled();
initReveal();
