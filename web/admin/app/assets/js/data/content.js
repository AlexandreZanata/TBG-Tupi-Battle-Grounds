// Game content snapshots from the repository (read-only): the same derived catalogs the landing uses.
import { getJson } from '../api/http.js';

const cache = new Map();

function load(path, maxBytes) {
  if (!cache.has(path)) {
    const pending = getJson(path, { maxBytes }).catch(error => {
      cache.delete(path);
      throw error;
    });
    cache.set(path, pending);
  }
  return cache.get(path);
}

export const champions = () => load('/data/champions.json', 128 * 1024);
export const arenas = () => load('/data/arenas.json', 64 * 1024);
export const patchNotes = () => load('/data/patch-notes.json', 256 * 1024);
