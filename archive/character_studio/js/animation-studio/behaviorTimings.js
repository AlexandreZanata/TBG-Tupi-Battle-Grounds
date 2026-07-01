/** Read-only behavior timing panel for Animation Studio. */

function formatMs(value) {
  if (value == null || Number.isNaN(Number(value))) return "—";
  return `${value} ms`;
}

export function renderBehaviorTimings(container, profile) {
  if (!container) return;
  if (!profile?.timings) {
    container.hidden = true;
    container.innerHTML = "";
    return;
  }

  const timings = profile.timings;
  const combat = profile.combat ?? {};
  container.hidden = false;
  container.innerHTML = `
    <header class="behavior-timings__head">
      <h3>Behavior timings</h3>
      <span class="behavior-timings__id">${profile.id ?? "—"}</span>
    </header>
    <dl class="behavior-timings__grid">
      <dt>Attack windup</dt><dd>${formatMs(timings.attackWindupMs)}</dd>
      <dt>Attack hit</dt><dd>${formatMs(timings.attackHitMs)}</dd>
      <dt>Ultimate windup</dt><dd>${formatMs(timings.ultimateWindupMs)}</dd>
      <dt>Ultimate duration</dt><dd>${formatMs(timings.ultimateDurationMs)}</dd>
      <dt>Attack cycle</dt><dd>${combat.attackCycle ?? "repeat"}</dd>
    </dl>
    <p class="behavior-timings__note">Read-only — edit JSON in <code>content/characters/behaviors/</code></p>
  `;
}

export async function loadBehaviorTimings(container, behaviorProfileId) {
  if (!behaviorProfileId) {
    renderBehaviorTimings(container, null);
    return;
  }
  const url = `/api/content/characters/behaviors/${behaviorProfileId}.json`;
  const response = await fetch(url);
  if (!response.ok) {
    renderBehaviorTimings(container, null);
    return;
  }
  renderBehaviorTimings(container, await response.json());
}
