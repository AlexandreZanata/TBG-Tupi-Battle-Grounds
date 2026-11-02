# Native update trust anchor

`root.json` is the public TUF root bundled out of band with the managed Linux
bootstrap. Its SHA-256 is
`db40abac2b2b5fbf3f19cc8fb37cbc1bed274f56223a3e4bd20771346df2d141`.
The private role keys are never committed, packaged, or uploaded to the CDN.
Only the owner-controlled publisher can authorize executable updates.

The repository URL is `https://api.tupibg.com/updates/`, using the existing
static-file route. Game HTTP/WebSocket routing and the authoritative backend
are independent. Publish immutable targets first and timestamp metadata last.
Keep old targets/metadata available for running or recovering clients.

The corresponding owner-only signing-key custody and initial channel activation
must be recorded in the native settings delivery report before distribution.
Subsequent releases use `tbg-release publish` with the same authority and a
strictly increasing sequence; never replace the root just to fix a download.
