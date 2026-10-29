// Pure BR-AR02 preview validation, shared with the client contract tests.
(() => {
  'use strict';
  const editable = (ranks, data) => Array.isArray(ranks) && ranks.length === data.nodes.length
    && ranks.every(rank => Number.isInteger(rank) && rank >= 0 && rank <= data.maxRank)
    && ranks.reduce((sum, rank) => sum + rank, 0) <= data.pointBudget;
  const spent = ranks => ranks.reduce((sum, rank) => sum + rank, 0);
  const complete = (ranks, data) => editable(ranks, data) && spent(ranks) === data.pointBudget;
  globalThis.TbgArcanaPreset = Object.freeze({editable, spent, complete});
})();
