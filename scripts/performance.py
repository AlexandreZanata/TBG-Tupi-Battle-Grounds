#!/usr/bin/env python3
"""Run P30-D native benchmarks; compare complete traces, never just hashes."""

import argparse
import csv
import hashlib
import io
import json
import pathlib
import subprocess
import tempfile
import time


def matrix(args):
    """Run seeded release samples on one specified CPU, preserving raw rows."""
    output = pathlib.Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    metadata = []
    started = time.time()
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    source_diff = subprocess.check_output(["git", "diff", "HEAD", "--", "crates", "scripts", "Cargo.toml", "Cargo.lock"])
    (output / f"{args.label}-source.patch").write_bytes(source_diff)
    for seed in args.seeds.split(","):
        command = [
            "taskset", "-c", args.cpu, str(pathlib.Path(args.binary).resolve()),
            "--ticks", str(args.ticks), "--seed", seed, "--sizes", args.sizes,
            "--scenes", args.scenes,
        ]
        run = subprocess.run(command, capture_output=True, text=True, check=True)
        lines = run.stdout.splitlines()
        metadata.append(lines[0])
        for row in csv.DictReader(io.StringIO("\n".join(lines[1:]))):
            rows.append({"label": args.label, **row})
        print(f"{args.label}: seed {seed} complete ({len(rows)} rows)", flush=True)
        with (output / f"{args.label}.csv").open("w") as handle:
            writer = csv.DictWriter(handle, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)
    provenance = {
        "binary": str(pathlib.Path(args.binary).resolve()),
        "binary_sha256": hashlib.sha256(pathlib.Path(args.binary).read_bytes()).hexdigest(),
        "git_head": revision,
        "working_tree_diff_sha256": hashlib.sha256(source_diff).hexdigest(),
        "source_capture": "Tracked source at run start; binary identity is independently hashed.",
        "arguments": vars(args), "metadata": metadata,
        "elapsed_s": round(time.time() - started, 3),
    }
    (output / f"{args.label}.json").write_text(json.dumps(provenance, indent=2) + "\n")


def compare(args):
    """Compare length-framed full-state/event/view traces byte for byte."""
    total = 0
    for seed in args.seeds.split(","):
        for scene in args.scenes.split(","):
            with tempfile.TemporaryDirectory(prefix="tbg-perf-trace-") as work:
                paths = [pathlib.Path(work) / name for name in ("reference", "candidate")]
                for binary, path in zip([args.binary, args.compare], paths):
                    command = [
                        str(pathlib.Path(binary).resolve()), "--trace",
                        "--seed", seed, "--ticks", str(args.ticks),
                        "--sizes", args.sizes, "--scenes", scene,
                    ]
                    with path.open("wb") as handle:
                        subprocess.run(command, stdout=handle, check=True)
                with paths[0].open("rb") as baseline, paths[1].open("rb") as candidate:
                    frame = 0
                    while header := baseline.read(8):
                        assert candidate.read(8) == header, (scene, seed, frame, "length")
                        length = int.from_bytes(header, "little")
                        assert baseline.read(length) == candidate.read(length), (scene, seed, frame)
                        frame += 1
                    assert not candidate.read(1), "extra candidate state"
                total += frame
                print(f"exact equality: {scene}, seed {seed}, {frame} states", flush=True)
    print(f"EXACT_TRACES_PASS: {total} complete states/events/both views", flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--compare")
    parser.add_argument("--output", default=".local/p30d/results")
    parser.add_argument("--label", default="baseline")
    parser.add_argument("--cpu", default="2")
    parser.add_argument("--ticks", type=int, default=240)
    parser.add_argument("--sizes", default="10,20,50,100,200,300,500,1000")
    parser.add_argument("--seeds", default="1,7,42")
    parser.add_argument("--scenes", default="melee,ranged,healing,mixed,formation,puppet,cargo")
    args = parser.parse_args()
    if args.compare:
        compare(args)
    else:
        matrix(args)


if __name__ == "__main__":
    main()
