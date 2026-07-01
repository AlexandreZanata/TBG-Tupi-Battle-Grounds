/** Instant client-side preview — stature scale (morph targets use mblabMorphDriver). */

import { applyStatureScale } from "./mblabMorphDriver.js";

export function applyQuickMorph(sceneRoot, morphs) {
  applyStatureScale(sceneRoot, morphs?.stature);
}
