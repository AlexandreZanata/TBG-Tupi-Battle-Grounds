# TBG Unity Project

Open `unity/` as a Unity 2022.3 LTS project.

## Character Studio

1. Open scene `Assets/TBG/CharacterSystem/Scenes/CharacterStudioTest.unity`
2. Menu **TBG → Character Studio**
3. Definitions read/write `content/characters/definitions/*.json` at repo root

## Headless pipeline (CI)

```bash
./scripts/verify.sh
python3 -m tools.character_studio export-all-cards
```

Card PNGs are also exportable from Unity via **Export Card** when `CardRenderer` is in scene.
