// Admin preview entry point: build the frame, route, and rebuild both when the language changes.
import { onLangChange } from './i18n.js';
import { renderShell } from './shell.js';
import { setFrame, showRoute, startRouter } from './router.js';

const root = document.getElementById('app');
startRouter(renderShell(root));
onLangChange(() => {
  setFrame(renderShell(root));
  showRoute({ focus: false });
});
showRoute({ focus: false });
