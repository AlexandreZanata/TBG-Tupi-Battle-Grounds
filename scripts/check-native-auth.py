#!/usr/bin/env python3
"""Run real native auth/vault journeys with owned loopback processes and synthetic data."""
import argparse
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import tempfile
import time
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.error import HTTPError
from urllib.request import Request
from urllib.request import urlopen

def confirmation_proxy(backend):
    """Inject only two synthetic confirm failures; never log credential bodies."""
    class Handler(BaseHTTPRequestHandler):
        failures = 2
        def log_message(self, *_args):
            pass
        def forward(self):
            size = int(self.headers.get('Content-Length', '0'))
            if size > 4096:
                self.send_error(413)
                return
            body = self.rfile.read(size) if size else None
            if self.path.endswith('/confirm') and Handler.failures:
                Handler.failures -= 1
                status, payload = 503, b'{"error":"unavailable"}'
            else:
                request = Request(backend+self.path, data=body, method=self.command,
                    headers={key:value for key,value in self.headers.items() if key.lower() not in ('host','content-length','connection')})
                try:
                    with urlopen(request, timeout=6) as response:
                        status, payload = response.status, response.read(4097)
                except HTTPError as error:
                    status, payload = error.code, error.read(4097)
                except OSError:
                    status, payload = 503, b'{"error":"unavailable"}'
            self.send_response(status)
            self.send_header('Content-Type','application/json')
            self.send_header('Content-Length',str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        do_POST = forward
        do_GET = forward
    proxy = ThreadingHTTPServer(('127.0.0.1',0),Handler)
    threading.Thread(target=proxy.serve_forever,daemon=True).start()
    return proxy

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client',type=Path,required=True);parser.add_argument('--server',type=Path,required=True);parser.add_argument('--report',type=Path,required=True)
    parser.add_argument('--confirmation-outage',action='store_true',help='Inject a confirmation outage and test same-family/stale-vault recovery')
    args=parser.parse_args();client=args.client.resolve();server=args.server.resolve()
    client_identity=json.loads(subprocess.check_output([str(client),'--build-info'],text=True))
    with tempfile.TemporaryDirectory(prefix='tbg-p68-native-auth-') as temporary:
        root=Path(temporary)
        with socket.socket() as reserved:reserved.bind(('127.0.0.1',0));port=reserved.getsockname()[1]
        backend=f'http://127.0.0.1:{port}';proxy=confirmation_proxy(backend) if args.confirmation_outage else None
        origin=f'http://127.0.0.1:{proxy.server_port}' if proxy else backend
        server_env={**os.environ,'TBG_ADDR':f'127.0.0.1:{port}','TBG_DB':str(root/'synthetic.sqlite'),'TBG_WORKERS':'2','TBG_AUTH_ORIGINS':origin,'TBG_REGISTER_PER_IP_HOUR':'100'}
        running=None;journey=None
        with (root/'server.log').open('w') as log:
            def start():
                process=subprocess.Popen([str(server)],cwd=root,env=server_env,stdout=log,stderr=log)
                try:
                    for _ in range(100):
                        if process.poll() is not None:raise RuntimeError('owned synthetic backend stopped before readiness')
                        try:
                            with urlopen(origin+'/api/version',timeout=.3) as response:identity=json.load(response)
                            if identity.get('protocol')!=client_identity['protocol']:raise RuntimeError('synthetic backend protocol mismatch')
                            return process
                        except OSError:time.sleep(.05)
                    raise RuntimeError('synthetic backend readiness timed out')
                except BaseException:process.terminate();process.wait(timeout=10);raise
            def stop(process):
                if process and process.poll() is None:
                    process.terminate()
                    try:process.wait(timeout=10)
                    except subprocess.TimeoutExpired:process.kill();process.wait(timeout=5)
            def read_stage():
                if not select.select([journey.stdout],[],[],90)[0]:raise RuntimeError('native journey timed out; real vault may need owner unlock')
                line=journey.stdout.readline()
                if not line:raise RuntimeError('native journey ended before its acceptance stage')
                return json.loads(line)
            try:
                running=start()
                journey=subprocess.Popen([str(client),'--auth-native-journey','--auth-stepwise',*(['--auth-persistence-retry'] if proxy else []),'--server',origin.replace('http:','ws:')+'/ws'],cwd=root,env={**os.environ,'TBG_DATA_DIR':str(root/'client')},stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
                first=read_stage()
                if first.get('stage')!='ready':raise RuntimeError('native journey did not reach ready')
                stop(running);running=None
                journey.stdin.write('offline\n');journey.stdin.flush();pending=read_stage()
                if pending.get('stage')!='offline-pending':raise RuntimeError('native logout did not remain pending offline')
                running=start();journey.stdin.write('online\n');journey.stdin.flush();report=read_stage()
                journey.wait(timeout=15)
                if journey.returncode:raise RuntimeError('native acceptance returned a failure')
                required=['synthetic_loopback_only','process_coordination','ordinary_auth_metadata_contains_no_tokens','remote_revocation','offline_pending_observed']
                if proxy:required += ['confirmation_outage_recovered','stale_vault_recovered','secure_persistence']
                if not all(report.get(key) for key in required):raise RuntimeError('native acceptance contract failed')
                if report['secure_persistence'] and not all(report.get(key) for key in ['restored_same_family','migration_completed','vault_cleanup']):raise RuntimeError('real secure persistence acceptance incomplete')
                args.report.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
            finally:
                if journey and journey.poll() is None:journey.kill();journey.wait(timeout=5)
                stop(running)
                if proxy:proxy.shutdown();proxy.server_close()
if __name__=='__main__':main()
