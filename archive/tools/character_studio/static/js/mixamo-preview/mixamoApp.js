/** Mixamo import preview — baked GLB/FBX clips via AnimationMixer (no procedural walk). */

import { mountAnimViewer } from "../assembly-studio/animViewer.js";

const MANIFEST_URL = "/api/content/characters/imports/mixamo/manifest.json";
let viewer = null;
let selectedId = null;
let manifest = { characters: [] };

function setStatus(msg) {
  const el = document.getElementById("mixamoStatus");
  if (el) el.textContent = msg;
}

function renderClipSelect(names) {
  const sel = document.getElementById("clipSelect");
  if (!sel) return;
  sel.innerHTML = "";
  if (!names.length) {
    const opt = document.createElement("option");
    opt.value = "";
    opt.textContent = "No clips — re-import with --animations";
    sel.appendChild(opt);
    sel.disabled = true;
    return;
  }
  sel.disabled = false;
  for (const name of names) {
    const opt = document.createElement("option");
    opt.value = name;
    opt.textContent = name;
    sel.appendChild(opt);
  }
}

function renderList(characters) {
  const list = document.getElementById("charList");
  if (!list) return;
  list.innerHTML = "";
  for (const ch of characters) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "manual-list__item";
    if (ch.id === selectedId) btn.classList.add("manual-list__item--active");
    btn.textContent = ch.label;
    btn.addEventListener("click", () => void selectCharacter(ch));
    list.appendChild(btn);
  }
}

async function selectCharacter(ch) {
  selectedId = ch.id;
  const fbx = document.getElementById("fbxPath");
  if (fbx) fbx.textContent = `content/characters/imports/mixamo/${ch.sourceFbx}`;
  renderList(manifest.characters);
  setStatus(`Loading ${ch.label}…`);
  try {
    const url = ch.meshUrl || ch.fbxUrl;
    const bust = `${url}${url.includes("?") ? "&" : "?"}t=${Date.now()}`;
    const info = await viewer.loadAsset(bust);
    const names = info?.clipNames ?? [];
    renderClipSelect(names);
    const pick = ch.defaultClip && names.includes(ch.defaultClip) ? ch.defaultClip : names[0];
    if (pick) viewer.playClip(pick, true);
    setStatus(names.length ? `${ch.label} · ${names.length} clip(s)` : `${ch.label} · static mesh`);
  } catch (err) {
    console.error("Mixamo preview failed", err);
    setStatus(err.message || "Load failed — run ./scripts/import-mixamo-studio.sh");
  }
}

function bindControls() {
  document.getElementById("btnPlayClip")?.addEventListener("click", () => {
    const name = document.getElementById("clipSelect")?.value;
    if (name) viewer.playClip(name, true);
  });
  document.getElementById("btnStopClip")?.addEventListener("click", () => viewer.stopClip());
  document.getElementById("clipSelect")?.addEventListener("change", (ev) => {
    const name = ev.target.value;
    if (name) viewer.playClip(name, true);
  });
}

async function init() {
  viewer = mountAnimViewer(document.getElementById("viewport"));
  bindControls();
  const res = await fetch(MANIFEST_URL);
  if (!res.ok) {
    setStatus("Run ./scripts/import-mixamo-studio.sh");
    return;
  }
  manifest = await res.json();
  if (!manifest.characters?.length) {
    setStatus("No characters in manifest");
    return;
  }
  renderList(manifest.characters);
  const defeated = manifest.characters.find((c) => c.id === "defeated");
  await selectCharacter(defeated ?? manifest.characters[0]);
}

document.addEventListener("DOMContentLoaded", () => {
  void init();
});
