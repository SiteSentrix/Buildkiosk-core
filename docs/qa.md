# QA: the regression suite

_Status: 2026-11-04._

The QA bot's black-box suite (`crates/app/tests/regress/`) starts the real `buildkiosk` program the way a person
would run it and drives it over HTTP. There is no staging server: the suite makes its own throwaway data folder with
the fictional demo company (`buildkiosk demo`), makes an API token per role (`buildkiosk token`), starts
`buildkiosk serve` on a free local port, and signs in as each demo person. It uses only the standard library (a small
HTTP client with a cookie jar) and reads pages and forms as a browser would, filling in what a person would type. No
browser and no JavaScript are involved.

## Running it

```sh
BK_REGRESS=smoke cargo test -p buildkiosk --test regress     # the fast checks
BK_REGRESS=release cargo test -p buildkiosk --test regress   # everything
```

Without `BK_REGRESS` the test does nothing, so `cargo xtask ci` is not slowed down. The report is written to
`target/regress/regress-latest.md` and `.json` (`BK_REGRESS_OUT` changes the folder). `BK_REGRESS_BIN` tests another
build of the program (for example a release build); `BK_REGRESS_PREV_BIN` names the previous release's program for
the upgrade check; `BK_REGRESS_PREVIOUS` names the last run's JSON, for the "new failures" list.

## What it checks

Every check is tagged. `@smoke` runs on every pull request (aim: under five minutes); `@release` adds the rest and
runs nightly and for a release tag.

| Area | `@smoke` | `@release` adds |
|---|---|---|
| boot | the demo company, API tokens, the server starts and answers, still running at the end | the restarted server still running |
| sign-in | each of the six demo roles signs in; cookie flags; a wrong password is refused | repeated failures are slowed down; signing out ends the session; behind HTTPS cookies are `Secure` and HSTS is sent |
| routes | every page in the server's route table, for every role, answers without a server error | made-up and malformed ids |
| crawl | every link each role is shown (from the home page, capped) answers without a server error | a larger crawl; every link shown opens (no 4xx, no "not found" page) |
| access | anonymous visitors are sent to sign in; other roles can't open company admin pages; each role's tool pages open or are refused exactly as its permission template says | refused pages answer 403/404; a record opened under another project is not found |
| loops | RFI, punch list, daily log: create, edit, attach, comment or respond, close or submit, with the journal checked after each step | every other tool: create, open, edit, attach; comments on punch items and observations; submittal sent for review, approved by the reviewer and closed; change orders (commitment and prime) approved; drawing upload and the viewer page; schedule task |
| imports, exports | | every CSV export; formulas defused in CSV; schedule XML, `.xer` exchange file (built from one project's schedule export) and budget CSV round trips between projects; CSV imports for punch, observations and submittals found again in their exports |
| uploads | | every signed-in upload route (attachments, responses, reviews, bulk uploads, imports) refuses a file just over its limit without a change; every upload route refuses a web page or serves it back only as a download; photos must be images |
| api | | a token works and no token is refused; a subcontractor's token can't add people; MCP `initialize` |
| safety | every post without its CSRF token is refused and changes nothing; security headers on every answer; no panic text, stack trace, source location or secret in any answer; nothing secret in the journal or the server log; no panics in the server log | |
| data, upgrade | | `buildkiosk check` replays the journals; data is still there after a restart; the previous release's data folder opens with this build |

The route table is read from the server's source (`.route(…)` in `crates/server/src/`), plus the routes the generic
tools register with `format!` (listed in `stories.rs`), so new pages are covered without editing the suite.

## Gates and reports

- `.github/workflows/regress.yml`: smoke on every pull request (label one `regress-release` for the full suite),
  release nightly at 03:10 UTC and by hand. `release.yml` calls it for a version tag and publishes nothing when it
  fails.
- The report (`regress-latest.md`: pass, fail, known and skip per area, what fails the gate, new failures since the
  last run on main, slowest checks and requests, every check) goes to the job summary, a workflow artifact, and the
  `qa-reports` branch, never main: main's runs under `qa/`, other branches' under `qa/branches/<branch>/`, with
  `regress-smoke.*` and `regress-release.*` kept side by side.

## Known failures and skips

A check that fails or skips fails the run unless `qa/known-skips.json` lists it: the check's id (`area › name`, as in
the report), a reason, the issue filed for it, and an expiry date at most two weeks away. `"quarantine": true` lets a
failing check report as `known` until it expires. An expired entry, one more than two weeks out, or a listed check
that now passes (reported, so the entry can go) keeps the list honest. Never weaken a check to get green: file the
bug (labels `bug`, a severity `S1`–`S3`, `claude-queue`) and list the check with the issue number.

## Adding a check

Add it to `crates/app/tests/regress/stories.rs` with `run.check(area, name, Tag::Smoke | Tag::Release, || …)`:
`Ok(detail)` passes, `Err(detail)` fails, and a panic inside a check fails only that check. Use `submit` (open a page,
fill a form like a person, expect the "done" redirect), `press` (a one-button form) and `journaled` (the journal got
the right command, made by a person) for tool loops.

## The tools as a library

The report and gate, the known-skips rules, the HTTP client, the page reader and the server launcher live in the
`buildkiosk-regress-kit` crate (`crates/regress-kit`, `publish = false`); the stories stay in
`crates/app/tests/regress/`. Another suite built on the core can depend on it (as a git dependency) to write the same
report and follow the same skip rules: `report::Run::new(suite)`, `run.check(…)`, `report::load_skips(path)`,
`report::finish(&run, &inputs)`, and `server::Server::start(bin, data, extra, log)` for its own program.
