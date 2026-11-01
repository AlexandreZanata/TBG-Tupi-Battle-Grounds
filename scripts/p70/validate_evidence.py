#!/usr/bin/env python3
"""Validate P70's frozen cohort and evidence integrity without compiling the game.

A successful default run validates bookkeeping, not visual completion. Use
--require-complete at phase acceptance; absent captures never count as passes.
Use --metadata-only for bookkeeping updates that reuse capture-time integrity
checks; its output distinguishes recorded coverage from reverified originals.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import struct
import subprocess
import sys
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "docs/planning/P70-VISUAL-MANIFEST.json"
INDEX = ROOT / "docs/planning/P70-VISUAL-EVIDENCE.json"
COMMON_CLIPS = {"Idle", "Walk", "Hit", "Death", "Spawn", "Evolve", "Victory"}
LOOPING = {"Idle", "Walk", "Victory"}
EVENT_TIMES = {"AttackMelee": 0.30, "AttackRanged": 0.18, "Cast": 0.22}
REVIEW_REQUIREMENTS = (
    "five_forms_at_match_distance", "front_profile_back", "nameless_lineup",
    "silhouette_lineup", "temporal_clips_and_transitions", "live_cues_and_effects",
    "troops_preview", "training", "p63_victory", "blue_artifact_review",
)


def read_json(path):
    return json.loads(path.read_text())


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def local_path(uri):
    if urlparse(uri).scheme:
        raise ValueError(f"External artifact is not locally available: {uri}")
    path = Path(uri)
    return path if path.is_absolute() else ROOT / path


def linked_captures(index):
    captures, errors = [], []
    for link in index.get("capture_manifests", []):
        uri = link.get("path_or_uri", "<missing>")
        try:
            path = local_path(uri)
            if not path.is_file():
                raise ValueError("linked capture manifest unavailable on this machine")
            if not re.fullmatch(r"[0-9a-f]{64}", link.get("sha256", "")):
                raise ValueError("linked manifest needs a valid SHA-256")
            if sha256(path) != link["sha256"]:
                raise ValueError("linked capture manifest SHA-256 mismatch")
            data = read_json(path)
            records = data if isinstance(data, list) else data["captures"]
            if len(records) != link["capture_count"]:
                raise ValueError("linked capture count differs from the index")
            for field in ("content_sha", "tooling_sha", "phase_before_or_after"):
                if any(record.get(field) != link[field] for record in records):
                    raise ValueError(f"linked captures differ from recorded {field}")
            captures.extend(records)
        except (OSError, ValueError, KeyError, TypeError) as error:
            errors.append(f"{uri}: {error}")
    return captures, errors


def roster_errors(manifest, index):
    errors = []
    source = (ROOT / "crates/tbg-core/src/types.rs").read_text()
    pool = re.search(r"FULL_POOL: \[UnitKind; (\d+)\] = \[([\s\S]*?)\];", source)
    if not pool:
        return ["Could not read FULL_POOL from core types.rs"]
    ids = re.findall(r"UnitKind::(\w+)", pool[2])
    manifest_ids = [c["id"] for c in manifest["champions"]]
    index_ids = [c["id"] for c in index["champions"]]
    if len(ids) != 46 or len(set(ids)) != 46 or set(ids) != set(manifest_ids):
        errors.append("Manifest does not match the fixed 46 unique FULL_POOL IDs")
    if len(index_ids) != 46 or set(index_ids) != set(ids):
        errors.append("Evidence index does not contain exactly the 46 champion IDs")
    forms = [(f["name"], f["tier"], f["branch"]) for f in manifest["forms"]]
    if forms != [("Base", 0, 0), ("A1", 1, 0), ("A2", 2, 0), ("B1", 1, 1), ("B2", 2, 1)]:
        errors.append("Manifest form/branch mapping changed")
    if index["forms"] != manifest["forms"] or index["clip_ids"] != manifest["clip_ids"]:
        errors.append("Evidence form/clip axes differ from the manifest")
    form_source = (ROOT / "crates/tbg-client/src/art/mod.rs").read_text()
    form_row = re.search(r"pub const FORMS:[^=]+=(.*?);", form_source)
    actual_forms = re.findall(r"\((\d+),\s*(\d+)\)", form_row[1]) if form_row else []
    if [(int(tier), int(branch)) for tier, branch in actual_forms] != [(f[1], f[2]) for f in forms]:
        errors.append("FigureLibrary form mapping differs from the frozen five forms")
    if len(ids) * len(forms) != 230 or len(ids) * len(forms) * len(index["clip_ids"]) != 2300:
        errors.append("Expected 230 forms and 2,300 clip/form slots per revision")
    registry = (ROOT / "crates/tbg-client/src/art/kits/mod.rs").read_text()
    for champion in manifest["champions"]:
        unit = champion["id"]
        folder = ROOT / champion["kit_directory"]
        if not folder.is_dir() or not (folder / "mod.rs").is_file():
            errors.append(f"{unit}: kit directory/module missing")
        mapping = re.search(rf"UnitKind::{unit}\s*=>\s*Some\(&(\w+)\)", registry)
        if not mapping:
            errors.append(f"{unit}: kit registry mapping missing")
        elif not re.search(rf"static {mapping[1]}: Kit = {folder.name}::KIT;", registry):
            errors.append(f"{unit}: kit registry points outside its manifest directory")
    auxiliaries = set(manifest["excluded_auxiliary_ids"])
    if auxiliaries & set(ids) or auxiliaries != {a["id"] for a in index["auxiliaries"]}:
        errors.append("Auxiliary inventory changed or counted as champions")
    return errors


def applicability(champion, clip, index):
    return champion["applicability"].get(clip, index["common_applicability"].get(clip))


def source_errors(index):
    errors = []
    for champion in index["champions"]:
        for clip in index["clip_ids"]:
            entry = applicability(champion, clip, index)
            if not entry or entry.get("status") not in {"applicable", "not_applicable", "pending_source_review"}:
                errors.append(f'{champion["id"]}/{clip}: missing applicability decision')
                continue
            if not entry.get("reason") or not entry.get("sources"):
                errors.append(f'{champion["id"]}/{clip}: applicability needs reason and source')
            for source in entry.get("sources", []):
                if not (ROOT / source.split("#", 1)[0]).is_file():
                    errors.append(f'{champion["id"]}/{clip}: source missing: {source}')
    return errors


def finite_number(value):
    return isinstance(value, (int, float)) and not isinstance(value, bool) and math.isfinite(value)


def capture_errors(capture, ids, forms):
    errors = []
    required = (
        "id", "unit_id", "forms", "clip_or_ability", "phase_before_or_after",
        "samples", "kind", "content_sha", "tooling_sha", "camera", "aim",
        "viewport", "dpi", "quality", "platform", "renderer", "scenario_seed",
        "path_or_uri", "sha256", "width", "height",
    )
    missing = [key for key in required if key not in capture]
    if missing:
        return [f"missing metadata: {', '.join(missing)}"]
    if capture["unit_id"] not in ids:
        errors.append("unit_id outside frozen champion/auxiliary inventory")
    if not capture["forms"] or not set(capture["forms"]) <= forms:
        errors.append("unknown/empty form selection")
    if capture["phase_before_or_after"] not in {"before", "after"}:
        errors.append("phase must be before or after")
    for key, length in [("content_sha", 40), ("tooling_sha", 40), ("sha256", 64)]:
        if not isinstance(capture[key], str) or not re.fullmatch(rf"[0-9a-f]{{{length}}}", capture[key]):
            errors.append(f"invalid {key}")
    if not capture["camera"] or not capture["renderer"]:
        errors.append("camera and observed renderer metadata required")
    if not finite_number(capture["dpi"]) or capture["dpi"] <= 0:
        errors.append("dpi must be positive")
    width, height = capture["width"], capture["height"]
    if not all(isinstance(v, int) and v > 0 for v in [width, height]):
        errors.append("image dimensions must be positive integers")
    if capture["viewport"] != [width, height] and capture["kind"] == "raw_pose_sheet":
        errors.append("raw screenshot dimensions differ from viewport")
    for sample in capture["samples"]:
        if sample.get("form") not in capture["forms"]:
            errors.append("sample form outside capture forms")
        time, duration = sample.get("sample_time"), sample.get("duration")
        if not finite_number(time) or not finite_number(duration) or duration <= 0 or not 0 <= time <= duration + 1e-4:
            errors.append("invalid sample time or actual duration")
        if not sample.get("sample_label") or not sample.get("clip"):
            errors.append("sample clip/label missing")
    return errors


def artifact_errors(capture, metadata_only=False):
    uri = capture["path_or_uri"]
    if urlparse(uri).scheme:
        return ["external artifact not locally verified; download through the approved artifact workflow first"]
    path = Path(uri)
    if not path.is_absolute():
        path = ROOT / path
    if not path.is_file():
        return [f"artifact unavailable: {path}"]
    errors = []
    if capture["kind"] == "raw_pose_sheet" and path.suffix.lower() != ".png":
        errors.append("raw pose evidence must preserve the renderer's original PNG")
    if metadata_only:
        # Availability is checked now; capture_roster owns the recorded byte/decode checks.
        return errors
    if sha256(path) != capture["sha256"]:
        errors.append("artifact SHA-256 mismatch")
    if path.suffix.lower() == ".png":
        try:
            subprocess.run(["convert", "-regard-warnings", str(path), "null:"],
                           check=True, capture_output=True, timeout=30)
        except (OSError, subprocess.SubprocessError) as error:
            errors.append(f"full PNG decode failed: {error}")
        with path.open("rb") as image:
            header = image.read(24)
        if len(header) != 24 or header[:8] != b"\x89PNG\r\n\x1a\n" or header[12:16] != b"IHDR":
            errors.append("invalid PNG header")
        elif list(struct.unpack(">II", header[16:24])) != [capture["width"], capture["height"]]:
            errors.append("PNG dimensions do not match metadata")
    return errors


def complete_samples(captures, unit, phase, form, clip):
    samples = [sample for capture in captures
               if capture["unit_id"] == unit and capture["phase_before_or_after"] == phase
               and capture["kind"] == "raw_pose_sheet"
               for sample in capture["samples"]
               if sample["form"] == form and sample["clip"] == clip]
    if not samples:
        return False
    duration = samples[0]["duration"]
    if any(abs(s["duration"] - duration) > 1e-4 for s in samples):
        return False
    required = [duration * fraction for fraction in (0, 0.25, 0.5, 0.75)]
    if clip in EVENT_TIMES:
        required.append(EVENT_TIMES[clip])
    if clip not in LOOPING:
        required.append(duration)
    return all(any(abs(s["sample_time"] - wanted) < 1e-4 for s in samples) for wanted in required)


def summarize(index, captures):
    forms = [f["name"] for f in index["forms"]]
    missing = []
    covered = {phase: 0 for phase in ("before", "after")}
    applicable = 0
    artifact_ids = {capture["id"] for capture in captures}
    for champion in index["champions"]:
        unit = champion["id"]
        for clip in index["clip_ids"]:
            decision = applicability(champion, clip, index)
            if not decision or decision["status"] == "pending_source_review":
                missing.append(f"{unit}/{clip}: applicability unresolved")
                continue
            if decision["status"] == "not_applicable":
                continue
            applicable += len(forms)
            for phase in covered:
                for form in forms:
                    if complete_samples(captures, unit, phase, form, clip):
                        covered[phase] += 1
                    else:
                        missing.append(f"{unit}/{phase}/{form}/{clip}: pose samples pending")
        for requirement in REVIEW_REQUIREMENTS:
            review = champion["review_requirements"].get(requirement, {})
            paired_ids = review.get("before", []) + review.get("after", [])
            if (review.get("status") != "reviewed" or not review.get("before")
                    or not review.get("after") or not set(paired_ids) <= artifact_ids):
                missing.append(f"{unit}: {requirement} pending")
        for action in champion["event_driven_actions"]:
            paired_ids = action["before"] + action["after"]
            if (action["review_status"] != "reviewed" or not action["before"]
                    or not action["after"] or not set(paired_ids) <= artifact_ids):
                missing.append(f'{unit}: {action["label"]} live evidence pending')
        if champion["owner_visual_approval"] != "approved":
            missing.append(f"{unit}: owner visual approval pending")
    for auxiliary in index["auxiliaries"]:
        if auxiliary["review_status"] != "reviewed":
            missing.append(f'{auxiliary["id"]}: auxiliary review pending')
    for requirement, status in index["phase_acceptance"].items():
        if status != "accepted":
            missing.append(f"phase: {requirement} pending")
    return covered, applicable, missing


def reference_errors(index, captures):
    errors = []
    by_id = {capture["id"]: capture for capture in captures}
    for entry in index["champions"] + index["auxiliaries"]:
        for phase in ("before", "after"):
            references = entry[phase]
            if len(references) != len(set(references)):
                errors.append(f'{entry["id"]}: duplicate {phase} references')
            for reference in references:
                capture = by_id.get(reference)
                if capture is None:
                    errors.append(f'{entry["id"]}: unavailable {phase} capture reference {reference}')
                elif capture["unit_id"] != entry["id"] or capture["phase_before_or_after"] != phase:
                    errors.append(f'{entry["id"]}: mismatched {phase} capture reference {reference}')
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--index", type=Path, default=INDEX)
    parser.add_argument("--captures", type=Path, action="append", default=[],
                        help="Additional capture array or object containing a captures array")
    parser.add_argument("--require-complete", action="store_true")
    parser.add_argument("--metadata-only", action="store_true",
                        help="Check manifest hashes, metadata and file availability; do not rehash/decode original PNGs")
    parser.add_argument("--json", action="store_true", help="Print the summary as JSON")
    args = parser.parse_args()
    try:
        manifest, index = read_json(MANIFEST), read_json(args.index)
        errors = roster_errors(manifest, index) + source_errors(index)
        captures = list(index["captures"])
        linked, link_errors = linked_captures(index)
        captures.extend(linked)
        errors.extend(link_errors)
        for path in args.captures:
            extra = read_json(path)
            captures.extend(extra if isinstance(extra, list) else extra["captures"])
        valid, seen = [], set()
        ids = {c["id"] for c in index["champions"] + index["auxiliaries"]}
        forms = {f["name"] for f in index["forms"]}
        for capture in captures:
            name = capture.get("id", "<unnamed>")
            found = capture_errors(capture, ids, forms)
            if name in seen:
                found.append("duplicate capture ID")
            seen.add(name)
            if not found:
                found.extend(artifact_errors(capture, args.metadata_only))
            errors.extend(f"{name}: {error}" for error in found)
            if not found:
                valid.append(capture)
        covered, applicable, missing = summarize(index, valid)
        errors.extend(reference_errors(index, valid))
        verified = valid if not args.metadata_only else []
        verified_covered = covered if not args.metadata_only else {"before": 0, "after": 0}
        summary = {
            "bookkeeping_valid": not errors,
            "phase_complete": not errors and not missing and not args.metadata_only,
            "champions": len(index["champions"]), "forms": 230,
            "clip_form_slots_per_revision": 2300,
            "applicable_clip_form_slots_per_revision": applicable,
            "recorded_raw_pose_slots": covered, "recorded_artifacts": len(captures),
            "locally_available_artifacts": len(valid),
            "verified_raw_pose_slots": verified_covered, "verified_artifacts": len(verified),
            "original_integrity_rechecked": not args.metadata_only,
            "pending_items": len(missing), "errors": errors,
            "note": ("Bookkeeping validity does not imply implemented art or passed visual review. "
                     + ("Original hashes, PNG dimensions and full decode were not repeated; recorded capture-time checks are reused."
                        if args.metadata_only else "Original hashes, PNG dimensions and full decode were checked in this invocation.")),
        }
        if args.json:
            print(json.dumps(summary, indent=2))
        else:
            print(f"46 champions / 230 forms / 2,300 clip-form slots per revision")
            print(f"Applicable pose slots: {applicable}; recorded before: {covered['before']}; after: {covered['after']}")
            print(f"Recorded artifacts: {len(captures)}; available: {len(valid)}; reverified: {len(verified)}; pending acceptance items: {len(missing)}")
            for error in errors:
                print(f"ERROR: {error}", file=sys.stderr)
            print(summary["note"])
        return 1 if errors or (args.require_complete and not summary["phase_complete"]) else 0
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Invalid evidence input: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
