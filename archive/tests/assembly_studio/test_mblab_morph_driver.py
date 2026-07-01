"""Morph driver influence math for browser preview."""

from __future__ import annotations


def morph_influences(value: float, neutral: float = 0.5) -> tuple[float, float]:
    return max(0.0, (value - neutral) * 2.0), max(0.0, (neutral - value) * 2.0)


def test_neutral_is_zero() -> None:
    assert morph_influences(0.5) == (0.0, 0.0)


def test_max_at_one() -> None:
    assert morph_influences(1.0) == (1.0, 0.0)


def test_min_at_zero() -> None:
    assert morph_influences(0.0) == (0.0, 1.0)
