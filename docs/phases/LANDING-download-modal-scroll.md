# Landing download dialog: preserve page position

Owner request: closing either native download dialog must leave the visitor at
Where to play / Choose your platform. Scope is the public landing only.

## Cause and change

The shared `.frame` positioning left the modal's computed position absolute.
Focusing its close button during `showModal()` scrolled the document to the top;
closing the dialog then exposed that new position. The live reproduction measured
document scroll 7741 before opening and 0 during/after the Windows dialog.

Anchor `.native-dialog` to the viewport with `position:fixed; inset:0`. Keep the
native dialog's focus restoration, keyboard handling, backdrop close and bounded
internal scrolling. No JavaScript scroll reset, layout redesign or dependency.

## Work-front validation

In the local desktop browser, both Windows and Linux dialogs were opened using
real pointer actions and closed by button, Escape and backdrop: all six paths
kept scroll **7739 before, during and after**, closed successfully and restored
focus to the correct platform button. The local preview uses the normal safe
pending catalog; live complete-package dialogs are checked after publication.

The first locator-based measurement also moved the button into view before its
click; direct pointer measurements above separate that test setup from dialog
scrolling. No product workaround or weakened assertion was needed.

`git diff --check` and `python3 web/scripts/check.py` passed (120 landing files,
16 public-admin files, 115 local-preview files and generated content checks).
Public verification is recorded when executed. No Rust/Trunk build, native packaging, full suite or
gameplay validation is needed for this CSS-only public-site fix. Other browsers
and phone layouts remain unverified; mobile is outside this request.

## Publication

Publish only a new immutable landing directory and atomically switch its entry.
Preserve the current download catalog, backend process, other site pointers,
proxy configuration, databases and deployment activation flags/timers.
