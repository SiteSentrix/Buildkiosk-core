# QA latest: BuildKiosk regression (smoke)

Generated 2026-11-04T04:12:06Z · commit 56d28eea1356 · smoke suite · 1 s

The real `buildkiosk` program, started on a throwaway data folder with the fictional demo company and driven over HTTP like a person (sign in, pages, forms, uploads, downloads). No production data; no secrets printed.

**Gate: PASS.** 41 passed, 0 failed, 0 known (quarantined, filed), 0 skipped.

## New failures since the last run

No earlier report to compare with.

## By area

| Area | Pass | Fail | Known | Skip |
|---|---|---|---|---|
| access | 4 | 0 | 0 | 0 |
| boot | 5 | 0 | 0 | 0 |
| crawl | 6 | 0 | 0 | 0 |
| loops | 3 | 0 | 0 | 0 |
| projects | 1 | 0 | 0 | 0 |
| routes | 6 | 0 | 0 | 0 |
| safety | 7 | 0 | 0 | 0 |
| sign-in | 8 | 0 | 0 | 0 |
| suite | 1 | 0 | 0 | 0 |

## Slowest checks

| Check | Time |
|---|---|
| boot › the demo company is created from the command line | 0.2 s |
| boot › the server starts on a free port and answers | 0.1 s |
| routes › admin: every page in the route table answers without a server error | 0.1 s |
| routes › project manager: every page in the route table answers without a server error | 0.1 s |
| routes › project engineer: every page in the route table answers without a server error | 0.1 s |
| routes › superintendent: every page in the route table answers without a server error | 0.1 s |
| routes › subcontractor: every page in the route table answers without a server error | 0.1 s |
| routes › owner and design team: every page in the route table answers without a server error | 0.1 s |

## Slowest requests (of 2681)

| Request | Status | Time |
|---|---|---|
| POST /login (admin) | 303 | 0.0 s |
| POST /login (project manager) | 303 | 0.0 s |
| POST /login (superintendent) | 303 | 0.0 s |
| POST /login (project engineer) | 303 | 0.0 s |
| POST /login (subcontractor) | 303 | 0.0 s |
| POST /login (owner and design team) | 303 | 0.0 s |
| POST /login (anonymous) | 303 | 0.0 s |
| POST /login (anonymous) | 401 | 0.0 s |

## Every check

| Area | Check | Tag | Result | Time | Detail |
|---|---|---|---|---|---|
| suite | the page reader understands forms and links | @smoke | pass | 0.0 s | forms, selects, checkboxes, buttons and links |
| boot | the demo company is created from the command line | @smoke | pass | 0.2 s | Demo company created in /tmp/buildkiosk-regress-3329/data. |
| boot | an API token is made for every role from the command line | @smoke | pass | 0.0 s | 6 tokens |
| boot | the server starts on a free port and answers | @smoke | pass | 0.1 s | 127.0.0.1:45849 |
| boot | the sign-in page answers | @smoke | pass | 0.0 s | 200 |
| sign-in | admin signs in | @smoke | pass | 0.0 s | ada@example.com |
| sign-in | project manager signs in | @smoke | pass | 0.0 s | priya.raman@example.com |
| sign-in | superintendent signs in | @smoke | pass | 0.0 s | marcus.lee@example.com |
| sign-in | project engineer signs in | @smoke | pass | 0.0 s | elena.cruz@example.com |
| sign-in | subcontractor signs in | @smoke | pass | 0.0 s | tom.okafor@example.com |
| sign-in | owner and design team signs in | @smoke | pass | 0.0 s | hana.berg@example.com |
| sign-in | session and form cookies are HttpOnly and SameSite | @smoke | pass | 0.0 s | HttpOnly, SameSite, Path=/ |
| sign-in | a wrong password is refused | @smoke | pass | 0.0 s | 401 |
| safety | a sign-in without the form token is refused (CSRF) | @smoke | pass | 0.0 s | 403 |
| projects | the demo projects are listed on the home page | @smoke | pass | 0.0 s | projects [2, 1] |
| routes | admin: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×176, 303×5, 404×53 |
| routes | project manager: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×174, 303×7, 404×53 |
| routes | superintendent: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | project engineer: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | subcontractor: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×166, 303×7, 404×61 |
| routes | owner and design team: every page in the route table answers without a server error | @smoke | pass | 0.1 s | 234 routes: 200×169, 303×7, 404×58 |
| access | anonymous: every private page sends the visitor to sign in | @smoke | pass | 0.0 s | 221 private pages |
| access | subcontractor: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | owner and design team: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | project engineer: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| crawl | admin: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| crawl | project manager: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| crawl | superintendent: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| crawl | project engineer: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| crawl | subcontractor: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| crawl | owner and design team: every linked page answers without a server error | @smoke | pass | 0.0 s | 150 pages: 200×150 |
| loops | RFI: create, edit, attach, respond, close (journal checked each step) | @smoke | pass | 0.0 s | RFI 7: rfi.create → rfi.update → rfi.attach → rfi.respond → rfi.close |
| loops | punch list: create, edit, attach, finish (journal checked each step) | @smoke | pass | 0.0 s | item 7: punch.create → punch.update → punch.attach → punch.complete |
| loops | daily log: create, edit, attach, submit (journal checked each step) | @smoke | pass | 0.0 s | log 3: daily_log.create → daily_log.update → daily_log.attach → daily_log.submit |
| safety | every form post without its CSRF token is refused and changes nothing | @smoke | pass | 0.0 s | 80 post routes refused |
| safety | every answer carries the security headers | @smoke | pass | 0.0 s | 2681 answers checked |
| safety | no answer shows panic text, a stack trace, a source location or a secret | @smoke | pass | 0.0 s | 2681 answers checked |
| safety | the journal holds no password, token or session cookie | @smoke | pass | 0.0 s | 134 lines |
| boot | the server was still running at the end of the run | @smoke | pass | 0.0 s | yes |
| safety | no panics in the server log (first run) | @smoke | pass | 0.0 s | 1 log lines |
| safety | the server log shows no password, token or session cookie (first run) | @smoke | pass | 0.0 s | clean |

Known-failing checks live in `qa/known-skips.json` (reason, issue, expiry of at most two weeks); see `docs/qa.md`.
