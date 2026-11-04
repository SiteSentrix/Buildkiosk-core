# Core and add-ons: what lives where

_Status: 2026-11-04 (tenant resolver wired)._

BuildKiosk comes in two parts:

- **Core (this repository, public, MIT OR Apache-2.0):** the complete single-company product. One company per
  server, hosted by the company itself. Everything a builder or remodeler needs to run their own projects.
- **Add-ons (a separate private repository):** features for running many companies together, mainly as a hosted
  service. They depend on the core's crates and plug in through the extension points below.

Rules:
- Code flows one way: the private add-ons depend on the core, never the reverse. The core never names, imports or
  checks for an add-on.
- A change an add-on needs in the core is made here first, as its own pull request, and must make sense for
  single-company users too (or be a no-op for them).
- Nothing private, no secrets, no customer data. The `public-guard` CI job checks every pull request.

## What the core contains

Every crate of the workspace: `core`, `model`, `engine`, `testkit`, `store`, `web`, `viewer`, `server`, `mcp`,
`app` (the `buildkiosk` binary), plus `xtask` (the quality gate), `install.sh`, `contrib/`, the docs and the
screenshots. The earlier private workspace already was the single-company product, so it was copied whole.

Left out of this repository:
- `docs/hosting.md` (the design for hosting many companies on one server): that is add-on work. The core keeps the
  per-company data layout that design builds on.
- Git history: the core starts with one fresh commit that names the commit it was copied from.

## What belongs to the add-ons

- Hosting many companies on one server (company chosen per request, platform admin, limits, billing).
- One login across companies, single sign-on.
- Company network: a shared directory across companies, connect codes, cross-company project roles.
- Portfolio reports across companies.
- Later, if the owner un-parks them: job-site sign-in and badges, drones, native mobile apps, integrations.

What stays in the core even though an add-on extends it: inviting people by email with an accept flow and
per-project roles (single company), documents shared by link, an owner/client view, offline field use in the
browser.

## Extension points (`crates/core/src/ext.rs`)

Each trait has a default that behaves exactly like the core today, so single-company users see no change.

| Trait | Default | Used for | Wired in yet? |
|---|---|---|---|
| `TenantResolver` | `SingleCompany` (the company the server was started with) | choosing the company per request | Yes: every request is resolved first and handled by that company's own `App` (`crates/server/src/tenants.rs`, `serve_tenants`, `App::open_shared`, `App::with_company_secret`). The handlers still reach their company through `App` (one per company), so the ~100 call sites did not need to change. Timed work runs per company. Scheduled backups still copy the whole data directory |
| `Authenticator` | `LocalOnly` (local argon2 passwords) | sign-in from accounts kept elsewhere | No: `crates/server/src/auth.rs` checks local credentials only |
| `InviteAccept` | `LocalAccounts` (accepting makes a local account) | linking an invited person to an existing account | No: the invite-by-email flow itself is still to build (docs/gaps.md) |
| `MailSender` | none in core (SMTP in `crates/server/src/mail.rs`) | a different mail service per company | No: `mail::send` is called directly |
| `BlobStore` | none in core (files in the company folder, `crates/store`) | object storage as the main store | No: the store writes files directly |
| `BackgroundJob` | none in core | running timed work (mail queue, reminders, recurring forms, backups) for each company | No: the server runs these passes for its one company |
| `FeatureFlags` | `NoFlags` (nothing optional is on) | showing or hiding features a build adds | No |

Other public hooks:
- `buildkiosk_server::route_patterns()`: every route the web server serves (pattern such as
  `/projects/{id}/rfis/{rid}` and its methods), built by the same calls that build the router, so tests outside the
  core can visit every page without reading its source.

Still to add when the add-ons need them: a way to register extra commands (the command registry in
`crates/engine/src/cmd/mod.rs` and the tool list in `crates/model/src/tool.rs` are fixed today), and a way to add
pages and menu items to the web UI.

Wiring a trait is a normal core task: one trait per pull request, default behaviour unchanged, tests showing the
default still works.
