# QA latest: BuildKiosk regression (release)

Generated 2026-11-04T03:27:16Z · commit 116da82ae63a · release suite · 2 s

The real `buildkiosk` program, started on a throwaway data folder with the fictional demo company and driven over HTTP like a person (sign in, pages, forms, uploads, downloads). No production data; no secrets printed.

**Gate: FAIL.** 95 passed, 1 failed, 6 known (quarantined, filed), 1 skipped.

## What fails the gate

- FAIL loops › prime-change-orders: create, approve (journal checked)

## New failures since the last run

No earlier report to compare with.

## By area

| Area | Pass | Fail | Known | Skip |
|---|---|---|---|---|
| access | 10 | 0 | 2 | 0 |
| api | 3 | 0 | 0 | 0 |
| boot | 6 | 0 | 0 | 0 |
| crawl | 9 | 0 | 3 | 0 |
| data | 3 | 0 | 0 | 0 |
| exports | 2 | 0 | 0 | 0 |
| imports | 6 | 0 | 0 | 0 |
| loops | 7 | 1 | 0 | 0 |
| projects | 1 | 0 | 0 | 0 |
| routes | 7 | 0 | 0 | 0 |
| safety | 10 | 0 | 0 | 0 |
| sign-in | 10 | 0 | 1 | 0 |
| suite | 1 | 0 | 0 | 0 |
| tools | 16 | 0 | 0 | 0 |
| upgrade | 0 | 0 | 0 | 1 |
| uploads | 4 | 0 | 0 | 0 |

## Slowest checks

| Check | Time |
|---|---|
| uploads › every upload route refuses a file over its size limit without a server error or a change | 1.0 s |
| sign-in › repeated wrong passwords are slowed down | 0.1 s |
| routes › admin: made-up and malformed ids never cause a server error | 0.1 s |
| boot › the server starts on a free port and answers | 0.1 s |
| data › the server starts again on the same data (behind HTTPS: --secure-cookies) | 0.1 s |
| boot › the demo company is created from the command line | 0.1 s |
| uploads › a file over 25 MB is refused without a server error | 0.1 s |
| routes › admin: every page in the route table answers without a server error | 0.0 s |

## Slowest requests (of 5481)

| Request | Status | Time |
|---|---|---|
| POST /projects/2/rfis/8/upload (project manager) | 303 | 0.0 s |
| POST /login (admin (after restart)) | 303 | 0.0 s |
| POST /login (admin) | 303 | 0.0 s |
| POST /projects/2/action-plans/1/upload (admin) | 303 | 0.0 s |
| POST /projects/2/drawings/9/upload (admin) | 303 | 0.0 s |
| POST /login (project manager) | 303 | 0.0 s |
| POST /projects/2/bidding/1/upload (admin) | 303 | 0.0 s |
| POST /login (anonymous) | 401 | 0.0 s |

## Every check

| Area | Check | Tag | Result | Time | Detail |
|---|---|---|---|---|---|
| suite | the page reader understands forms and links | @smoke | pass | 0.0 s | forms, selects, checkboxes, buttons and links |
| boot | the demo company is created from the command line | @smoke | pass | 0.1 s | Demo company created in /tmp/buildkiosk-regress-7592/data. |
| boot | an API token is made for every role from the command line | @smoke | pass | 0.0 s | 6 tokens |
| boot | the server starts on a free port and answers | @smoke | pass | 0.1 s | 127.0.0.1:38081 |
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
| routes | admin: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×176, 303×5, 404×53 |
| routes | project manager: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×174, 303×7, 404×53 |
| routes | superintendent: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | project engineer: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×172, 303×7, 404×55 |
| routes | subcontractor: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×166, 303×7, 404×61 |
| routes | owner and design team: every page in the route table answers without a server error | @smoke | pass | 0.0 s | 234 routes: 200×169, 303×7, 404×58 |
| routes | admin: made-up and malformed ids never cause a server error | @release | pass | 0.1 s | no server errors |
| access | anonymous: every private page sends the visitor to sign in | @smoke | pass | 0.0 s | 221 private pages |
| access | subcontractor: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | owner and design team: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | project engineer: company admin pages are refused | @smoke | pass | 0.0 s | /settings, /admin/backups, /admin/email |
| access | refused and missing pages answer 403, 404 or a redirect, not 200 | @release | known | 0.0 s | /settings → 200; /projects/2/rfis/99999999 → 200 (known: #22, until 2026-11-17: FAILS (real finding): the "not found" page is served with status 200 (/settings for the subcontractor, a made-up RFI id)) |
| crawl | admin: every linked page answers without a server error | @smoke | pass | 0.0 s | 491 pages: 200×489, 303×2 |
| crawl | admin: every link shown opens (no 4xx, no "not found" page) | @release | known | 0.0 s | 1 broken: /prequalification/2 shows "not found" (linked from /projects/2/commitments) (known: #24, until 2026-11-17: FAILS (real finding): the commitments list links to /prequalification/{vendor} for vendors without a prequalification) |
| crawl | project manager: every linked page answers without a server error | @smoke | pass | 0.0 s | 469 pages: 200×467, 303×2 |
| crawl | project manager: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 469 pages |
| crawl | superintendent: every linked page answers without a server error | @smoke | pass | 0.0 s | 399 pages: 200×399 |
| crawl | superintendent: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 399 pages |
| crawl | project engineer: every linked page answers without a server error | @smoke | pass | 0.0 s | 389 pages: 200×387, 303×2 |
| crawl | project engineer: every link shown opens (no 4xx, no "not found" page) | @release | pass | 0.0 s | 389 pages |
| crawl | subcontractor: every linked page answers without a server error | @smoke | pass | 0.0 s | 280 pages: 200×280 |
| crawl | subcontractor: every link shown opens (no 4xx, no "not found" page) | @release | known | 0.0 s | 6 broken: /projects/2/schedule/12 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/17 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/13 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/14 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/1/schedule/3 shows "not foun… |
| crawl | owner and design team: every linked page answers without a server error | @smoke | pass | 0.0 s | 310 pages: 200×310 |
| crawl | owner and design team: every link shown opens (no 4xx, no "not found" page) | @release | known | 0.0 s | 6 broken: /projects/2/schedule/12 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/17 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/13 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/2/schedule/14 shows "not found" (linked from /projects/2/schedule/lookahead); /projects/1/schedule/3 shows "not foun… |
| loops | RFI: create, edit, attach, respond, close (journal checked each step) | @smoke | pass | 0.0 s | RFI 7: rfi.create → rfi.update → rfi.attach → rfi.respond → rfi.close |
| loops | punch list: create, edit, attach, finish (journal checked each step) | @smoke | pass | 0.0 s | item 7: punch.create → punch.update → punch.attach → punch.complete |
| loops | daily log: create, edit, attach, submit (journal checked each step) | @smoke | pass | 0.0 s | log 3: daily_log.create → daily_log.update → daily_log.attach → daily_log.submit |
| access | admin: the tools its permission template allows open, the others are refused | @smoke | pass | 0.0 s | 29 tools open, 0 refused |
| access | project manager: the tools its permission template allows open, the others are refused | @smoke | pass | 0.0 s | 29 tools open, 0 refused |
| access | superintendent: the tools its permission template allows open, the others are refused | @smoke | pass | 0.0 s | 29 tools open, 0 refused |
| access | project engineer: the tools its permission template allows open, the others are refused | @smoke | pass | 0.0 s | 29 tools open, 0 refused |
| access | subcontractor: the tools its permission template allows open, the others are refused | @smoke | known | 0.0 s | timesheets is refused (200) with level read (known: #30, until 2026-11-17: FAILS (real finding): the template gives Timesheets Read, but the timesheets page needs Standard and says not found) |
| access | owner and design team: the tools its permission template allows open, the others are refused | @smoke | pass | 0.0 s | 21 tools open, 8 refused |
| tools | submittals: create, open, edit, attach | @release | pass | 0.0 s | submittals 5: submittal.create → edit (no change) → submittal.attach |
| tools | observations: create, open, edit, attach | @release | pass | 0.0 s | observations 5: observation.create → edit (no change) → observation.attach |
| tools | tasks: create, open, edit, attach | @release | pass | 0.0 s | tasks 1: task.create → edit (no change) → task.attach |
| tools | incidents: create, open, edit, attach | @release | pass | 0.0 s | incidents 1: incident.create → edit (no change) → incident.attach |
| tools | meetings: create, open, edit, attach | @release | pass | 0.0 s | meetings 3: meeting.create → edit → meeting.attach |
| tools | transmittals: create, open, edit, attach | @release | pass | 0.0 s | transmittals 3: transmittal.create → edit → transmittal.attach |
| tools | correspondence: create, open, edit, attach | @release | pass | 0.0 s | correspondence 1: correspondence.create → edit → correspondence.attach |
| tools | action-plans: create, open, edit, attach | @release | pass | 0.0 s | action-plans 1: action_plan.create → edit → action_plan.attach |
| tools | inspections: create, open, edit, attach | @release | pass | 0.0 s | inspections 3: inspection.create → edit (no change) → inspection.attach |
| tools | commitments: create, open, edit, attach | @release | pass | 0.0 s | commitments 5: commitment.create → edit (no change) → commitment.attach |
| tools | change-events: create, open, edit, attach | @release | pass | 0.0 s | change-events 1: change_event.create → edit (no change) |
| tools | prime-contracts: create, open, edit, attach | @release | pass | 0.0 s | prime-contracts 1: prime_contract.create → edit (no change) |
| tools | bidding: create, open, edit, attach | @release | pass | 0.0 s | bidding 1: bid_package.create → edit → bid_package.attach |
| tools | estimating: create, open, edit, attach | @release | pass | 0.0 s | estimating 1: estimate.create → edit |
| tools | drawings: upload a sheet, open it and its viewer page | @release | pass | 0.0 s | sheet 9; view.png 404 |
| tools | schedule: add a task | @release | pass | 0.0 s | schedule.add |
| loops | punch-list: comment on an item (journal checked) | @release | pass | 0.0 s | punch 8: create → comment, shown on the page |
| loops | observations: comment on an item (journal checked) | @release | pass | 0.0 s | observation 6: create → comment, shown on the page |
| loops | submittal: create, send for review, reviewer approves, close (journal checked) | @release | pass | 0.0 s | submittal 6: create+open → review (approved) → close |
| loops | change-orders: create, approve (journal checked) | @release | pass | 0.0 s | change-orders 1: create → commitment_change.approve |
| loops | prime-change-orders: create, approve (journal checked) | @release | fail | 0.0 s | POST /projects/2/prime-change-orders/save refused: invalid parameters for `prime_change.create`: `contract` is required |
| exports | every CSV export downloads as CSV | @release | pass | 0.0 s | 17 exports |
| exports | a spreadsheet formula typed into an RFI is defused in the CSV | @release | pass | 0.0 s | written as '=1+2… |
| imports | schedule exchange file: export from one project, import into the other | @release | pass | 0.0 s | 4359 bytes round-tripped |
| imports | budget: export the CSV, import it into the other project | @release | pass | 0.0 s | budget.import |
| imports | punch-list: import a CSV file, then find the lines in the CSV export | @release | pass | 0.0 s | punch.import, then in punch-list.csv |
| imports | observations: import a CSV file, then find the lines in the CSV export | @release | pass | 0.0 s | observation.import, then in observations.csv |
| imports | submittals: import a CSV file, then find the lines in the CSV export | @release | pass | 0.0 s | submittal.import, then in submittals.csv |
| imports | schedule exchange (.xer) file: tasks from one project's export imported into the other | @release | pass | 0.0 s | 10 tasks round-tripped |
| uploads | a file over 25 MB is refused without a server error | @release | pass | 0.1 s | answered 303 The upload failed or the file is larger than 25 MB. |
| uploads | a photo upload must be an image | @release | pass | 0.0 s | POST /projects/2/photos/upload refused: Photos must be JPEG, PNG, GIF or WebP pictures. |
| safety | an uploaded web page is served as a download, never as a live page | @release | pass | 0.0 s | attachment + nosniff |
| uploads | every upload route refuses a file over its size limit without a server error or a change | @release | pass | 1.0 s | 32 upload routes refused an over-size file; no record to try: /prequalification/{vendor}/upload (no record listed on /prequalification), /projects/{id}/forms/{rid}/upload (no record listed on /projects/2/forms) |
| uploads | every upload route either refuses a web page or serves it back only as a download | @release | pass | 0.0 s | 32 routes; 22 took the file and serve files only as downloads; no record to try: /prequalification/{vendor}/upload (no record listed on /prequalification), /projects/{id}/forms/{rid}/upload (no record listed on /projects/2/forms) |
| api | a command with an API token works; without one it is refused | @release | pass | 0.0 s | 200 with, 401 without |
| api | a subcontractor's token cannot add people to the company | @release | pass | 0.0 s | 403 |
| api | MCP over HTTP answers initialize with a token | @release | pass | 0.0 s | initialize ok |
| sign-in | repeated wrong passwords are slowed down | @release | pass | 0.1 s | 429 after repeated failures |
| sign-in | signing out ends the session (the old cookie no longer works) | @release | known | 0.0 s | the session cookie from before signing out still opens the home page (known: #21, until 2026-11-17: FAILS (real finding): the session cookie from before sign-out still opens every page until it expires; sign-out ends nothing on the server) |
| access | a record opened under another project is not found | @release | pass | 0.0 s | RFI 8 under project 1: not found |
| safety | every form post without its CSRF token is refused and changes nothing | @smoke | pass | 0.0 s | 80 post routes refused |
| safety | every answer carries the security headers | @smoke | pass | 0.0 s | 5478 answers checked |
| safety | no answer shows panic text, a stack trace, a source location or a secret | @smoke | pass | 0.0 s | 5478 answers checked |
| safety | the journal holds no password, token or session cookie | @smoke | pass | 0.0 s | 206 lines |
| boot | the server was still running at the end of the run | @smoke | pass | 0.0 s | yes |
| safety | no panics in the server log (first run) | @smoke | pass | 0.0 s | 1 log lines |
| safety | the server log shows no password, token or session cookie (first run) | @smoke | pass | 0.0 s | clean |
| data | `buildkiosk check` replays every journal after the run | @release | pass | 0.0 s | summit-ridge: ok, 206 journal entries; company 1, people 6, vendors 3, templates 6, projects 2, members 12, cost_codes 75, rfis 8, submittals 8, transmittals 3, correspondence 1, meetings 3, tasks 2, spec_sections 6, punch_items 11, inspection_templates 8, inspections 3, observations 8, action_plans 1, incidents 2, daily_logs 3, schedule_tasks 39, budget_lines 13, commitments 5, direct_costs 6, ch… |
| data | the server starts again on the same data (behind HTTPS: --secure-cookies) | @release | pass | 0.1 s | started |
| sign-in | behind HTTPS the cookies are Secure and HSTS is sent | @release | pass | 0.0 s | Secure + HSTS |
| data | what the run made is still there after the restart | @release | pass | 0.0 s | the regression RFI is listed |
| boot | the restarted server was still running at the end | @release | pass | 0.0 s | yes |
| safety | no panics in the server log (after the restart) | @smoke | pass | 0.0 s | 1 log lines |
| safety | the server log shows no password, token or session cookie (after the restart) | @smoke | pass | 0.0 s | clean |
| upgrade | the previous release's data folder opens with this build | @release | skip | 0.0 s | no previous release program (BK_REGRESS_PREV_BIN) (known until 2026-11-17: no published release yet to upgrade from; the first release tag makes this check run (regress.yml downloads the previous release's Linux build)) |

Known-failing checks live in `qa/known-skips.json` (reason, issue, expiry of at most two weeks); see `docs/qa.md`.
