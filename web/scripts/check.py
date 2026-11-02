#!/usr/bin/env python3
"""Static checks for the web frontends; no network, no build, no Cargo.

Covers manifests and every local reference (HTML, CSS, JS imports and
fetch paths), JSON validity, JS syntax (node, when installed), the landing
i18n parity rules, the admin string tables, the release availability gates
against the game's protocol and contract, forbidden DOM/script patterns,
the admin public page restrictions and the generated content freshness.
"""

import json
import posixpath
import re
import shutil
import subprocess
import sys
import tempfile
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlsplit

from sites import REPO, SITES, ManifestError, files

import sync_content

ERRORS = []
LANDING_RELEASE_PREFIX = "https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases/download/"
FORBIDDEN_JS = [r"\.innerHTML\b", r"\.outerHTML\b", r"insertAdjacentHTML", r"document\.write", r"\beval\(", r"new Function\("]
MAX_FILE_BYTES = 400 * 1024


def fail(message):
    ERRORS.append(message)


class Refs(HTMLParser):
    """Collect local references, i18n bindings and risky markup from one HTML page."""

    VOID = {"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"}

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.urls, self.texts, self.attrs, self.problems = [], {}, {}, []
        self.stack = []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        for name in ("src", "href"):
            if a.get(name):
                self.urls.append(a[name])
        for part in (a.get("srcset") or "").split(","):
            if part.strip():
                self.urls.append(part.split()[0])
        if tag == "script" and not a.get("src"):
            self.problems.append("inline <script>")
        if "style" in a:
            self.problems.append(f"inline style attribute on <{tag}>")
        for name, value in attrs:
            if name.startswith("on"):
                self.problems.append(f"inline handler {name} on <{tag}>")
            if value and value.strip().lower().startswith("javascript:"):
                self.problems.append(f"javascript: URL on <{tag}>")
        for pair in filter(None, (a.get("data-i18n-attr") or "").split(";")):
            attr, key = pair.split("=")
            self.attrs[key] = a.get(attr.strip(), "")
        if tag not in self.VOID:
            self.stack.append([a.get("data-i18n"), ""])

    def handle_endtag(self, tag):
        if tag in self.VOID or not self.stack:
            return
        key, text = self.stack.pop()
        if self.stack:
            self.stack[-1][1] += text
        if key:
            self.texts[key] = " ".join(text.split())

    def handle_data(self, data):
        if self.stack:
            self.stack[-1][1] += data


def local_target(base, ref):
    """Published path a reference resolves to, or None when it is external or an in-page anchor."""
    parts = urlsplit(ref)
    if parts.scheme or parts.netloc or not parts.path:
        return None
    path = parts.path if parts.path.startswith("/") else posixpath.join(posixpath.dirname("/" + base), parts.path)
    path = posixpath.normpath(path) + ("/" if parts.path.endswith("/") and path != "/" else "")
    rel = path.lstrip("/")
    return rel + "index.html" if rel == "" or rel.endswith("/") else rel


def check_references(site, mapping):
    pages = {}
    for published, source in mapping.items():
        if published.endswith(".html"):
            parser = Refs()
            parser.feed(source.read_text())
            pages[published] = parser
            for problem in parser.problems:
                fail(f"{site.name}/{published}: {problem}")
            for ref in parser.urls:
                if ref.startswith("http://"):
                    fail(f"{site.name}/{published}: insecure link {ref}")
                target = local_target(published, ref)
                if target and target not in mapping:
                    fail(f"{site.name}/{published}: missing {ref}")
        elif published.endswith(".css"):
            for ref in re.findall(r"url\(\s*['\"]?([^'\")]+)", source.read_text()):
                target = local_target(published, ref)
                if target and target not in mapping:
                    fail(f"{site.name}/{published}: missing {ref}")
        elif published.endswith(".js"):
            text = source.read_text()
            for ref in re.findall(r"(?:from|import)\s*['\"](\.{1,2}/[^'\"]+)['\"]", text):
                target = local_target(published, ref)
                if target not in mapping:
                    fail(f"{site.name}/{published}: missing import {ref}")
            for ref in re.findall(r"['\"`](/(?:assets|data|config|media)/[^'\"`$]+)['\"`]", text):
                if local_target(published, ref) not in mapping:
                    fail(f"{site.name}/{published}: missing fetch path {ref}")
            for pattern in FORBIDDEN_JS:
                if re.search(pattern, text):
                    fail(f"{site.name}/{published}: forbidden pattern {pattern}")
            if "localStorage" in text and source != REPO / "web/shared/lang.js":
                fail(f"{site.name}/{published}: localStorage is reserved for the language preference")
        elif published.endswith(".json"):
            try:
                json.loads(source.read_text())
            except ValueError as error:
                fail(f"{site.name}/{published}: invalid JSON ({error})")
        if source.stat().st_size > MAX_FILE_BYTES:
            fail(f"{site.name}/{published}: {source.stat().st_size // 1024} KiB exceeds the {MAX_FILE_BYTES // 1024} KiB file budget")
    return pages


def check_js_syntax(mapping):
    node = shutil.which("node")
    if not node:
        print("note: node not found; JS syntax check skipped")
        return
    with tempfile.TemporaryDirectory() as tmp:
        for published, source in mapping.items():
            if published.endswith(".js"):
                copy = Path(tmp) / "module.mjs"
                copy.write_text(source.read_text())
                result = subprocess.run([node, "--check", str(copy)], capture_output=True, text=True)
                if result.returncode:
                    fail(f"{published}: JS syntax error\n{result.stderr.strip()}")


def check_landing_i18n(mapping, pages):
    dicts = {}
    for lang in ("pt-BR", "en"):
        dicts[lang] = json.loads(mapping[f"assets/i18n/{lang}.json"].read_text())
    if set(dicts["pt-BR"]) != set(dicts["en"]):
        fail(f"landing i18n key sets differ: {sorted(set(dicts['pt-BR']) ^ set(dicts['en']))}")
    for page, parser in pages.items():
        for key, text in {**parser.texts, **parser.attrs}.items():
            if key not in dicts["pt-BR"]:
                fail(f"landing/{page}: i18n key {key} missing from pt-BR.json")
            elif " ".join(dicts["pt-BR"][key].split()) != text:
                fail(f"landing/{page}: HTML text for {key} differs from pt-BR.json")
    used = set()
    for published, source in mapping.items():
        if published.startswith("assets/js/"):
            used |= set(re.findall(r"\bt\(['`]([\w.]+)['`]", source.read_text()))
    for key in sorted(used - set(dicts["pt-BR"])):
        fail(f"landing JS uses unknown i18n key {key}")


def check_admin_strings(mapping):
    text = mapping["assets/js/strings.js"].read_text()
    blocks = re.split(r"^\s{2}en: \{$", text, flags=re.M)
    pt_keys = set(re.findall(r"^\s{4}'([\w.]+)':", blocks[0], flags=re.M))
    en_keys = set(re.findall(r"^\s{4}'([\w.]+)':", blocks[1], flags=re.M))
    if pt_keys != en_keys:
        fail(f"admin strings differ between pt-BR and en: {sorted(pt_keys ^ en_keys)}")
    used = set()
    for published, source in mapping.items():
        if published.startswith("assets/js/") and published != "assets/js/strings.js":
            used |= set(re.findall(r"\bt\('([\w.]+)'", source.read_text()))
    for key in sorted(used - pt_keys):
        fail(f"admin JS uses unknown string {key}")


def current_contract():
    core = REPO / "crates/tbg-core"
    protocol = int(re.search(r"PROTOCOL_VERSION: u16 = (\d+);", (core / "src/protocol.rs").read_text()).group(1))
    revision = int(re.search(r"RULESET_VERSION: u32 = (\d+);", (core / "src/ruleset_version.rs").read_text()).group(1))
    registry = (core / "tests/fixtures/ruleset-versions.txt").read_text().splitlines()
    hashes = dict(line.split() for line in registry if line.strip() and not line.startswith("#"))
    return protocol, f"w{protocol}-r{revision}-{hashes[str(revision)]}"


def check_release(mapping):
    config = json.loads(mapping["config/release.json"].read_text())
    protocol, contract = current_contract()
    server = config.get("server", {})
    if server.get("protocol") != protocol or server.get("contract") != contract:
        fail(f"release.json server is {server}, but the game is protocol {protocol}, contract {contract}")
    web = config.get("webGame", {})
    if web.get("state") not in ("pending", "live") or not str(web.get("url", "")).startswith("https://app.tupibg.com/"):
        fail("release.json webGame needs state pending|live and an https://app.tupibg.com/ URL")
    for os, kinds in {"linux": {"appimage", "tarball"}, "windows": {"installer", "zip"}}.items():
        native = config.get(os, {})
        if native.get("state") != "pending":
            fail(f"release.json {os} must stay pending; only the verified live catalog may enable native downloads")
    fallback = json.loads(mapping["downloads.json"].read_text())
    if fallback != {"schema": 1, "server": server, "linux": {"state": "pending"}, "windows": {"state": "pending"}}:
        fail("downloads.json fallback must match the authored server contract and keep both native downloads pending")
    return config


def check_admin_public(mapping):
    for published, source in mapping.items():
        if published.endswith((".js", ".json")):
            fail(f"admin-public must not publish scripts or data: {published}")
        if published.endswith(".html"):
            text = source.read_text().lower()
            for marker in ("<script", "<form", "<input", "fetch("):
                if marker in text:
                    fail(f"admin-public/{published} contains {marker}")


def main():
    for site in SITES.values():
        if not site.root.is_dir():
            print(f"note: {site.name} is not present; skipped")
            continue
        try:
            mapping = files(site)
        except ManifestError as error:
            fail(f"{site.name}: {error}")
            continue
        pages = check_references(site, mapping)
        check_js_syntax(mapping)
        if site.name == "landing":
            check_landing_i18n(mapping, pages)
            check_release(mapping)
        elif site.name in {"admin-preview", "admin-app"}:
            check_admin_strings(mapping)
            api = mapping["assets/js/api/admin.js"].read_text()
            if "X-TBG-CSRF" not in api or "credentials:'same-origin'" not in api or "method!=='GET'" not in api:
                fail(f"{site.name}: console requests require same-origin credentials and mutation CSRF")
        elif site.name == "admin-public":
            check_admin_public(mapping)
        total = sum(source.stat().st_size for source in mapping.values())
        print(f"{site.name}: {len(mapping)} files, {total / 1024:.0f} KiB checked")
    if sync_content.main_check() != 0:
        fail("generated landing content is stale (run web/scripts/sync_content.py)")
    for message in ERRORS:
        print("FAIL", message)
    print("web checks:", "FAILED" if ERRORS else "passed")
    return 1 if ERRORS else 0


if __name__ == "__main__":
    sys.exit(main())
