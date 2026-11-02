#!/usr/bin/env python3
"""Fetch public main candidates over HTTPS; no inbound runner, SSH key or GitHub token."""
import argparse
import json
import re
import subprocess
import tempfile
from pathlib import Path

import slots
from activate import JOURNAL, STATE, activate, check_compatibility, recover, write_json
from control import Deferred, deployment_lock, hold, release_hold, require_no_hold, run_controller
from github import api, candidate, fetch
from policy import MAX_ARCHIVE, REPOSITORY, WORKFLOW, require, unpack


def verify(package, bundle, source, workflow=WORKFLOW, source_ref='refs/heads/main'):
    subprocess.run([
        'gh', 'attestation', 'verify', str(package), '--bundle', str(bundle),
        '--repo', REPOSITORY, '--signer-workflow', workflow,
        '--source-ref', source_ref, '--source-digest', source,
        '--deny-self-hosted-runners',
    ], check=True, stdout=subprocess.DEVNULL, timeout=60)


def slot_config(config):
    """The public API origin the promoted slot is checked through."""
    return {'public_api': config.get('public_api') or config['public_health'].removesuffix('/health')}


def resume_transition(host, config):
    """Finish whatever the previous run left half-done: a slot promotion or a single-process rollback."""
    checked = STATE/'checked.json'
    try:
        if host is None:
            recover()
        else:
            slots.resume(host, slot_config(config))
    except Deferred:
        raise
    except slots.Rejected as error:
        write_json(checked, {'source_commit': error.source, 'outcome': 'rejected', 'reason': str(error)[:500]})
        raise
    except BaseException as error:
        # A refused rollback or cleanup needs a person; a busy database or a failed command retries on the next run.
        if host is None or isinstance(error, slots.OperatorRequired):
            hold('backend', f'recovery failed: {error}')
        raise


def run(config, apply, recheck, maintenance_window=False, ignore_lobby=False):
    require_no_hold()
    host = slots.SystemHost()
    slot_mode = slots.in_slot_mode(host)
    if apply:
        resume_transition(host if slot_mode else None, config)
    release = candidate()
    if release is None:
        print('Waiting for the first CI-approved main backend candidate.')
        return
    source = release['tag_name'][7:]
    checked = STATE/'checked.json'
    if not recheck and checked.exists() and json.loads(checked.read_text()).get('source_commit') == source:
        print('Candidate already handled; inspect checked.json for its outcome.')
        return
    current = STATE/'current.json'
    floor = json.loads(current.read_text())['source_commit'] if current.exists() else config['minimum_source']
    require(re.fullmatch(r'[0-9a-f]{40}', floor) is not None, 'verified deployment floor required')
    compare = api('/compare/'+floor+'...'+source)
    require(compare['status'] in ['ahead', 'identical'], 'candidate does not descend from deployment floor')
    if current.exists() and json.loads(current.read_text()).get('source_commit') == source:
        print('Candidate is the current verified deployment.')
        return
    download_and_handle(release, source, config, apply, maintenance_window, host if slot_mode else None, ignore_lobby)


def activate_in_slot(host, staged, manifest, config, source, ignore_lobby=False):
    """Promote through the idle release slot; only a state that needs a person holds the controller."""
    checked = STATE/'checked.json'
    try:
        slots.activate(host, staged, manifest, slot_config(config), ignore_lobby)
    except slots.Rejected as error:
        write_json(checked, {'source_commit': source, 'outcome': 'rejected', 'reason': str(error)[:500]})
        raise
    except slots.OperatorRequired as error:
        write_json(checked, {'source_commit': source, 'outcome': 'operator_required', 'reason': str(error)[:500]})
        hold('backend', f'slot transition needs an operator: {error}')
        raise
    except Deferred:
        # Promoted and serving, previous slot still draining: the next run finishes it.
        current = STATE/'current.json'
        if current.exists() and json.loads(current.read_text()).get('source_commit') == source:
            write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
        raise
    write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
    print('Promoted CI-approved compatible source '+source+' through the idle release slot')


def download_and_handle(release, source, config, apply, maintenance_window=False, host=None, ignore_lobby=False):
    checked = STATE/'checked.json'
    if apply and host is not None:
        # Nothing is downloaded while the host cannot start a release (a busy lobby, a previous release still draining).
        slots.gate(host, ignore_lobby)
    with tempfile.TemporaryDirectory(prefix='candidate-', dir=STATE) as temporary:
        folder = Path(temporary)
        package = folder/'backend.tar.gz'
        bundle = folder/'provenance.jsonl'
        prefix = 'https://github.com/'+REPOSITORY+'/releases/download/'+release['tag_name']+'/'
        names = {'tbg-backend-'+source+'.tar.gz': package, 'provenance.jsonl': bundle}
        for name, destination in names.items():
            asset = next((a for a in release['assets'] if a['name'] == name), None)
            require(asset is not None and asset['browser_download_url'] == prefix+name, 'missing or unexpected asset URL')
            destination.write_bytes(fetch(prefix+name, MAX_ARCHIVE if destination == package else 8*1024*1024))
        try:
            verify(package, bundle, source)
            manifest = unpack(package, folder/'verified', source)
        except (subprocess.CalledProcessError, ValueError) as error:
            # A forged or malformed candidate never becomes valid: latch it so it
            # is not downloaded again every run. Network failures stay retryable.
            if apply:
                write_json(checked, {'source_commit': source, 'outcome': 'rejected', 'reason': str(error)[:500]})
            raise
        if not apply:
            if host is None:
                check_compatibility(manifest)
            else:
                slots.preflight(host, manifest)
            print(json.dumps({'verified_source': source, 'contract': manifest['contract'], 'database_schema': manifest['database_schema'],
                              'mode': 'single' if host is None else 'slots', 'activated': False}))
            return
        if host is not None:
            activate_in_slot(host, folder/'verified', manifest, config, source, ignore_lobby)
            return
        # Automation never stops the only game process: a single-process
        # restart interrupts players and needs an explicit operator window.
        require(maintenance_window, 'single-process restart needs --maintenance-window; automatic delivery requires release slots')
        try:
            activate(folder/'verified', manifest, config['public_health'])
        except Exception as error:
            write_json(checked, {'source_commit': source, 'outcome': 'operator_required', 'reason': str(error)[:500]})
            if JOURNAL.exists():
                hold('backend', f'rollback did not complete: {error}')
            raise
        write_json(checked, {'source_commit': source, 'outcome': 'healthy'})
        print('Activated CI-approved compatible source '+source)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', type=Path, default=Path('/etc/tbg/auto-deploy.json'))
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--recheck', action='store_true')
    parser.add_argument('--maintenance-window', action='store_true',
                        help='operator-attended single-process restart; never used by the timer')
    parser.add_argument('--ignore-lobby', action='store_true',
                        help='operator only: promote without waiting for an empty lobby (waiters are refused and ask again)')
    parser.add_argument('--release-hold', action='store_true', help='clear a reviewed operator hold first')
    args = parser.parse_args()
    config = json.loads(args.config.read_text())

    def body():
        with deployment_lock():
            if args.release_hold:
                release_hold()
            run(config, args.apply, args.recheck, args.maintenance_window, args.ignore_lobby)
    run_controller(body)


if __name__ == '__main__':
    main()
