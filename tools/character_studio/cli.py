"""Character Studio CLI — create, edit, delete definitions and export cards."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from .card_renderer import render_card
from .defaults import create_default_definition
from .models import CharacterDefinition
from .paths import find_repo_root
from .repository import CharacterRepository


def main(argv: list[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)
    repo = CharacterRepository(find_repo_root())

    if args.command == "list":
        for unit_id in repo.list_ids():
            print(unit_id)
        return 0

    if args.command == "create":
        defn = create_default_definition(args.id, args.archetype, args.faction, args.name)
        repo.create(defn)
        print(f"created {args.id}")
        return 0

    if args.command == "show":
        print(json.dumps(repo.load(args.id).to_dict(), indent=2))
        return 0

    if args.command == "delete":
        repo.delete(args.id)
        print(f"deleted {args.id}")
        return 0

    if args.command == "export-card":
        defn = repo.load(args.id)
        path = render_card(defn, repo.card_path(args.id))
        print(path)
        return 0

    if args.command == "export-preview":
        defn = repo.load(args.id)
        path = render_card(defn, repo.preview_path(args.id))
        print(path)
        return 0

    if args.command == "export-all-cards":
        for unit_id in repo.list_ids():
            defn = repo.load(unit_id)
            path = render_card(defn, repo.card_path(unit_id))
            print(path)
        return 0

    parser.print_help()
    return 1


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="TBG Character Studio CLI")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("list", help="List character definition ids")

    create = sub.add_parser("create", help="Create definition JSON")
    create.add_argument("id")
    create.add_argument("archetype")
    create.add_argument("faction")
    create.add_argument("--name", default=None)

    show = sub.add_parser("show", help="Print definition JSON")
    show.add_argument("id")

    delete = sub.add_parser("delete", help="Delete definition and assets")
    delete.add_argument("id")

    export = sub.add_parser("export-card", help="Render card PNG")
    export.add_argument("id")

    preview = sub.add_parser("export-preview", help="Render preview PNG")
    preview.add_argument("id")

    sub.add_parser("export-all-cards", help="Render all card PNGs")
    return parser


if __name__ == "__main__":
    sys.exit(main())
