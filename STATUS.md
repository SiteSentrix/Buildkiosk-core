# Status

_Updated every session; under 40 lines. Longer history is in the ROADMAP progress log._

## Last task
- 2026-11-04: initial core copied from the earlier private workspace (one fresh commit, no history); vendor
  wording removed; `public-guard` CI job; extension traits in `crates/core/src/ext.rs`; agent kit and Claude Code
  hooks. PR "Initial core from buildkiosk" (needs-review).

## In progress
- Nothing.

## Next (from docs/core-vs-enterprise.md and docs/gaps.md)
1. Invite by email with an accept flow and per-project roles (uses `ext::InviteAccept`).
2. Wire the extension traits into the server (`TenantResolver`, `Authenticator`, `MailSender`, `BlobStore`,
   `BackgroundJob`, `FeatureFlags`); the tenant one is the big refactor (~100 call sites).
3. Owner/client portal (reuse the token-portal pattern from the bidder and prequal portals).

## Blockers / owner decisions
- Licence for the public repo is MIT OR Apache-2.0 as inherited; attorney question Q15 is still open.
