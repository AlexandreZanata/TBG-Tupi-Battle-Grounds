#!/usr/bin/env python3
"""Sign one native release, then optionally publish its immutable files and timestamp."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shlex
import subprocess


def deployment_plan(repo, host, remote):
    if not re.fullmatch(r'[A-Za-z0-9_.@-]+', host) or not re.fullmatch(r'/[A-Za-z0-9_./-]+', remote) or '..' in remote.split('/'):
        raise ValueError('Use a fixed SSH host and absolute static-repository path')
    base = host + ':' + remote.rstrip('/')
    common = ['rsync', '--archive', '--compress', '--chmod=D755,F644']
    return [
        ['ssh', '-o', 'BatchMode=yes', host, shlex.join(['install','-d','-m','755',remote,remote+'/targets',remote+'/metadata'])],
        [*common, '--ignore-existing', str(repo/'targets')+'/', base+'/targets/'],
        [*common, '--ignore-existing', '--exclude=timestamp.json', str(repo/'metadata')+'/', base+'/metadata/'],
        [*common, str(repo/'metadata/timestamp.json'), base+'/metadata/timestamp.json'],
    ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['publisher','keys','repo','package','descriptor','notes']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--previous-contract')
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--host', default='tupibg-vps')
    parser.add_argument('--remote', default='/srv/tbg/updates')
    args = parser.parse_args()
    repo = args.repo.resolve()
    keys = args.keys.resolve()
    if keys == repo or repo in keys.parents or keys in repo.parents:
        parser.error('Signing keys must live outside the public repository')
    descriptor = json.loads(args.descriptor.read_text())
    command = [str(args.publisher.resolve()),'publish','--keys',str(keys),'--repo',str(repo),
        '--package',str(args.package.resolve()),'--descriptor',str(args.descriptor.resolve()),'--notes',str(args.notes.resolve())]
    if args.previous_contract:
        command += ['--previous-contract',args.previous_contract]
    subprocess.run(command, check=True)
    root = repo/'metadata/1.root.json'
    root_hash = hashlib.sha256(root.read_bytes()).hexdigest()
    if args.publish:
        # Refuse to replace an existing authority with an unrelated root.
        remote_root = args.remote.rstrip('/')+'/metadata/1.root.json'
        plan = deployment_plan(repo,args.host,args.remote)
        probe = 'if test -f '+shlex.quote(remote_root)+'; then sha256sum '+shlex.quote(remote_root)+'; fi'
        existing = subprocess.check_output(['ssh','-o','BatchMode=yes',args.host,probe],text=True).split()
        if existing and existing[0] != root_hash:
            raise SystemExit('The existing update root differs; use reviewed TUF root rotation')
        for command in plan:
            subprocess.run(command,check=True)
    print(json.dumps({'signed_build':descriptor['build_id'],'contract':descriptor['compatibility']['match_contract'],
        'root_sha256':root_hash,'published':args.publish,'game_backend_modified':False},indent=2))


if __name__ == '__main__':
    main()
