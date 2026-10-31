# Mandatory patch notes

Every gameplay revision entering main must have an entry in catalog.json and
a Markdown document here. Numeric balance revisions also require accepted
D-33 evidence: exact before/after values and units, rationale, unchanged
thresholds (or an explicit owner decision), discovery/screening/confirmation
seed partitions, sample counts, uncertainty, limitations and rollback.
Values remain Proposed until owner playtesting. Failed or unexecuted checks
are never a pass. Preserve historical entries and ranked revision identities.

scripts/check-patch-notes.py checks the bilingual catalog, evidence/document
links, current source hash and RulesetVersion bump against the PR base. CI
runs it before Rust checks. This requirement does not merge or deploy changes;
the owner reviews CI and merges manually. Preview deployments remain labeled.

Public API: GET /api/patch-notes returns schema 1 and the reviewed notes;
GET /api/patch-notes/{id} returns one entry or 404. No account, adjudication
or private player data is included. Responses may cache for 60 seconds.
Native clients use the same compiled snapshot when the feed is unavailable.
The lobby news card opens the shared patch page in Linux and the browser.

ponytail: the feed is bounded to 50 entries / 256 KiB. Implement cursor
pagination before exceeding this budget; never silently drop old records.

## Publishing a gameplay or balance revision

1. Preserve prior catalog entries. Copy TEMPLATE.md to a dated patch document
   and fill exact before/after values, units, cohort definitions and caveats.
2. For numeric balance, collect discovery and screening evidence, freeze the
   candidate, then use untouched confirmation seeds. A failed required gate
   cannot be marked accepted. Maintenance audits may report rejected candidates
   without changing gameplay numbers or claiming competitive acceptance.
3. Increment RulesetVersion when a watched gameplay source changes and append
   its source hash to the ruleset registry. Protocol changes are independent.
4. Add the bilingual schema-1 catalog entry with that revision/hash and the
   document/report paths. Set kind=balance only with evidence.accepted=true.
5. Run `python3 scripts/test-patch-notes.py` and
   `python3 scripts/check-patch-notes.py --base origin/main`. CI compares the
   actual PR base or previous push revision, rather than trusting a branch name.
6. The server and shared client compile the same catalog. Update them together;
   future catalog changes trigger native release validation. Deploy and merge
   remain explicit owner actions. This task already authorizes its deployment.

API schema 1 contains notes[]. Each entry identifies id, marketing version,
release date/status/kind, RulesetVersion/hash, bilingual title/summary/sections,
concrete changes (subject/field/before/after/reason), document and safe aggregate
evidence. The collection route returns the catalog; detail returns the entry
itself. Invalid/unknown IDs return 404; both routes use public max-age=60.
There are no write methods and no database migrations. Clients retain their
compiled snapshot on network failure or malformed/oversized responses.
