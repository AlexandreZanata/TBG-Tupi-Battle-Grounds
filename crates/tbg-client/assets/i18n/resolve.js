// P41 language resolver for pages whose CSP forbids inline scripts (the
// standalone ranking pages): ?lang=, then tbg.lang, then the browser, else
// en. Load it synchronously before i18n.js. The game inlines the same rule
// in index.html so its first paint never waits on a request.
(function () {
  'use strict';
  var root = document.documentElement;
  try {
    var asked = new URLSearchParams(location.search).get('lang');
    var code = null;
    if (asked) code = /^pt/i.test(asked) ? 'pt-BR' : (/^en/i.test(asked) ? 'en' : null);
    if (!code) { try { code = localStorage.getItem('tbg.lang'); } catch (e) {} }
    if (code !== 'pt-BR' && code !== 'en') {
      var list = (navigator.languages && navigator.languages.length)
        ? navigator.languages : (navigator.language ? [navigator.language] : []);
      code = 'en';
      for (var i = 0; i < list.length; i++) {
        if (/^pt/i.test(list[i])) { code = 'pt-BR'; break; }
        if (/^en/i.test(list[i])) { code = 'en'; break; }
      }
    }
    root.lang = code;
  } catch (e) { root.lang = 'en'; }
  root.dataset.i18nBase = '/assets/i18n/';
})();
