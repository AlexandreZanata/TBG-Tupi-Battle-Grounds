"""Android beta identity and the inputs that require a new immutable APK revision."""
import json
import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'AlexandreZanata/TBG-Tupi-Battle-Grounds'
WORKFLOW = '.github/workflows/android-release.yml'
CERTIFICATE = '5b0762f8270e29e73466305224b3315ffc69bea77cc8c828b7a5b1999696f168'


def configuration(root=ROOT):
    value = json.loads((root/'platform/android/release.json').read_text())
    if not (value.get('schema') == 1 and value.get('channel') == 'beta'
            and type(value.get('version_code')) is int and 54 < value['version_code'] < 2100000000
            and value.get('package') == 'io.github.alexandrezanata.tbg.prototype'
            and value.get('abi') == 'arm64-v8a' and value.get('certificate_sha256') == CERTIFICATE):
        raise ValueError('Invalid Android beta/signing/upgrade identity')
    return value


def version(root=ROOT):
    client = tomllib.loads((root/'crates/tbg-client/Cargo.toml').read_text())['package']['version']
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?', client):
        raise ValueError('Invalid shared client version')
    return client + '-android.' + str(configuration(root)['version_code'])


def filename(value):
    return f'tupi-battle-grounds-{value}-android-arm64-v8a.apk'


def relevant(paths):
    files = {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml',
             'scripts/package-android.py', 'scripts/android_binary.py', 'scripts/android_release.py',
             'scripts/build-android-beta.py', 'scripts/setup-android-toolchain.py'}
    prefixes = ('crates/tbg-client/', 'crates/tbg-core/', 'platform/android/')
    return any(path in files or (path.startswith(prefixes) and not path.endswith('.md')
               and not any(part in path for part in ['/tests/', '/examples/', '/benches/'])) for path in paths)
