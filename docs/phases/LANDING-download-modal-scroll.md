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
No Rust/Trunk build, native packaging, full suite or gameplay validation is
needed for this CSS-only public-site fix. Other browsers
and phone layouts remain unverified; mobile is outside this request.

## Publication

Published source `cb68ad4` as immutable landing release
`20261102T002642Z-cb68ad4e34a9` using the existing static staging/upload helper.
The upload helper repeated its required static checks successfully. Activation
ran under the existing nonblocking VPS deployment lock after matching the staged
HTML/CSS digests. The previous entry and protected state were backed up; public
HTML returned 200 with the exact staged digest.

On the actual public site with both complete 0.1.2-rc.1 packages available, all
six pointer/close paths passed again: scroll **8029 before, during and after**,
correct opener focus, closed dialog and no browser-console errors. No page jump
occurred on opening or closing. Proof screenshots/measurements are outside Git.

Before/after checks confirmed the same backend PID 5283, kernel executable,
Caddy configuration, public download-catalog digest and game/admin site pointers.
All four deployment timers/flags remained disabled/absent. No database mutation,
proxy reload, backend restart or native application change was involved. The old
landing release remains available for rollback. No CI completion was awaited.
