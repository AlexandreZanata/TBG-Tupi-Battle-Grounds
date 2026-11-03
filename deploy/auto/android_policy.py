"""Validate an attested Android beta against its signing identity and the live match contract."""
import re
from policy import CONTRACT, REPOSITORY, SHA, require

WORKFLOW = REPOSITORY + '/.github/workflows/android-release.yml'
CERTIFICATE = '5b0762f8270e29e73466305224b3315ffc69bea77cc8c828b7a5b1999696f168'
PACKAGE = 'io.github.alexandrezanata.tbg.prototype'


def catalog(manifest, release, live):
    require(isinstance(manifest, dict) and isinstance(release, dict), 'Android metadata must be objects')
    value = manifest.get('version', '')
    code = manifest.get('version_code')
    require(re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?-android\.[0-9]+', value), 'invalid Android version')
    require(type(code) is int and 54 < code < 2100000000 and value.endswith('-android.'+str(code)), 'Android upgrade revision mismatch')
    require(manifest.get('schema') == 1 and SHA.fullmatch(manifest.get('source_commit', '')), 'invalid Android source')
    require(manifest.get('tag') == release.get('tag_name') == 'android-v'+value
            and not release.get('draft') and release.get('prerelease') is True, 'Android beta tag/publication mismatch')
    require(manifest.get('channel') == 'beta' and manifest.get('debuggable') is False
            and manifest.get('features') == [], 'internal/debug Android builds cannot be public downloads')
    require(manifest.get('certificate_sha256') == CERTIFICATE and manifest.get('package') == PACKAGE
            and manifest.get('abi') == 'arm64-v8a' and manifest.get('min_sdk') == 31
            and manifest.get('target_sdk') == 36, 'unreviewed Android signing/package/platform identity')
    contract = manifest.get('contract', '')
    require(CONTRACT.fullmatch(contract), 'invalid Android contract')
    protocol = int(re.match(r'w([0-9]+)-', contract)[1])
    require(manifest.get('protocol') == protocol and contract == live['contract'], 'Android/server contract differs')
    assets = manifest.get('assets', [])
    require(isinstance(assets, list) and len(assets) == 1, 'one ARM64 APK is required')
    asset = assets[0]
    name = f'tupi-battle-grounds-{value}-android-arm64-v8a.apk'
    prefix = f'https://github.com/{REPOSITORY}/releases/download/{release["tag_name"]}/'
    require(asset.get('kind') == 'apk' and asset.get('name') == name and asset.get('url') == prefix+name, 'unexpected Android APK URL/ABI')
    require(re.fullmatch(r'[0-9a-f]{64}', asset.get('sha256', ''))
            and type(asset.get('bytes')) is int and asset['bytes'] > 0, 'invalid Android digest/size')
    published = next((a for a in release.get('assets', []) if a.get('name') == name), None)
    require(published is not None and published.get('browser_download_url') == prefix+name
            and published.get('digest') == 'sha256:'+asset['sha256'] and published.get('size') == asset['bytes'], 'published Android APK differs from the attested manifest')
    return {'schema': 1, 'server': {'protocol': protocol, 'contract': contract}, 'android': {
        'state': 'available', 'version': value, 'version_code': code, 'tag': manifest['tag'],
        'protocol': protocol, 'contract': contract, 'channel': 'beta', 'abi': 'arm64-v8a',
        'package': PACKAGE, 'certificate_sha256': CERTIFICATE, 'debuggable': False,
        'min_sdk': 31, 'target_sdk': 36, 'assets': assets,
        'visual_acceptance': manifest.get('visual_acceptance', 'pending')}}
