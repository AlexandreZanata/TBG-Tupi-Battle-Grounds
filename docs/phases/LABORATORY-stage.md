# Laboratory courtyard and desktop usability

Owner request: use the supplied Bone Brute laboratory reference as visual guidance,
build a real procedural 3D stage in the existing arena style, polish the laboratory,
and reinstall the local native Linux client. The reference image stays local.

## Scope

- Preserve the shared header, all champion forms and animation actions, facts,
  exact-team editor, local trial controls and Practice rules.
- Three columns: fixed search/filter header and independently scrolling champion
  list, central real model and compact selected controls, inspector/team/bot pane.
- Match the arena mesh vocabulary: painted bevelled stone, faceted columns,
  stepped circular plinth, blue/gold banners, braziers and distant greenery.
- Reuse the existing rendering pipeline and settings; no new dependency, downloaded
  backdrop, game rule, balance value or server change.
- Search matches both localized and canonical champion names. Empty filters explain
  the result. Selected forms, animation, difficulty and reduced motion stay visible.
- Drag/wheel input follows the actual center viewport rather than fixed percentages.
  Leaving Troops restores the scene, lighting, camera and fog; the stage is removed.

## Validation and delivery

Level: work front, with the owner-requested native release build/install and focused
render observations. All compilation uses the shared host lock with two build jobs.
The owner has explicitly allowed waiting for that lock on requested local deliveries.

Pending: formatting, release compilation, real renderer at desktop sizes, representative
champion framing, tab/input and transition observations, package provenance and installation.
No full suite, whole-roster gallery, balance gate or public backend deployment is requested.
Browser and Windows acceptance remain separate from native Linux evidence.
