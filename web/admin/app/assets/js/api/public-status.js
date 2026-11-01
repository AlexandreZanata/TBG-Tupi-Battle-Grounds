// Read-only public service status: GET /health and GET /api/version, the two routes the
// backend already exposes to everyone. This is not administrative access.
import { getJson, getText } from './http.js';

let environment = null;

/** The local preview server describes itself in /preview.json (see web/scripts/serve.py). */
export async function previewEnvironment() {
  if (!environment) {
    try {
      environment = await getJson('/preview.json', { maxBytes: 4096 });
    } catch {
      environment = { mode: 'unknown', publicStatus: false };
    }
  }
  return environment;
}

export async function publicStatus() {
  const env = await previewEnvironment();
  if (!env.publicStatus) return { enabled: false };
  const [health, version] = await Promise.allSettled([getText('/health', { maxBytes: 256 }), getJson('/api/version', { maxBytes: 4096 })]);
  const info = version.status === 'fulfilled' ? version.value : null;
  return {
    enabled: true,
    healthy: health.status === 'fulfilled' && health.value.trim() === 'ok',
    protocol: Number.isInteger(info?.protocol) ? info.protocol : null,
    latestClient: typeof info?.latest_client === 'string' ? info.latest_client : null,
  };
}
