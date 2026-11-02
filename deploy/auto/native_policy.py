"""Validate a complete native release before generating public OS-specific download metadata."""
import re
from policy import CONTRACT, REPOSITORY, SHA, require

WORKFLOW = REPOSITORY + '/.github/workflows/native-release.yml'
VERSION = re.compile(r'([0-9]+)\.([0-9]+)\.([0-9]+)(?:-([0-9A-Za-z.-]+))?')
KINDS = {'linux': {'appimage', 'tarball'}, 'windows': {'installer', 'zip'}}


def version_key(value):
    require(isinstance(value, str) and len(value) <= 64, 'invalid native version')
    match = VERSION.fullmatch(value)
    require(match is not None and len(value) <= 64, 'invalid native version')
    core = tuple(int(part) for part in match.groups()[:3])
    pre = match[4]
    return core + ((1,) if pre is None else (0, tuple((0, int(p)) if p.isdigit() else (1, p) for p in pre.split('.'))))


def catalog(manifest, release, live):
    require(isinstance(manifest, dict) and isinstance(release, dict) and isinstance(live, dict), 'native metadata must be objects')
    version = manifest.get('version', '')
    version_key(version)
    require(manifest.get('schema') == 1 and SHA.fullmatch(manifest.get('source_commit', '')), 'invalid native identity')
    require(manifest.get('tag') == release['tag_name'] == 'v' + version, 'native tag/version mismatch')
    require(CONTRACT.fullmatch(manifest.get('contract', '')) is not None, 'invalid native contract')
    protocol = int(re.match(r'w([0-9]+)-', manifest['contract'])[1])
    require(manifest.get('protocol') == protocol and manifest['contract'] == live['contract'], 'native/server contract differs')
    require(manifest.get('channel') == 'beta' and manifest.get('windows_publisher_signature') == 'unsigned', 'unreviewed signing/channel policy')
    require(set(manifest.get('platforms', {})) == set(KINDS), 'complete Linux and Windows publication required')
    result = {'schema': 1, 'server': {'protocol': protocol, 'contract': live['contract']}}
    published = {asset['name']: asset for asset in release['assets']}
    prefix = f'https://github.com/{REPOSITORY}/releases/download/{release["tag_name"]}/'
    for os, expected in KINDS.items():
        assets = manifest['platforms'][os].get('assets', [])
        require(len(assets) == len(expected) and {a.get('kind') for a in assets} == expected, 'incomplete or duplicate native formats')
        for asset in assets:
            name = asset.get('name', '')
            suffixes = {'appimage': '.AppImage', 'tarball': '.tar.gz', 'installer': '-setup.exe', 'zip': '.zip'}
            expected_name = f'tupi-battle-grounds-{version}-{os}-x86_64' + suffixes[asset['kind']]
            require(name == expected_name, 'native filename/platform mismatch')
            require(re.fullmatch(r'[A-Za-z0-9._-]{1,180}', name) and asset.get('url') == prefix + name, 'unexpected native URL')
            require(re.fullmatch(r'[0-9a-f]{64}', asset.get('sha256', '')) and type(asset.get('bytes')) is int and asset['bytes'] > 0, 'invalid native digest/size')
            actual = published.get(name)
            require(actual is not None and actual.get('browser_download_url') == prefix + name, 'native asset not published')
            require(actual.get('size') == asset['bytes'] and actual.get('digest') == 'sha256:' + asset['sha256'], 'published native asset differs from signed manifest')
        result[os] = {'state': 'available', 'version': version, 'protocol': protocol,
                      'contract': live['contract'], 'assets': assets,
                      'visual_acceptance': manifest.get('visual_acceptance', 'pending')}
    return result
