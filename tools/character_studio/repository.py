"""JSON CRUD for content/characters/definitions — source of truth on disk."""

from __future__ import annotations

import json
from pathlib import Path

from .models import CharacterDefinition
from .paths import cards_dir, definitions_dir, previews_dir
from .validation import ValidationError, validate_definition


class CharacterRepository:
    def __init__(self, repo_root: Path | None = None) -> None:
        self._definitions = definitions_dir(repo_root)
        self._cards = cards_dir(repo_root)
        self._previews = previews_dir(repo_root)
        self._definitions.mkdir(parents=True, exist_ok=True)
        self._cards.mkdir(parents=True, exist_ok=True)
        self._previews.mkdir(parents=True, exist_ok=True)

    def list_ids(self) -> list[str]:
        ids: list[str] = []
        for path in sorted(self._definitions.glob("*.json")):
            if path.name.startswith("_"):
                continue
            ids.append(path.stem)
        return ids

    def load(self, unit_id: str) -> CharacterDefinition:
        path = self._definitions / f"{unit_id}.json"
        if not path.is_file():
            raise FileNotFoundError(f"definition not found: {unit_id}")
        data = json.loads(path.read_text(encoding="utf-8"))
        return CharacterDefinition.from_dict(data)

    def create(self, defn: CharacterDefinition) -> Path:
        if defn.id in self.list_ids():
            raise ValidationError([f"duplicate id: {defn.id}"])
        validate_definition(defn, existing_ids=set(self.list_ids()))
        return self._write(defn)

    def save(self, defn: CharacterDefinition) -> Path:
        existing = set(self.list_ids()) - {defn.id}
        validate_definition(defn, existing_ids=existing)
        return self._write(defn)

    def _write(self, defn: CharacterDefinition) -> Path:
        path = self._definitions / f"{defn.id}.json"
        path.write_text(json.dumps(defn.to_dict(), indent=2) + "\n", encoding="utf-8")
        return path

    def delete(self, unit_id: str) -> None:
        json_path = self._definitions / f"{unit_id}.json"
        if not json_path.is_file():
            raise FileNotFoundError(f"definition not found: {unit_id}")
        json_path.unlink()
        for folder in (self._cards, self._previews):
            asset = folder / f"{unit_id}.png"
            if asset.is_file():
                asset.unlink()

    def card_path(self, unit_id: str) -> Path:
        return self._cards / f"{unit_id}.png"

    def preview_path(self, unit_id: str) -> Path:
        return self._previews / f"{unit_id}.png"
