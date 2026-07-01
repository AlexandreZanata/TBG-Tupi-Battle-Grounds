# Character Studio — static assets

```
static/
├── index.html              # Studio UI entry
├── showcase.html           # Showcase carousel entry
├── animation-studio.html   # Game mesh clip preview
├── css/
│   ├── studio.css          # Shared + studio layout
│   ├── showcase.css        # Showcase + modal presentation
│   └── animation-studio.css
└── js/
    ├── config/
    │   └── previewLimits.js
    ├── viewer/             # Three.js viewer pipeline (shared)
    │   ├── viewer3d.js
    │   ├── viewerLighting.js
    │   ├── viewerMaterials.js
    │   ├── viewerResize.js
    │   ├── gltfLoader.js
    │   ├── gltfCache.js
    │   ├── proceduralMesh.js
    │   ├── proceduralClips.js
    │   └── viewerSceneUtils.js
    ├── studio/             # Character Studio app
    │   ├── app.js
    │   └── behaviorPlayback.js
    ├── animation-studio/   # Animation Studio app
    │   ├── studio.js
    │   ├── clipControls.js
    │   ├── meshTierToggle.js
    │   ├── meshBadge.js
    │   └── timelineSync.js
    └── showcase/           # Showcase carousel + fullscreen modal
        ├── showcase.js
        ├── showcaseCarousel.js
        ├── showcaseModal.js
        └── showcaseOrder.js
```

## Conventions

- **Pages** at `static/` root (FastAPI `StaticFiles` + `html=True`).
- **Shared 3D** code lives in `js/viewer/` only — no duplication in studio/showcase.
- **Stage variants:** `stageVariant: "dark"` (studio), `"light"` (showcase/modal).
