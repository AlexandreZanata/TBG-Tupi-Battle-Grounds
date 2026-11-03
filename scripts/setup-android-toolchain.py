#!/usr/bin/env python3
"""Install and verify the declared Android SDK pins without depending on runner PATH."""
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def install(sdk, pins, output_env):
    manager = sdk/'cmdline-tools/latest/bin/sdkmanager'
    if not manager.is_file():
        raise ValueError('Android SDK command-line tools missing: '+str(manager))
    subprocess.run([str(manager), f"platforms;android-{pins['compile_sdk']}",
                    'build-tools;'+pins['build_tools'], 'ndk;'+pins['ndk']], check=True)
    versions = {sdk/f"platforms/android-{pins['compile_sdk']}": str(pins['sdk_platform_revision']),
                sdk/'build-tools'/pins['build_tools']: pins['build_tools'],
                sdk/'ndk'/pins['ndk']: pins['ndk']}
    for folder, expected in versions.items():
        raw = (folder/'source.properties').read_text()
        revision = re.search(r'(?m)^Pkg\.Revision\s*=\s*(\S+)\s*$', raw)
        if revision is None or revision[1] != expected:
            raise ValueError('Installed Android package differs from its reviewed pin: '+str(folder))
    ndk = sdk/'ndk'/pins['ndk']
    values = {'ANDROID_HOME': sdk, 'ANDROID_SDK_ROOT': sdk,
              'ANDROID_NDK_HOME': ndk, 'ANDROID_NDK_ROOT': ndk, 'ANDROID_NDK': ndk}
    with output_env.open('a') as stream:
        stream.writelines(f'{key}={value}\n' for key, value in values.items())
    print('Verified Android SDK/platform/build-tools/NDK pins:', sdk)


def main():
    sdk = os.environ.get('ANDROID_HOME') or os.environ.get('ANDROID_SDK_ROOT')
    if not sdk or not os.environ.get('GITHUB_ENV'):
        raise SystemExit('Set the SDK root and GITHUB_ENV before CI toolchain provisioning')
    pins = json.loads((ROOT/'platform/android/toolchain.json').read_text())
    install(Path(sdk).resolve(), pins, Path(os.environ['GITHUB_ENV']))


if __name__ == '__main__':
    main()
