#!/usr/bin/env python3
"""Local-only P30-D capacity probes in disposable user cgroups; never SSH."""

import argparse
import json
import os
import pathlib
import socket
import subprocess
import sys
import time
import urllib.request


def cpus(value):
    """Parse explicit CPU ids; server and generator must be disjoint."""
    return {int(x) for x in value.split(",")}


def child(binary, cpu, pidfile):
    """Record only our own PID before replacing the disposable child."""
    os.sched_setaffinity(0, cpus(cpu))
    pathlib.Path(pidfile).write_text(str(os.getpid()))
    os.execve(binary, [binary], os.environ)


def read_json(url):
    with urllib.request.urlopen(url, timeout=5) as response:
        return json.load(response)


def process(pid):
    text = pathlib.Path(f"/proc/{pid}/stat").read_text().split()
    return (int(text[13]) + int(text[14])) / os.sysconf("SC_CLK_TCK")


def rss(pid):
    for line in pathlib.Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith("VmRSS:"):
            return int(line.split()[1]) * 1024
    return 0


def wait_ready(runner, pidfile, log):
    for _ in range(200):
        if pidfile.exists():
            try:
                read_json("http://127.0.0.1:18730/metrics")
                return
            except OSError:
                pass
        if runner.poll() is not None:
            raise RuntimeError(log.read_text())
        time.sleep(0.1)
    raise RuntimeError("owned server did not become ready")


def wait_population(level):
    for _ in range(180):
        if read_json("http://127.0.0.1:18730/metrics")["matches_running"] >= level * 0.95:
            return
        time.sleep(0.5)
    raise RuntimeError("did not reach 95% of requested matches; no capacity claim")


def sample(window, pid, folder):
    samples = []
    start = time.monotonic()
    for _ in range(window // 5):
        time.sleep(5)
        metrics = read_json("http://127.0.0.1:18730/metrics")
        samples.append({"elapsed_s": time.monotonic() - start, "rss_bytes": rss(pid), **metrics})
        (folder / "samples.json").write_text(json.dumps(samples, indent=2) + "\n")
    return samples


def probe(args, level, output):
    """Stop only the scope created here; retain all owned logs and samples."""
    unit = f"tbg-p30d-{os.getpid()}-{level}.scope"
    folder = output / str(level)
    folder.mkdir(parents=True, exist_ok=True)
    pidfile = folder / "server.pid"
    if pidfile.exists():
        raise RuntimeError("use a new output directory; existing evidence is preserved")
    env = {**os.environ, "TBG_ADDR": "127.0.0.1:18730", "TBG_WORKERS": str(len(cpus(args.server_cpus))),
           "TBG_MAX_CONNECTIONS": "20000", "TBG_MAX_PER_IP": "20000",
           "TBG_DB": str(folder / "scratch.sqlite3"), "RUST_LOG": "error"}
    command = ["systemd-run", "--user", "--scope", "--quiet", f"--unit={unit}",
               "-p", f"CPUQuota={100 * len(cpus(args.server_cpus))}%", "-p", f"MemoryMax={args.memory}",
               sys.executable, str(pathlib.Path(__file__).resolve()), "--server-child",
               str(pathlib.Path(args.server).resolve()), args.server_cpus, str(pidfile)]
    bot = None
    with (folder / "server.log").open("w") as server_log, (folder / "bots.log").open("w") as bots_log:
        runner = subprocess.Popen(command, env=env, stdout=server_log, stderr=subprocess.STDOUT)
        try:
            wait_ready(runner, pidfile, folder / "server.log")
            pid = int(pidfile.read_text())
            bot = subprocess.Popen(["taskset", "-c", args.bot_cpus, str(pathlib.Path(args.bots).resolve()),
                                    "--url", "ws://127.0.0.1:18730/ws", "--matches", str(level),
                                    "--ramp-ms", "5", "--timeout-s", "900"],
                                   stdout=bots_log, stderr=subprocess.STDOUT,
                                   env={**os.environ, "TOKIO_WORKER_THREADS": str(len(cpus(args.bot_cpus)))})
            wait_population(level)
            time.sleep(3)
            before = read_json("http://127.0.0.1:18730/metrics")
            start, server_cpu, bot_cpu = time.monotonic(), process(pid), process(bot.pid)
            samples = sample(args.window, pid, folder)
            elapsed = time.monotonic() - start
            after = samples[-1]
            result = dict(label=args.label, requested_matches=level,
                          min_running=min(s["matches_running"] for s in samples), final_running=after["matches_running"],
                          window_s=elapsed, server_cpu_cores=(process(pid) - server_cpu) / elapsed,
                          generator_cpu_cores=(process(bot.pid) - bot_cpu) / elapsed,
                          peak_rss_bytes=max(s["rss_bytes"] for s in samples),
                          late_ticks=after["late_ticks"] - before["late_ticks"], rtt_p95_ms=after["rtt_p95_ms"],
                          bytes_per_player_second=(after["bytes_out"] - before["bytes_out"]) / elapsed / (2 * level),
                          server_cpus=args.server_cpus, generator_cpus=args.bot_cpus, memory_limit=args.memory,
                          cgroup=pathlib.Path(f"/proc/{pid}/cgroup").read_text().strip())
            result["valid_population"] = result["min_running"] >= level * 0.95
            group = pathlib.Path('/sys/fs/cgroup') / result['cgroup'].split('::', 1)[1].lstrip('/')
            result['cgroup_limits'] = {name: (group / name).read_text().strip()
                                      for name in ['cpu.max', 'memory.max', 'memory.peak', 'memory.events']}
            (folder / "result.json").write_text(json.dumps(result, indent=2) + "\n")
            return result
        finally:
            if bot is not None:
                bot.kill()
                bot.wait()
            # Capacity windows deliberately abort disposable matches. This
            # scope contains only the child created above, never live apps.
            subprocess.run(["systemctl", "--user", "kill", "--kill-whom=all", "--signal=KILL", unit],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
            runner.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--server", required=True)
    parser.add_argument("--bots", default="target/release/tbg-bots")
    parser.add_argument("--label", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--levels", default="250,500,1000,1500,2000")
    parser.add_argument("--window", type=int, default=60)
    parser.add_argument("--server-cpus", default="2")
    parser.add_argument("--bot-cpus", default="4,5,6,7")
    parser.add_argument("--memory", default="1G")
    args = parser.parse_args()
    assert cpus(args.server_cpus).isdisjoint(cpus(args.bot_cpus))
    assert args.window >= 5 and args.window % 5 == 0
    with socket.socket() as check:
        # Match the server's normal bind behavior without mistaking our
        # previous disposable connections in TIME_WAIT for a live listener.
        check.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        check.bind(("127.0.0.1", 18730))
    output = pathlib.Path(args.output).resolve()
    output.mkdir(parents=True, exist_ok=True)
    results = []
    for level in map(int, args.levels.split(",")):
        result = probe(args, level, output)
        results.append(result)
        (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
        print(json.dumps(result), flush=True)
        if result["late_ticks"] or not result["valid_population"] or result["generator_cpu_cores"] > len(cpus(args.bot_cpus)) * 0.85:
            print("Stopped increasing load: lateness, population loss, or generator saturation.", flush=True)
            break


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--server-child":
        child(*sys.argv[2:])
    else:
        main()
