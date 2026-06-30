"""Generate shared Unity AnimatorController assets per TBG body class."""

from __future__ import annotations

import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CONTENT_OUTPUT_DIR = ROOT / "content" / "characters" / "animations"
PACKAGE_OUTPUT_DIR = ROOT / "unity" / "Packages" / "com.tbg.character-library" / "Animations"

TRIGGERS = ("Idle", "Spawn", "Attack", "Death", "Walk", "Ultimate")

BODY_CLASSES: dict[str, str] = {
    "HumanoidHeavy": "AttackHeavy",
    "HumanoidLight": "AttackRanged",
    "Mechanical": "AttackCast",
    "Organic": "AttackFast",
}


def build_controller_yaml(name: str, attack_state: str) -> str:
    states = ["Idle", "Spawn", attack_state, "Death", "Walk", "Ultimate"]
    state_ids = {state: 1_102_000_000_100 + index for index, state in enumerate(states)}
    machine_id = 1_107_000_000_200
    transition_ids = {
        trigger: 1_101_000_000_300 + index for index, trigger in enumerate(TRIGGERS)
    }
    trigger_to_state = {
        "Idle": "Idle",
        "Spawn": "Spawn",
        "Attack": attack_state,
        "Death": "Death",
        "Walk": "Walk",
        "Ultimate": "Ultimate",
    }

    lines = [
        "%YAML 1.1",
        "%TAG !u! tag:unity3d.com,2011:",
        "--- !u!91 &9100000",
        "AnimatorController:",
        "  m_ObjectHideFlags: 0",
        f"  m_Name: {name}",
        "  serializedVersion: 5",
        "  m_AnimatorParameters:",
    ]
    for trigger in TRIGGERS:
        lines.extend(_parameter_block(trigger))

    lines.extend(
        [
            "  m_AnimatorLayers:",
            "  - serializedVersion: 5",
            "    m_Name: Base Layer",
            f"    m_StateMachine: {{fileID: {machine_id}}}",
            "    m_Mask: {fileID: 0}",
            "    m_Motions: []",
            "    m_Behaviours: []",
            "    m_BlendingMode: 0",
            "    m_SyncedLayerIndex: -1",
            "    m_DefaultWeight: 0",
            "    m_IKPass: 0",
            "    m_SyncedLayerAffectsTiming: 0",
            "    m_Controller: {fileID: 9100000}",
        ]
    )

    for trigger in TRIGGERS:
        state = trigger_to_state[trigger]
        lines.extend(_transition_block(transition_ids[trigger], trigger, state_ids[state]))

    y_positions = {
        "Idle": 0,
        "Spawn": 120,
        attack_state: 240,
        "Death": 360,
        "Walk": 480,
        "Ultimate": 600,
    }
    for state in states:
        lines.extend(_state_block(state_ids[state], state, y_positions[state]))

    lines.extend(_state_machine_block(machine_id, state_ids, states, transition_ids))
    return "\n".join(lines) + "\n"


def _parameter_block(name: str) -> list[str]:
    return [
        "  - m_Name: " + name,
        "    m_Type: 9",
        "    m_DefaultFloat: 0",
        "    m_DefaultInt: 0",
        "    m_DefaultBool: 0",
        "    m_Controller: {fileID: 0}",
    ]


def _transition_block(transition_id: int, trigger: str, dst_state_id: int) -> list[str]:
    return [
        f"--- !u!1101 &{transition_id}",
        "AnimatorStateTransition:",
        "  m_ObjectHideFlags: 1",
        "  m_Conditions:",
        "  - m_ConditionMode: 1",
        f"    m_ConditionEvent: {trigger}",
        "    m_EventTreshold: 0",
        "  m_DstState: {fileID: " + str(dst_state_id) + "}",
        "  serializedVersion: 3",
        "  m_TransitionDuration: 0.1",
        "  m_HasExitTime: 0",
        "  m_HasFixedDuration: 1",
        "  m_CanTransitionToSelf: 1",
    ]


def _state_block(state_id: int, name: str, y_pos: int) -> list[str]:
    return [
        f"--- !u!1102 &{state_id}",
        "AnimatorState:",
        "  serializedVersion: 5",
        "  m_ObjectHideFlags: 1",
        f"  m_Name: {name}",
        "  m_Speed: 1",
        "  m_Transitions: []",
        f"  m_Position: {{x: 300, y: {y_pos}, z: 0}}",
        "  m_Motion: {fileID: 0}",
    ]


def _state_machine_block(
    machine_id: int,
    state_ids: dict[str, int],
    states: list[str],
    transition_ids: dict[str, int],
) -> list[str]:
    child_states = []
    for state in states:
        child_states.extend(
            [
                "  - serializedVersion: 1",
                f"    m_State: {{fileID: {state_ids[state]}}}",
                f"    m_Position: {{x: 300, y: {state_ids[state] % 400}, z: 0}}",
            ]
        )

    any_transitions = [f"  - {{fileID: {transition_ids[t]}}}" for t in TRIGGERS]
    return [
        f"--- !u!1107 &{machine_id}",
        "AnimatorStateMachine:",
        "  serializedVersion: 5",
        "  m_ObjectHideFlags: 1",
        "  m_Name: Base Layer",
        "  m_ChildStates:",
        *child_states,
        "  m_ChildStateMachines: []",
        "  m_AnyStateTransitions:",
        *any_transitions,
        "  m_EntryTransitions: []",
        "  m_StateMachineTransitions: {}",
        f"  m_DefaultState: {{fileID: {state_ids['Idle']}}}",
    ]


def generate_all(output_dirs: Path | list[Path] | None = None) -> list[Path]:
    if output_dirs is None:
        targets = [CONTENT_OUTPUT_DIR, PACKAGE_OUTPUT_DIR]
    elif isinstance(output_dirs, Path):
        targets = [output_dirs]
    else:
        targets = output_dirs
    written: list[Path] = []
    for body_class, attack_state in BODY_CLASSES.items():
        yaml_text = build_controller_yaml(body_class, attack_state)
        for output_dir in targets:
            output_dir.mkdir(parents=True, exist_ok=True)
            path = output_dir / f"{body_class}.controller"
            path.write_text(yaml_text, encoding="utf-8")
            written.append(path)
    return written


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate TBG body-class animator controllers.")
    parser.add_argument(
        "--output-dir",
        type=Path,
        action="append",
        help="Destination directory (repeatable; defaults to content + package paths)",
    )
    args = parser.parse_args()
    output_dirs = args.output_dir if args.output_dir else None
    for path in generate_all(output_dirs):
        print(path.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
