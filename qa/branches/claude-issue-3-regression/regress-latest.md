# QA latest: BuildKiosk regression (release)

Generated 2026-11-04T02:47:45Z · commit ceb253d7cb8a · release suite · 4 s

The real `buildkiosk` program, started on a throwaway data folder with the fictional demo company and driven over HTTP like a person (sign in, pages, forms, uploads, downloads). No production data; no secrets printed.

**Gate: FAIL.** 73 passed, 12 failed, 0 known (quarantined, filed), 1 skipped.

## What fails the gate

- FAIL access › anonymous: every private page sends the visitor to sign in
- FAIL access › refused and missing pages answer 403 or 404, not 200
- FAIL crawl › admin: every link shown opens (no 4xx, no "not found" page)
- FAIL crawl › subcontractor: every link shown opens (no 4xx, no "not found" page)
- FAIL crawl › owner and design team: every link shown opens (no 4xx, no "not found" page)
- FAIL loops › RFI: create, edit, attach, respond, close (journal checked each step)
- FAIL loops › punch list: create, edit, attach, finish (journal checked each step)
- FAIL loops › daily log: create, edit, attach, submit (journal checked each step)
- FAIL tools › inspections: create, open, edit, attach
- FAIL exports › every CSV export downloads as CSV
- FAIL sign-in › signing out ends the session (the old cookie no longer works)
- FAIL data › what the run made is still there after the restart

## New failures since the last run

No earlier report to compare with.

## By area

| Area | Pass | Fail | Known | Skip |
|---|---|---|---|---|
| access | 4 | 2 | 0 | 0 |
| api | 3 | 0 | 0 | 0 |
| boot | 6 | 0 | 0 | 0 |
| crawl | 9 | 3 | 0 | 0 |
| data | 2 | 1 | 0 | 0 |
| exports | 1 | 1 | 0 | 0 |
| imports | 2 | 0 | 0 | 0 |
| loops | 0 | 3 | 0 | 0 |
| projects | 1 | 0 | 0 | 0 |
| routes | 7 | 0 | 0 | 0 |
| safety | 10 | 0 | 0 | 0 |
| sign-in | 10 | 1 | 0 | 0 |
| suite | 1 | 0 | 0 | 0 |
| tools | 15 | 1 | 0 | 0 |
| upgrade | 0 | 0 | 0 | 1 |
| uploads | 2 | 0 | 0 | 0 |

## Slowest checks

| Check | Time |
|---|---|
| routes › admin: made-up and malformed ids never cause a server error | 0.7 s |
| access › anonymous: every private page sends the visitor to sign in | 0.2 s |
| routes › project manager: every page in the route table answers without a server error | 0.2 s |
| routes › superintendent: every page in the route table answers without a server error | 0.2 s |
| routes › admin: every page in the route table answers without a server error | 0.2 s |
| routes › project engineer: every page in the route table answers without a server error | 0.2 s |
| boot › the demo company is created from the command line | 0.2 s |
| routes › subcontractor: every page in the route table answers without a server error | 0.2 s |

## Slowest requests (of 5112)

| Request | Status | Time |
|---|---|---|
| POST /projects/2/rfis/7/upload (project manager) | 303 | 0.0 s |
| POST /login (admin) | 303 | 0.0 s |
| POST /login (admin (after restart)) | 303 | 0.0 s |
| POST /login (project manager) | 303 | 0.0 s |
| POST /login (anonymous) | 401 | 0.0 s |
| POST /login (anonymous) | 401 | 0.0 s |
| POST /login (superintendent) | 303 | 0.0 s |
| POST /login (project engineer) | 303 | 0.0 s |

## Every check

| Area | Check | Tag | Result | Time | Detail |
|---|---|---|---|---|---|
| suite | the page reader understands forms and links | @smoke | pass | 0.0 s | forms, selects, checkboxes, buttons and links |
| boot | the demo company is created from the command line | @smoke | pass | 0.2 s | Demo company created in /tmp/buildkiosk-regress-7734/data. |
| boot | an API token is made for every role from the command line | @smoke | pass | 0.0 s | 6 tokens |
| boot | the server starts on a free port and answers | @smoke | pass | 0.1 s | 127.0.0.1:42093 |
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
| routes | admin: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×176, 303×5, 404×53 |
| routes | project manager: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×174, 303×7, 404×53 |
| routes | superintendent: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | project engineer: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | subcontractor: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×166, 303×7, 404×61 |
| routes | owner and design team: every page in the route table answers without a server error | @smoke | pass | 0.2 s | 234 routes: 200×169, 303×7, 404×58 |
| routes | admin: made-up and malformed ids never cause a server error | @release | pass | 0.7 s | no server errors |
| access | anonymous: every private page sends the visitor to sign in | @smoke | fail | 0.2 s | /projects/2/estimating/compare → 303 /projects/2/estimating |
| access | subcontractor: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | owner and design team: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | project engineer: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | refused and missing pages answer 403 or 404, not 200 | @release | fail | 0.0 s | /settings → 200; /admin/backups → 303; /admin/email → 303; /projects/2/rfis/99999999 → 200 |
| crawl | admin: every linked page answers without a server error | @smoke | pass | 0.0 s | 491 pages: 200×489, 303×2 |
| crawl | admin: every link shown opens (no 4xx, no "not found" page) | @release | fail | 0.0 s | 1 broken: /prequalification/2 shows "not found" (linked from /projects/2/commitments) |
| crawl | project manager: every linked page answers without a server error | @smoke | pass | 0.0 s | 469 pages: 200×467, 303×2 |
| crawl | project manager: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 469 pages |
| crawl | superintendent: every linked page answers without a server error | @smoke | pass | 0.0 s | 399 pages: 200×399 |
| crawl | superintendent: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 399 pages |
| crawl | project engineer: every linked page answers without a server error | @smoke | pass | 0.0 s | 389 pages: 200×387, 303×2 |
| crawl | project engineer: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 389 pages |
| crawl | subcontractor: every linked page answers without a server error | @smoke | pass | 0.0 s | 280 pages: 200×280 |
| crawl | subcontractor: every link shown opens (no 4xx, no "not found" page) | @release | fail | 0.0 s | 6 broken: /projects/2/schedule/12 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/17 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/13 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/14 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/1/schedule/3 shows "not foun… |
| crawl | owner and design team: every linked page answers without a server error | @smoke | pass | 0.0 s | 310 pages: 200×310 |
| crawl | owner and design team: every link shown opens (no 4xx, no "not found" page) | @release | fail | 0.0 s | 6 broken: /projects/2/schedule/12 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/17 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/13 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/14 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/1/schedule/3 shows "not foun… |
| loops | RFI: create, edit, attach, respond, close (journal checked each step) | @smoke | fail | 0.0 s | POST /do/rfi.create refused: assign someone to answer before opening the RFI |
| loops | punch list: create, edit, attach, finish (journal checked each step) | @smoke | fail | 0.0 s | no `punch.` in the journal (new entries: none) |
| loops | daily log: create, edit, attach, submit (journal checked each step) | @smoke | fail | 0.0 s | no `daily_log.update` in the journal (new entries: none) |
| tools | submittals: create, open, edit, attach | @release | pass | 0.0 s | submittals 5: submittal.create → edit (no change) → submittal.attach |
| tools | observations: create, open, edit, attach | @release | pass | 0.0 s | observations 5: observation.create → edit (no change) → observation.attach |
| tools | tasks: create, open, edit, attach | @release | pass | 0.0 s | tasks 1: task.create → edit (no change) → task.attach |
| tools | incidents: create, open, edit, attach | @release | pass | 0.0 s | incidents 1: incident.create → edit (no change) → incident.attach |
| tools | meetings: create, open, edit, attach | @release | pass | 0.0 s | meetings 3: meeting.create → edit → meeting.attach |
| tools | transmittals: create, open, edit, attach | @release | pass | 0.0 s | transmittals 3: transmittal.create → edit → transmittal.attach |
| tools | correspondence: create, open, edit, attach | @release | pass | 0.0 s | correspondence 1: correspondence.create → edit → correspondence.attach |
| tools | action-plans: create, open, edit, attach | @release | pass | 0.0 s | action-plans 1: action_plan.create → edit → action_plan.attach |
| tools | inspections: create, open, edit, attach | @release | fail | 0.0 s | POST /do/inspection.create refused: invalid parameters for `inspection.create`: `items` is required |
| tools | commitments: create, open, edit, attach | @release | pass | 0.0 s | commitments 5: commitment.create → edit (no change) → commitment.attach |
| tools | change-events: create, open, edit, attach | @release | pass | 0.0 s | change-events 1: change_event.create → edit (no change) |
| tools | prime-contracts: create, open, edit, attach | @release | pass | 0.0 s | prime-contracts 1: prime_contract.create → edit (no change) |
| tools | bidding: create, open, edit, attach | @release | pass | 0.0 s | bidding 1: bid_package.create → edit → bid_package.attach |
| tools | estimating: create, open, edit, attach | @release | pass | 0.0 s | estimating 1: estimate.create → edit |
| tools | drawings: upload a sheet, open it and its viewer page | @release | pass | 0.0 s | sheet 9; view.png 404 |
| tools | schedule: add a task | @release | pass | 0.0 s | schedule.add |
| exports | every CSV export downloads as CSV | @release | fail | 0.0 s | /projects/2/forms/report.csv → 404 text/plain; charset=utf-8 |
| exports | a spreadsheet formula typed into an RFI is defused in the CSV | @release | pass | 0.0 s | written as '=1+2… |
| imports | schedule exchange file: export from one project, import into the other | @release | pass | 0.0 s | 4359 bytes round-tripped |
| imports | budget: export the CSV, import it into the other project | @release | pass | 0.0 s | budget.import |
| uploads | a file over 25 MB is refused without a server error | @release | pass | 0.1 s | answered 303 The upload failed or the file is larger than 25 MB. |
| uploads | a photo upload must be an image | @release | pass | 0.0 s | POST /projects/2/photos/upload refused: Photos must be JPEG, PNG, GIF or WebP pictures. |
| safety | an uploaded web page is served as a download, never as a live page | @release | pass | 0.0 s | attachment + nosniff |
| api | a command with an API token works; without one it is refused | @release | pass | 0.0 s | 200 with, 401 without |
| api | a subcontractor's token cannot add people to the company | @release | pass | 0.0 s | 403 |
| api | MCP over HTTP answers initialize with a token | @release | pass | 0.0 s | initialize ok |
| sign-in | repeated wrong passwords are slowed down | @release | pass | 0.1 s | 429 after repeated failures |
| sign-in | signing out ends the session (the old cookie no longer works) | @release | fail | 0.0 s | the session cookie from before signing out still opens the home page |
| access | a record opened under another project is not found | @release | pass | 0.0 s | RFI 7 under project 1: not found |
| safety | every form post without its CSRF token is refused and changes nothing | @smoke | pass | 0.1 s | 80 post routes refused |
| safety | every answer carries the security headers | @smoke | pass | 0.0 s | 5109 answers checked |
| safety | no answer shows panic text, a stack trace, a source location or a secret | @smoke | pass | 0.0 s | 5109 answers checked |
| safety | the journal holds no password, token or session cookie | @smoke | pass | 0.0 s | 158 lines |
| boot | the server was still running at the end of the run | @smoke | pass | 0.0 s | yes |
| safety | no panics in the server log (first run) | @smoke | pass | 0.0 s | 1 log lines |
| safety | the server log shows no password, token or session cookie (first run) | @smoke | pass | 0.0 s | clean |
| data | `buildkiosk check` replays every journal after the run | @release | pass | 0.0 s | summit-ridge: ok, 158 journal entries; company 1, people 6, vendors 3, templates 6, projects 2, members 12, cost_codes 75, rfis 7, submittals 5, transmittals 3, correspondence 1, meetings 3, tasks 1, spec_sections 6, punch_items 7, inspection_templates 8, inspections 2, observations 5, action_plans 1, incidents 1, daily_logs 3, schedule_tasks 29, budget_lines 12, commitments 5, direct_costs 6, cha… |
| data | the server starts again on the same data (behind HTTPS: --secure-cookies) | @release | pass | 0.1 s | started |
| sign-in | behind HTTPS the cookies are Secure and HSTS is sent | @release | pass | 0.0 s | Secure + HSTS |
| data | what the run made is still there after the restart | @release | fail | 0.0 s | RFI list answered 200 |
| boot | the restarted server was still running at the end | @release | pass | 0.0 s | yes |
| safety | no panics in the server log (after the restart) | @smoke | pass | 0.0 s | 1 log lines |
| safety | the server log shows no password, token or session cookie (after the restart) | @smoke | pass | 0.0 s | clean |
| upgrade | the previous release's data folder opens with this build | @release | skip | 0.0 s | no previous release program (BK_REGRESS_PREV_BIN) (known until 2026-11-17: no published release yet to upgrade from; the first release tag makes this check run (regress.yml downloads the previous release's Linux build)) |

Known-failing checks live in `qa/known-skips.json` (reason, issue, expiry of at most two weeks); see `docs/qa.md`.
