"""Pure-Python BVH parser — no bpy."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class BvhChannel:
    joint: str
    channel: str


@dataclass(frozen=True)
class BvhMotion:
    frame_time: float
    channels: tuple[BvhChannel, ...]
    frames: tuple[tuple[float, ...], ...]

    @property
    def fps(self) -> float:
        return 1.0 / self.frame_time if self.frame_time else 30.0

    @property
    def frame_count(self) -> int:
        return len(self.frames)

    def channel_index(self, joint: str, channel: str) -> int:
        for idx, item in enumerate(self.channels):
            if item.joint == joint and item.channel == channel:
                return idx
        raise KeyError(f"channel not found: {joint}.{channel}")

    def joint_series(self, joint: str, channel: str) -> tuple[float, ...]:
        idx = self.channel_index(joint, channel)
        return tuple(frame[idx] for frame in self.frames)


def parse_bvh(path: Path | str) -> BvhMotion:
    text = Path(path).read_text(encoding="utf-8")
    if "MOTION" not in text:
        raise ValueError("invalid BVH: missing MOTION section")
    hierarchy, motion = text.split("MOTION", 1)
    channels = _parse_hierarchy_channels(hierarchy)
    frame_time, frames = _parse_motion_block(motion)
    return BvhMotion(frame_time=frame_time, channels=channels, frames=frames)


def _parse_hierarchy_channels(hierarchy: str) -> tuple[BvhChannel, ...]:
    stack: list[str] = []
    channels: list[BvhChannel] = []
    for raw in hierarchy.splitlines():
        line = raw.strip()
        if line.startswith("ROOT") or line.startswith("JOINT"):
            stack.append(line.split()[1])
        elif line.startswith("CHANNELS"):
            parts = line.split()
            for channel in parts[2 : 2 + int(parts[1])]:
                channels.append(BvhChannel(joint=stack[-1], channel=channel))
        elif line == "}" and stack:
            stack.pop()
    return tuple(channels)


def _parse_motion_block(motion: str) -> tuple[float, list[tuple[float, ...]]]:
    lines = [ln.strip() for ln in motion.strip().splitlines() if ln.strip()]
    if len(lines) < 3:
        raise ValueError("invalid BVH motion block")
    frame_count = int(lines[0].split()[1])
    frame_time = float(lines[1].split()[2])
    frames: list[tuple[float, ...]] = []
    for line in lines[2 : 2 + frame_count]:
        values = tuple(float(part) for part in line.split())
        frames.append(values)
    if len(frames) != frame_count:
        raise ValueError(f"expected {frame_count} frames, found {len(frames)}")
    return frame_time, frames
