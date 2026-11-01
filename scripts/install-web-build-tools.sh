#!/usr/bin/env bash
# Official fixed releases; execute tools only after verifying committed digests.
set -euo pipefail
root="${RUNNER_TEMP:?}/tbg-web-tools"
mkdir -p "$root"
python3 - <<'PY'
import tomllib
with open('Cargo.lock', 'rb') as stream:
    packages = tomllib.load(stream)['package']
assert next(p['version'] for p in packages if p['name'] == 'wasm-bindgen') == '0.2.129', 'Update the pinned wasm-bindgen tool with Cargo.lock'
PY
while read -r name digest url binary; do
	mkdir -p "$root/$name"
	curl --fail --location --retry 3 --connect-timeout 10 --max-time 180 -o "$root/$name.tar.gz" "$url"
	printf '%s  %s\n' "$digest" "$root/$name.tar.gz" | sha256sum --check --strict
	tar -xzf "$root/$name.tar.gz" -C "$root/$name"
	tool="$(find "$root/$name" -type f -name "$binary" -print -quit)"
	test -n "$tool"
	dirname "$tool" >> "${GITHUB_PATH:?}"
	"$tool" --version
 done <<'TOOLS'
trunk 2ab8d548fd5ed9cda50906f61fd8654204e7893bd52ebbfd90533862b7d71ee1 https://github.com/trunk-rs/trunk/releases/download/v0.22.1/trunk-x86_64-unknown-linux-gnu.tar.gz trunk
bindgen 82d12bb940e2d4e72e0d5605387fc1b8ca179044e012b620f0ce4e7440e8320e https://github.com/wasm-bindgen/wasm-bindgen/releases/download/0.2.129/wasm-bindgen-0.2.129-x86_64-unknown-linux-musl.tar.gz wasm-bindgen
binaryen b5bf1f0eaf17c63ee588ff7a5954dc8f6ce2c26989051c66f24dfe9ece3e46db https://github.com/WebAssembly/binaryen/releases/download/version_131/binaryen-version_131-x86_64-linux.tar.gz wasm-opt
TOOLS
