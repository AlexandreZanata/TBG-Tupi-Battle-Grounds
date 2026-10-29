// Pure Arcana preset rules, shared by the editor and its Node contract tests.
(() => {
  'use strict';
  const integer = value => Number.isInteger(value) && value >= 0;
  const spent = (ranks, data) => ranks.reduce((sum, rank, i) => sum + rank * data.nodes[i].cost, 0);
  const maxRank = (index, data) => data.nodes[index].maxRank;
  const prerequisites = (ranks, data) => data.nodes.every(node => ranks[node.index] === 0
    || node.requires.every(([index, rank]) => ranks[index] >= rank));
  const editable = (ranks, data) => Array.isArray(ranks) && ranks.length === data.nodes.length
    && ranks.every((rank, i) => integer(rank) && rank <= maxRank(i, data))
    && prerequisites(ranks, data) && spent(ranks, data) <= data.pointBudget;
  const complete = (ranks, data) => editable(ranks, data) && spent(ranks, data) === data.pointBudget;
  const missing = (ranks, data, node) => node.requires
    .filter(([index, rank]) => ranks[index] < rank)
    .map(([index, rank]) => data.nodes[index].name + ' rank ' + rank + ' (now ' + ranks[index] + ')');
  const canAdd = (ranks, data, index) => {
    const node = data.nodes[index];
    return ranks[index] < node.maxRank
      && spent(ranks, data) + node.cost <= data.pointBudget
      && node.requires.every(([i, rank]) => ranks[i] >= rank);
  };
  const dependents = (ranks, data, index, nextRank) => data.nodes
    .filter(node => ranks[node.index] > 0
      && node.requires.some(([i, rank]) => i === index && nextRank < rank))
    .map(node => node.name);
  const canRemove = (ranks, data, index) => ranks[index] > 0
    && dependents(ranks, data, index, ranks[index] - 1).length === 0;
  const migrate = (old, data) => {
    const ranks = data.nodes.map(() => 0);
    for (let i = 0; i < Math.min(old.length, data.nodes.length); i++) {
      ranks[i] = Math.min(integer(old[i]) ? old[i] : 0, data.nodes[i].maxRank);
    }
    let total = spent(ranks, data);
    for (const node of data.nodes.filter(n => n.kind === 'original')) {
      while (total < data.pointBudget && ranks[node.index] < node.maxRank) {
        ranks[node.index] += 1; total += node.cost;
      }
    }
    return ranks;
  };
  globalThis.TbgArcanaPreset = Object.freeze({
    editable, spent, complete, canAdd, canRemove, dependents, missing, migrate
  });
})();
