# Account recovery — verified email and replacement account key

> Design contract, no email sent or account changed. Recovery is a prerequisite
> for paid cosmetics, reopening D-17 without replacing stable account identity.

## Enrollment and account continuity

Guests and existing non-paying accounts can continue playing. Registration/
login presents an explanation and recovery-email enrollment; existing members
manage it in authenticated account settings. A **verified recovery email is
required before cash checkout**, not before every match. Do not add a public
email column to the shared profile/leaderboard DTO.

Bind enrollment to the current AccountId and fresh proof of the account key,
not just a possibly stolen lobby/resume session. Store pending address and
verification challenge separately from the active verified address. Only a
valid single-use verification establishes recoverability. Resends/expiry
must not silently replace an already verified address.

Proposed initial lookup is account name plus verified email, allowing the
same family email for multiple accounts without listing them publicly.
Document normalization: trim permitted outer space, normalize domain safely,
preserve intended local-part semantics; never strip dots/plus tags or merge
accounts on a provider-specific guess. Syntax validation is not mailbox
ownership. Unicode/IDN and supported delivery policy are explicit.

## Lost-key workflow

1. User selects Lost account key and supplies account name/recovery email.
2. Respond generically for known/unknown/unverified/rate-limited identities;
   use comparable async timing and bounded account/IP/destination limits.
   A request alone never changes, locks or revokes the account.
3. Send a short-lived, single-use link to the **previously verified** address.
   Server records only token digest, AccountId, purpose, expiry and state;
   token generation uses cryptographic OS randomness.
4. Opening GET shows confirmation only: mail scanners/prefetch must not consume
   a token or rotate a key. An explicit protected POST performs the operation
   after the user confirms. No automatic login or permanent key in email.
5. In one authoritative transaction consume token, replace the key hash,
   increment auth epoch, invalidate other recovery challenges, append safe
   audit/outbox records and preserve the same AccountId/Inventory/rating.
6. Display the newly generated key once on the trusted HTTPS completion page,
   let the user save it, then use normal login. Clear token from navigation;
   no referrer/analytics leakage or third-party assets on the recovery page.
7. Notify the established mailbox of successful rotation without including
   the new key. A lost completion response requires a fresh recovery, never
   retaining plaintext keys for support staff to retrieve later.

Existing key format/codec compatibility must be audited before a key-length
change; P67 does not silently change protocol bounds. Temporary recovery/
verification secrets can be stronger independent tokens. Purpose separation
prevents an email-verification token from authorizing key rotation.

## Changing email, compromised accounts and sessions

Require fresh account-key proof, new-mailbox verification and approval from
the existing trusted mailbox for a routine change; notify both addresses.
Losing old-mailbox access is a separate reviewed recovery/escalation case,
not a bypass that lets a new address overwrite the old one. Newly attached
recovery factors are not sufficient independent proof after suspected theft.
Do not recover from a payment screenshot, CPF or billing email alone.

On actual rotation revoke all old game/community/reconnect sessions and
credentials via authoritative auth epoch/invalidation, including old P66
generations. Do not restore them from a backup. Revocation may disconnect a
compromised session under the security policy; it never rewrites an ongoing
match result or transfers paid items to a new account. Integration tests must
cover P65 HTTP sessions, lobby takeover and P66 generation ownership.

Support/Moderator roles cannot view or generate permanent keys or arbitrarily
change recovery addresses. Any future exceptional recovery action needs a
specific permission, independent proof, recent operator auth and audit.
If both key and verified mailbox are lost, do not promise recovery today.

## Sensitive storage and offline-first base

Emails and verification/recovery material stay private with access control,
appropriate at-rest protection, redacted logs and documented retention. Mail
outbox must not persist raw recovery links unprotected: use a bounded encrypted
delivery payload with a separately managed key, or an equivalent reviewed
short-lived delivery design. Hash-only verification records cannot regenerate
a link after restart. Do not invent home-grown cryptography.

Initial adapter is an in-memory mail capture in synthetic tests only; no
SMTP, DNS changes or email-provider integration is authorized. Production
delivery later needs authenticated sender/domain configuration, bounce/error
handling, safe fixed-origin links and privacy review. Recovery email and
Asaas payer/invoice email are different purposes; neither silently updates
the other. No marketing consent bundled into recovery enrollment.

Proposed defaults: recovery token 15-minute lifetime, verification token
24 hours, three recovery dispatches/hour per account/destination with separate
bounded IP throttling. Confirm at P67 O02; invalid attempts cannot permanently
lock the victim out. Never log tokens, full URLs or plaintext keys.

Reference principles: [OWASP recovery](https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html)
and [email verification](https://cheatsheetseries.owasp.org/cheatsheets/Email_Validation_and_Verification_Cheat_Sheet.html).
The specific account-key transaction and cross-generation invalidation are
TBG design requirements, not an already implemented recovery feature.
