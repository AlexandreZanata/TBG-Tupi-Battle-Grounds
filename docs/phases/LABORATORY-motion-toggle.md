# Laboratory reduced-motion toggle correction

Owner clarified that the frozen previews came from the reduced-motion button being
active on entry, not from broken champion clips. Scope is limited to that control.

Cause: `Catalog::default` inherited the saved Result/accessibility reduced-motion
preference, and the laboratory toggle wrote back to that same global preference.
This made a temporary preview setting persist and freeze subsequent visits. The
button label did not explain whether reduction was active.

Change: laboratory playback defaults to normal, the toggle stays local to the visit,
leaving Troops clears it, and localized text explicitly labels on/off. Existing
animations and the static reduced pose are preserved. The shared desktop visual
contract and motion matrix are updated in this same change.

Validation level: work front with owner-authorized native build and reinstall.
Pending: affected-package formatting, source routing/locale checks, Linux release
build, motion-toggle observations with an old global preference of 1, package and
installation verification. Windows/web motion acceptance is pending consolidation;
no full suite or whole-roster gallery is requested. The latest AGENTS.md forbids
pushing, so this adjustment is committed locally only.
