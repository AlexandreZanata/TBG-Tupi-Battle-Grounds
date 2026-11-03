#!/usr/bin/env python3
"""Build and inspect a public Android beta; local callers must hold the shared host lock."""
import argparse
import os
from pathlib import Path
import subprocess

from android_release import ROOT, configuration, filename, version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--abi', choices=['arm64-v8a', 'x86_64'], default='arm64-v8a')
    parser.add_argument('--output', type=Path, default=ROOT/'target/android-release')
    args = parser.parse_args()
    os.chdir(ROOT)
    if subprocess.check_output(['git', 'status', '--porcelain'], text=True).strip():
        raise SystemExit('Commit the reviewed source before building the public APK')
    source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    triple = {'arm64-v8a': 'aarch64-linux-android', 'x86_64': 'x86_64-linux-android'}[args.abi]
    env = dict(os.environ, TBG_SOURCE_COMMIT=source, TBG_DEFAULT_SERVER='wss://api.tupibg.com/ws',
               CARGO_BUILD_JOBS='2')
    env['CARGO_TARGET_'+triple.upper().replace('-', '_')+'_RUSTFLAGS'] = '-C link-arg=-Wl,-z,max-page-size=16384'
    target = Path(env.get('CARGO_TARGET_DIR', ROOT/'target'))
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    scratch = ROOT/'.local/android-beta-build'/args.abi
    scratch.mkdir(parents=True, exist_ok=True)

    def run(command, cwd=ROOT):
        subprocess.run([str(x) for x in command], cwd=cwd, env=env, check=True)

    run(['cargo', 'ndk', '-t', args.abi, '-P', '31', 'build', '--locked', '--release', '-p', 'tbg-client', '--lib'])
    library = scratch/'libtbg_client.so'
    run([Path(env['ANDROID_NDK_HOME'])/'toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip',
         '--strip-unneeded', '-o', library, target/triple/'release/libtbg_client.so'])
    run(['python3', 'scripts/package-android.py', 'prepare', '--source', source, '--library', library,
         '--abi', args.abi, '--version-code', str(configuration()['version_code']), '--channel', 'beta'])
    run(['./gradlew', ':app:assembleRelease', '--no-parallel', '--max-workers=2',
         '-PprototypeVersionCode='+str(configuration()['version_code']), '-PprototypeAbi='+args.abi], ROOT/'platform/android')
    apk_name = filename(version()).replace('arm64-v8a', args.abi)
    apk = output/apk_name
    apk.write_bytes((ROOT/'platform/android/app/build/outputs/apk/release/app-release.apk').read_bytes())
    run(['python3', 'scripts/package-android.py', 'inspect-apk', '--apk', apk,
         '--report', output/'android-inspection.json'])
    if args.abi == 'arm64-v8a':
        run(['python3', 'scripts/android-release-manifest.py', '--artifacts', output])
    print('Completed public Android beta build:', source, args.abi)


if __name__ == '__main__':
    main()
