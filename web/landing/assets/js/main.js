// Landing entry point: language first, then the data-driven sections, re-rendered on language change.
import { initI18n, onLangChange } from './i18n.js';
import { loadRelease, renderRelease } from './release.js';
import { loadChampions, renderChampions } from './champions.js';
import { loadArenas, renderArenas } from './arenas.js';
import { loadNews, renderNews } from './news.js';
import { initGallery } from './gallery.js';

await initI18n();
onLangChange(() => {
  renderRelease();
  renderChampions();
  renderArenas();
  renderNews();
});
initGallery();
await Promise.all([loadRelease(), loadChampions(), loadArenas(), loadNews()]);
