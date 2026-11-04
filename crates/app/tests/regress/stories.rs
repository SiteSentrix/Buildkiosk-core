//! The stories: what the suite does with the running program, in order. Each check is tagged `@smoke` (every pull
//! request) or `@release` (nightly and release tags); see `docs/qa.md`.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use crate::html;
use crate::http::{self, Client, FilePart, Log, Response, Seen};
use crate::report::{Run, Tag};
use crate::server::{self, Server};

/// The demo company (`buildkiosk demo`): every person signs in with this password.
const PASSWORD: &str = "demo-password-1";
const COMPANY: &str = "summit-ridge";

/// The demo people, one per role.
const ROLES: [(&str, &str); 6] = [
    ("admin", "ada@example.com"),
    ("project manager", "priya.raman@example.com"),
    ("superintendent", "marcus.lee@example.com"),
    ("project engineer", "elena.cruz@example.com"),
    ("subcontractor", "tom.okafor@example.com"),
    ("owner and design team", "hana.berg@example.com"),
];

/// What a page says when it is missing or not for this person (served with status 200: see qa/known-skips.json).
const NOT_FOUND_TEXT: &str = "That page doesn't exist, or you don't have access to it.";

/// Company-admin pages that other roles must not get.
const ADMIN_PAGES: [&str; 3] = ["/settings", "/admin/backups", "/admin/email"];

/// Paths anyone may open without signing in (sign-in itself, static files, the token links sent to outside people).
const PUBLIC: [&str; 9] = ["/login", "/setup", "/static/", "/bid/", "/sign/", "/prequal/", "/language", "/mcp", "/api/"];

/// An 8×8 white PNG, made for this suite.
const PNG: [u8; 72] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08,
    0x08, 0x02, 0x00, 0x00, 0x00, 0x4b, 0x6d, 0x29, 0xdc, 0x00, 0x00, 0x00, 0x0f, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xf8, 0x8f, 0x03, 0x30,
    0x0c, 0x2d, 0x09, 0x00, 0xba, 0x1e, 0xbf, 0x41, 0x89, 0xe8, 0xb6, 0xbb, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

struct Ctx {
    bin: PathBuf,
    work: PathBuf,
    data: PathBuf,
    server: Option<Server>,
    log: Log,
    secrets: Rc<RefCell<Vec<String>>>,
    users: BTreeMap<&'static str, Client>,
    tokens: BTreeMap<&'static str, String>,
    projects: Vec<u64>,
    routes: Vec<Route>,
}

impl Ctx {
    fn client(&self, who: &str) -> Client {
        let port = self.server.as_ref().map_or(0, |s| s.port);
        Client::new(port, who, &self.log, &self.secrets)
    }
}

fn user<'a>(users: &'a mut BTreeMap<&'static str, Client>, role: &str) -> Result<&'a mut Client, String> {
    users.get_mut(role).ok_or_else(|| format!("the {role} is not signed in (see the sign-in checks)"))
}

/// Runs every story this suite includes; the request log for the report.
pub fn all(run: &mut Run, root: &Path) -> Vec<Seen> {
    let log: Log = Rc::new(RefCell::new(Vec::new()));
    let work = std::env::temp_dir().join(format!("buildkiosk-regress-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    let _ = std::fs::create_dir_all(&work);
    let mut ctx = Ctx {
        bin: server::bin(),
        data: work.join("data"),
        work: work.clone(),
        server: None,
        log: log.clone(),
        secrets: Rc::new(RefCell::new(vec![PASSWORD.to_string()])),
        users: BTreeMap::new(),
        tokens: BTreeMap::new(),
        projects: Vec::new(),
        routes: route_table(root),
    };
    run.check("suite", "the page reader understands forms and links", Tag::Smoke, reader_self_test);
    if boot(run, &mut ctx) {
        sign_in_stories(run, &mut ctx);
        route_stories(run, &mut ctx);
        loop_stories(run, &mut ctx);
        if run.wants(Tag::Release) {
            tool_stories(run, &mut ctx);
            exchange_stories(run, &mut ctx);
            upload_stories(run, &mut ctx);
            api_stories(run, &mut ctx);
            session_stories(run, &mut ctx);
        }
        safety_stories(run, &mut ctx);
        restart_stories(run, &mut ctx);
    }
    upgrade_story(run, &ctx);
    if let Some(mut s) = ctx.server.take() {
        s.stop();
    }
    let _ = std::fs::remove_dir_all(&work);
    let seen = log.borrow().clone();
    drop(ctx);
    seen
}

fn reader_self_test() -> Result<String, String> {
    let page = r#"<form method="post" action="/do/rfi.create" class="stack"><input type="hidden" name="_csrf" value="abc"><input type="text" name="subject" required maxlength="200" value=""><textarea name="question">a &amp; b</textarea><select name="manager"><option value="">Choose</option><option value="7" selected>Ann</option></select><input type="checkbox" name="private" value="true"><button type="submit" name="open" value="true">Open</button></form><a href="/x?a=1&amp;b=2">x</a>"#;
    let f = html::forms(page);
    let form = f.first().ok_or("no form read")?;
    let sent = form.fill(&[("open", "true"), ("absent", "x")]);
    let want = [("_csrf", "abc"), ("subject", "QA regression subject"), ("question", "a & b"), ("manager", "7"), ("open", "true")];
    for (k, v) in want {
        if !sent.iter().any(|(a, b)| a == k && b == v) {
            return Err(format!("expected {k}={v} in {sent:?}"));
        }
    }
    if sent.iter().any(|(k, _)| k == "private" || k == "absent") {
        return Err(format!("sent fields a browser would not: {sent:?}"));
    }
    if html::links(page) != ["/x?a=1&b=2"] {
        return Err(format!("links: {:?}", html::links(page)));
    }
    Ok("forms, selects, checkboxes, buttons and links".to_string())
}

// -- boot

fn boot(run: &mut Run, ctx: &mut Ctx) -> bool {
    let (bin, data) = (ctx.bin.clone(), ctx.data.clone());
    let data_s = data.to_string_lossy().into_owned();
    if !run.check("boot", "the demo company is created from the command line", Tag::Smoke, || {
        server::cli(&bin, &["demo", "--data", data_s.as_str()]).map(|o| o.lines().next().unwrap_or("").to_string())
    }) {
        return false;
    }
    let tokens = &mut ctx.tokens;
    let secrets = ctx.secrets.clone();
    run.check("boot", "an API token is made for every role from the command line", Tag::Smoke, || {
        for (role, email) in ROLES {
            let out = server::cli(&bin, &["token", "--data", data_s.as_str(), "--company", COMPANY, "--as", email, "--name", "regress"])?;
            let token = out.trim().to_string();
            if token.len() < 16 {
                return Err(format!("odd token for {email}: {} characters", token.len()));
            }
            secrets.borrow_mut().push(token.clone());
            tokens.insert(role, token);
        }
        Ok(format!("{} tokens", tokens.len()))
    });
    let log_path = ctx.work.join("server.log");
    let mut started = None;
    let ok = run.check("boot", "the server starts on a free port and answers", Tag::Smoke, || {
        let s = Server::start(&bin, &data, &[], &log_path)?;
        let port = s.port;
        started = Some(s);
        Ok(format!("127.0.0.1:{port}"))
    });
    ctx.server = started;
    if !ok {
        return false;
    }
    let mut anon = ctx.client("anonymous");
    run.check("boot", "the sign-in page answers", Tag::Smoke, || {
        let res = anon.get("/login");
        if res.status == 200 && res.text().contains("password") { Ok("200".to_string()) } else { Err(format!("{}: {}", res.status, res.snippet())) }
    })
}

// -- sign-in

fn sign_in(c: &mut Client, email: &str, password: &str) -> Result<Response, String> {
    let page = c.get("/login");
    if page.status != 200 {
        return Err(format!("GET /login answered {}", page.status));
    }
    let forms = html::forms(&page.text());
    let form = html::post_form(&forms, "/login").ok_or("no sign-in form on /login")?;
    let fields = form.fill(&[("email", email), ("password", password)]);
    Ok(c.post_form("/login", &fields))
}

fn sign_in_stories(run: &mut Run, ctx: &mut Ctx) {
    for (role, email) in ROLES {
        let mut c = ctx.client(role);
        let ok = run.check("sign-in", &format!("{role} signs in"), Tag::Smoke, || {
            let res = sign_in(&mut c, email, PASSWORD)?;
            if !res.is_redirect() {
                return Err(format!("POST /login answered {}: {}", res.status, res.snippet()));
            }
            if !c.cookies.contains_key("bk_session") {
                return Err("no session cookie".to_string());
            }
            let home = c.get("/");
            if home.status != 200 {
                return Err(format!("home page answered {}", home.status));
            }
            Ok(email.to_string())
        });
        if ok {
            if let Some(v) = c.cookies.get("bk_session") {
                ctx.secrets.borrow_mut().push(v.clone());
            }
            ctx.users.insert(role, c);
        }
    }
    let mut c = ctx.client("anonymous");
    run.check("sign-in", "session and form cookies are HttpOnly and SameSite", Tag::Smoke, || {
        let page = c.get("/login");
        let csrf = page.all("set-cookie").into_iter().find(|v| v.starts_with("bk_csrf=")).map(str::to_string).ok_or("no bk_csrf cookie on /login")?;
        let res = sign_in(&mut c, "ada@example.com", PASSWORD)?;
        let session = res.all("set-cookie").into_iter().find(|v| v.starts_with("bk_session=")).map(str::to_string).ok_or("no bk_session cookie")?;
        for (name, v) in [("bk_csrf", &csrf), ("bk_session", &session)] {
            for flag in ["HttpOnly", "SameSite=", "Path=/"] {
                if !v.contains(flag) {
                    return Err(format!("{name} cookie lacks {flag}"));
                }
            }
        }
        Ok("HttpOnly, SameSite, Path=/".to_string())
    });
    let mut c = ctx.client("anonymous");
    run.check("sign-in", "a wrong password is refused", Tag::Smoke, || {
        let res = sign_in(&mut c, "ada@example.com", "not-the-password")?;
        if res.status == 401 && !c.cookies.contains_key("bk_session") { Ok("401".to_string()) } else { Err(format!("answered {}", res.status)) }
    });
    let mut c = ctx.client("anonymous");
    run.check("safety", "a sign-in without the form token is refused (CSRF)", Tag::Smoke, || {
        let _ = c.get("/login");
        let res = c.post_form("/login", &[("email".to_string(), "ada@example.com".to_string()), ("password".to_string(), PASSWORD.to_string())]);
        if res.status == 403 && !c.cookies.contains_key("bk_session") { Ok("403".to_string()) } else { Err(format!("answered {}", res.status)) }
    });
    let projects = &mut ctx.projects;
    if let Ok(admin) = user(&mut ctx.users, "admin") {
        run.check("projects", "the demo projects are listed on the home page", Tag::Smoke, || {
            let home = admin.get("/");
            for link in html::links(&home.text()) {
                if let Some(id) = link.strip_prefix("/projects/").and_then(|r| r.split(['/', '?']).next()).and_then(|n| n.parse::<u64>().ok())
                    && !projects.contains(&id)
                {
                    projects.push(id);
                }
            }
            if projects.len() >= 2 { Ok(format!("projects {projects:?}")) } else { Err(format!("found {projects:?}")) }
        });
    }
}

// -- routes

struct Route {
    path: String,
    get: bool,
    post: bool,
}

/// The routes the server registers, read from its source (`.route("…", get(…))`), plus the ones its generic
/// tools register with `format!`.
fn route_table(root: &Path) -> Vec<Route> {
    let mut out: Vec<Route> = Vec::new();
    let dir = root.join("crates/server/src");
    for f in ["lib.rs", "portal.rs", "sign_portal.rs", "prequal_portal.rs"] {
        let src = std::fs::read_to_string(dir.join(f)).unwrap_or_default();
        let mut rest = src.as_str();
        while let Some(i) = rest.find(".route(") {
            rest = &rest[i + 7..];
            let next = rest.find(".route(").unwrap_or(rest.len());
            let segment = &rest[..next];
            let Some(t) = segment.trim_start().strip_prefix('"') else { continue };
            let Some(j) = t.find('"') else { continue };
            let after = t[j + 1..].trim_start().trim_start_matches(',').trim_start();
            let get = after.starts_with("get(");
            let post = after.starts_with("post(") || segment.contains(").post(");
            out.push(Route { path: t[..j].to_string(), get, post });
        }
    }
    let mut add = |path: String, get: bool| out.push(Route { path, get, post: !get });
    for base in ["punch-list", "observations", "tasks", "incidents"] {
        for s in ["", ".csv", ".pdf", "/new", "/{iid}", "/{iid}/edit"] {
            add(format!("/projects/{{id}}/{base}{s}"), true);
        }
        add(format!("/projects/{{id}}/{base}/import"), false);
        add(format!("/projects/{{id}}/{base}/{{iid}}/upload"), false);
    }
    for base in ["change-events", "change-orders", "budget-changes", "prime-contracts", "prime-change-orders"] {
        for s in ["", "/new", "/{did}", "/{did}/edit"] {
            add(format!("/projects/{{id}}/{base}{s}"), true);
        }
        add(format!("/projects/{{id}}/{base}/save"), false);
        add(format!("/projects/{{id}}/{base}/{{did}}/save"), false);
    }
    for base in ["owner-invoices", "commitment-invoices"] {
        for s in ["", "/{iid}", "/{iid}/print", "/{iid}/pdf", "/{iid}/csv"] {
            add(format!("/projects/{{id}}/{base}{s}"), true);
        }
        add(format!("/projects/{{id}}/{base}/{{iid}}/save"), false);
    }
    out.sort_by(|a, b| a.path.as_str().cmp(b.path.as_str()));
    out
}

/// A route with its `{params}` filled: the project for `{id}`, `other` for the other ids.
fn expand(template: &str, project: u64, other: &str) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        let Some(j) = rest[i..].find('}') else { break };
        let name = &rest[i + 1..i + j];
        let value = match name {
            "id" => project.to_string(),
            "sha" => "0".repeat(64),
            "token" => "regress-not-a-real-link".to_string(),
            "kind" => "rfi".to_string(),
            "command" => "rfi.create".to_string(),
            _ => other.to_string(),
        };
        out.push_str(&value);
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}

fn is_public(path: &str) -> bool {
    PUBLIC.iter().any(|p| path.starts_with(p))
}

fn tally(statuses: &BTreeMap<u16, usize>) -> String {
    statuses.iter().map(|(s, n)| format!("{s}×{n}")).collect::<Vec<_>>().join(", ")
}

fn route_stories(run: &mut Run, ctx: &mut Ctx) {
    let project = ctx.projects.first().copied().unwrap_or(1);
    let gets: Vec<String> = ctx.routes.iter().filter(|r| r.get).map(|r| r.path.clone()).collect();
    let release = run.wants(Tag::Release);
    for (role, _) in ROLES {
        let Ok(c) = user(&mut ctx.users, role) else { continue };
        run.check("routes", &format!("{role}: every page in the route table answers without a server error"), Tag::Smoke, || {
            let mut bad = Vec::new();
            let mut statuses: BTreeMap<u16, usize> = BTreeMap::new();
            for t in &gets {
                let path = expand(t, project, "1");
                let res = c.get(&path);
                *statuses.entry(res.status).or_default() += 1;
                if res.status >= 500 || res.status == 0 {
                    bad.push(format!("{path} → {}", res.status));
                }
            }
            if bad.is_empty() {
                Ok(format!("{} routes: {}", gets.len(), tally(&statuses)))
            } else {
                Err(format!("{} server errors: {}", bad.len(), bad.join("; ")))
            }
        });
    }
    if let Ok(c) = user(&mut ctx.users, "admin") {
        run.check("routes", "admin: made-up and malformed ids never cause a server error", Tag::Release, || {
            let mut bad = Vec::new();
            for t in gets.iter().filter(|t| t.contains('{')) {
                for other in ["99999999", "x", "-1"] {
                    let path = expand(t, project, other);
                    let res = c.get(&path);
                    if res.status >= 500 || res.status == 0 {
                        bad.push(format!("{path} → {}", res.status));
                    }
                }
                let path = expand(t, 99_999_999, "1");
                let res = c.get(&path);
                if res.status >= 500 || res.status == 0 {
                    bad.push(format!("{path} → {}", res.status));
                }
            }
            if bad.is_empty() { Ok("no server errors".to_string()) } else { Err(bad.join("; ")) }
        });
    }
    let mut anon = ctx.client("anonymous");
    run.check("access", "anonymous: every private page sends the visitor to sign in", Tag::Smoke, || {
        let mut bad = Vec::new();
        let mut count = 0;
        for t in &gets {
            let path = expand(t, project, "1");
            let res = anon.get(&path);
            if res.status >= 500 || res.status == 0 {
                bad.push(format!("{path} → {}", res.status));
                continue;
            }
            if is_public(&path) {
                continue;
            }
            count += 1;
            // A page may first send the visitor elsewhere on the site (a list instead of a form): follow a few hops.
            let mut res = res;
            let mut hops = 0;
            while res.is_redirect() && !res.location().starts_with("/login") && res.location().starts_with('/') && hops < 3 {
                let next = res.location().to_string();
                res = anon.get(&next);
                hops += 1;
            }
            let to_login = res.is_redirect() && res.location().starts_with("/login");
            if !(to_login || matches!(res.status, 401 | 403 | 404)) {
                bad.push(format!("{path} → {} {}", res.status, res.location()));
            }
        }
        if bad.is_empty() { Ok(format!("{count} private pages")) } else { Err(bad.join("; ")) }
    });
    for role in ["subcontractor", "owner and design team", "project engineer"] {
        let Ok(c) = user(&mut ctx.users, role) else { continue };
        run.check("access", &format!("{role}: company admin pages are refused"), Tag::Smoke, || {
            let mut bad = Vec::new();
            for path in ADMIN_PAGES {
                let res = c.get(path);
                let refused = res.status != 200 || res.text().contains(NOT_FOUND_TEXT);
                if !refused || res.status >= 500 {
                    bad.push(format!("{path} → {}", res.status));
                }
            }
            if bad.is_empty() { Ok(ADMIN_PAGES.join(", ")) } else { Err(format!("shown: {}", bad.join("; "))) }
        });
    }
    if let Ok(c) = user(&mut ctx.users, "subcontractor") {
        run.check("access", "refused and missing pages answer 403, 404 or a redirect, not 200", Tag::Release, || {
            let mut bad = Vec::new();
            for path in ADMIN_PAGES.iter().map(|p| (*p).to_string()).chain([format!("/projects/{project}/rfis/99999999")]) {
                let res = c.get(&path);
                // A redirect (to sign in, or back to a page they may see) is a refusal too (issue #3).
                if !(matches!(res.status, 403 | 404) || res.is_redirect()) {
                    bad.push(format!("{path} → {}", res.status));
                }
            }
            if bad.is_empty() { Ok("403/404 or a redirect".to_string()) } else { Err(bad.join("; ")) }
        });
    }
    // Crawl: every link each role is shown, followed from the home page.
    let (limit, per_shape) = if release { (1500, 6) } else { (150, 2) };
    for (role, _) in ROLES {
        let Ok(c) = user(&mut ctx.users, role) else { continue };
        let found = crawl(c, limit, per_shape);
        let pages = found.pages;
        let statuses = tally(&found.statuses);
        let errors = found.errors;
        run.check("crawl", &format!("{role}: every linked page answers without a server error"), Tag::Smoke, || {
            if errors.is_empty() {
                Ok(format!("{pages} pages: {statuses}"))
            } else {
                Err(format!("{} server errors: {}", errors.len(), errors.join("; ")))
            }
        });
        let broken = found.broken;
        run.check("crawl", &format!("{role}: every link shown opens (no 4xx, no \"not found\" page)"), Tag::Release, || {
            if broken.is_empty() { Ok(format!("{pages} pages")) } else { Err(format!("{} broken: {}", broken.len(), broken.join("; "))) }
        });
    }
}

struct Crawl {
    pages: usize,
    statuses: BTreeMap<u16, usize>,
    errors: Vec<String>,
    broken: Vec<String>,
}

fn local_link(link: &str) -> Option<String> {
    let link = link.split('#').next().unwrap_or("");
    if !link.starts_with('/') || link.starts_with("//") || link.starts_with("/static/") || link.starts_with("/language") || link == "/logout" {
        return None;
    }
    Some(link.to_string())
}

/// A link's shape: ids replaced, query values dropped (so a list of 500 RFIs is visited a few times, not 500).
fn shape(link: &str) -> String {
    let (path, query) = link.split_once('?').unwrap_or((link, ""));
    let segs: Vec<&str> = path
        .split('/')
        .map(|s| {
            if !s.is_empty() && (s.chars().all(|c| c.is_ascii_digit()) || (s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit()))) {
                "#"
            } else {
                s
            }
        })
        .collect();
    let mut keys: Vec<&str> = query.split('&').filter_map(|kv| kv.split('=').next()).filter(|k| !k.is_empty()).collect();
    keys.sort_unstable();
    format!("{}?{}", segs.join("/"), keys.join("&"))
}

fn crawl(c: &mut Client, limit: usize, per_shape: usize) -> Crawl {
    let mut out = Crawl { pages: 0, statuses: BTreeMap::new(), errors: Vec::new(), broken: Vec::new() };
    let mut queue: VecDeque<(String, String)> = VecDeque::from([("/".to_string(), "the start".to_string())]);
    let mut seen: BTreeSet<String> = BTreeSet::from(["/".to_string()]);
    let mut shapes: BTreeMap<String, usize> = BTreeMap::new();
    while let Some((path, from)) = queue.pop_front() {
        if out.pages >= limit {
            break;
        }
        let res = c.get(&path);
        out.pages += 1;
        *out.statuses.entry(res.status).or_default() += 1;
        if res.status >= 500 || res.status == 0 {
            out.errors.push(format!("{path} → {} (linked from {from})", res.status));
            continue;
        }
        if (400..500).contains(&res.status) {
            out.broken.push(format!("{path} → {} (linked from {from})", res.status));
            continue;
        }
        if res.status != 200 || !res.is_html() {
            continue;
        }
        let text = res.text();
        if text.contains(NOT_FOUND_TEXT) {
            out.broken.push(format!("{path} shows \"not found\" (linked from {from})"));
            continue;
        }
        for link in html::links(&text) {
            let Some(p) = local_link(&link) else { continue };
            if seen.contains(&p) {
                continue;
            }
            let n = shapes.entry(shape(&p)).or_default();
            if *n >= per_shape {
                continue;
            }
            *n += 1;
            seen.insert(p.clone());
            queue.push_back((p, path.clone()));
        }
    }
    out
}

// -- tool loops

fn journal(data: &Path) -> Vec<Value> {
    let path = data.join("companies").join(COMPANY).join("journal.jsonl");
    std::fs::read_to_string(path).unwrap_or_default().lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

fn journal_len(data: &Path) -> usize {
    journal(data).len()
}

fn cmd_name(e: &Value) -> String {
    e.get("cmd").and_then(Value::as_str).unwrap_or("").to_string()
}

/// The commands journaled since `before`.
fn new_cmds(data: &Path, before: usize) -> Vec<String> {
    journal(data).iter().skip(before).map(cmd_name).collect()
}

/// The journal entries since `before`: one for `cmd` (an exact id, or a prefix ending in `.`), made by a person.
fn journaled(data: &Path, before: usize, cmd: &str) -> Result<String, String> {
    let all = journal(data);
    let new: Vec<&Value> = all.iter().skip(before).collect();
    let hit = new.iter().copied().find(|e| {
        let c = cmd_name(e);
        if cmd.ends_with('.') { c.starts_with(cmd) } else { c == cmd }
    });
    match hit {
        Some(e) if e.get("actor").is_some_and(|a| !a.is_null()) => Ok(cmd_name(e)),
        Some(_) => Err(format!("journal entry `{cmd}` names no person")),
        None => {
            let names: Vec<String> = new.iter().copied().map(cmd_name).collect();
            Err(format!("no `{cmd}` in the journal (new entries: {})", if names.is_empty() { "none".to_string() } else { names.join(", ") }))
        }
    }
}

/// The server's "done" redirect, or what went wrong.
fn done(res: &Response, what: &str) -> Result<String, String> {
    if !res.is_redirect() {
        return Err(format!("{what} answered {}: {}", res.status, res.snippet()));
    }
    let loc = res.location().to_string();
    if let Some(e) = http::query_value(&loc, "error") {
        return Err(format!("{what} refused: {e}"));
    }
    if loc.starts_with("/login") {
        return Err(format!("{what} sent the person back to sign in"));
    }
    Ok(loc)
}

fn page_forms(c: &mut Client, page: &str) -> Result<(Vec<html::Form>, String), String> {
    let res = c.get(page);
    if res.status != 200 {
        return Err(format!("GET {page} answered {}", res.status));
    }
    let text = res.text();
    if text.contains(NOT_FOUND_TEXT) {
        return Err(format!("GET {page} says not found"));
    }
    Ok((html::forms(&text), text))
}

fn post_actions(forms: &[html::Form]) -> String {
    forms.iter().filter(|f| f.method == "post").map(|f| f.action.as_str()).collect::<Vec<_>>().join(", ")
}

/// Opens `page`, fills the post form for `action` like a person would, sends it (with `file` on an upload form)
/// and expects the server's "done" redirect; its target.
fn submit(c: &mut Client, page: &str, action: &str, set: &[(&str, &str)], file: Option<FilePart>) -> Result<String, String> {
    let (forms, _) = page_forms(c, page)?;
    let Some(form) = html::post_form(&forms, action) else {
        return Err(format!("no form for `{action}` on {page} (forms: {})", post_actions(&forms)));
    };
    let fields = form.fill(set);
    let files: Vec<FilePart> = file.into_iter().collect();
    let res = if form.multipart { c.post_multipart(&form.action, &fields, &files) } else { c.post_form(&form.action, &fields) };
    done(&res, &format!("POST {}", form.action))
}

/// Presses the first of `commands` that has a button on `page`; the command pressed.
fn press(c: &mut Client, page: &str, commands: &[&str]) -> Result<String, String> {
    let (forms, _) = page_forms(c, page)?;
    for cmd in commands {
        let action = format!("/do/{cmd}");
        if let Some(f) = forms.iter().find(|f| f.method == "post" && f.action == action) {
            let res = c.post_form(&action, &f.fill(&[]));
            done(&res, &format!("POST {action}"))?;
            return Ok((*cmd).to_string());
        }
    }
    Err(format!("no button for {} on {page} (forms: {})", commands.join(" or "), post_actions(&forms)))
}

fn id_after(loc: &str, segment: &str) -> Option<u64> {
    let i = loc.find(segment)?;
    let digits: String = loc[i + segment.len()..].chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn text_file(name: &str) -> FilePart {
    FilePart::new("file", name, "text/plain", b"QA regression attachment: fictional test data.\n".to_vec())
}

/// One step of a tool loop: send the form like a person, then expect `cmd` in the journal. The redirect target.
fn step(c: &mut Client, data: &Path, page: &str, action: &str, set: &[(&str, &str)], file: Option<FilePart>, cmd: &str) -> Result<String, String> {
    let n = journal_len(data);
    let loc = submit(c, page, action, set, file)?;
    journaled(data, n, cmd).map_err(|e| format!("{e}, after the form on {page} went to {loc}"))?;
    Ok(loc)
}

/// Presses one of `commands` on `page`, then expects it in the journal; the command pressed.
fn press_step(c: &mut Client, data: &Path, page: &str, commands: &[&str]) -> Result<String, String> {
    let n = journal_len(data);
    let pressed = press(c, page, commands)?;
    journaled(data, n, &pressed)
}

fn loop_stories(run: &mut Run, ctx: &mut Ctx) {
    let p = ctx.projects.first().copied().unwrap_or(1);
    let data = ctx.data.clone();
    if let Ok(c) = user(&mut ctx.users, "project manager") {
        let base = format!("/projects/{p}/rfis");
        run.check("loops", "RFI: create, edit, attach, respond, close (journal checked each step)", Tag::Smoke, || {
            let set = [
                ("subject", "QA regression: ceiling height at grid C"),
                ("question", "Which drawing governs at grid C?"),
                ("assignees[]", html::FIRST),
                ("open", "true"),
            ];
            let loc = step(c, &data, &format!("{base}/new"), "/do/rfi.create", &set, None, "rfi.create")?;
            let id = id_after(&loc, "/rfis/").ok_or_else(|| format!("no RFI id in {loc}"))?;
            let here = format!("{base}/{id}");
            step(c, &data, &format!("{here}/edit"), "/do/rfi.update", &[("location", "Level 2, grid C")], None, "rfi.update")?;
            step(c, &data, &here, &format!("{here}/upload"), &[], Some(text_file("qa-note.txt")), "rfi.attach")?;
            step(c, &data, &here, &format!("{here}/respond"), &[("body", "QA regression: the section governs.")], None, "rfi.respond")?;
            press_step(c, &data, &here, &["rfi.close"])?;
            Ok(format!("RFI {id}: rfi.create → rfi.update → rfi.attach → rfi.respond → rfi.close"))
        });
        let base = format!("/projects/{p}/punch-list");
        run.check("loops", "punch list: create, edit, attach, finish (journal checked each step)", Tag::Smoke, || {
            let set = [("title", "QA regression: patch drywall at door 12"), ("location", "Level 1"), ("assignees[]", html::FIRST)];
            let loc = step(c, &data, &format!("{base}/new"), "/do/punch.create", &set, None, "punch.create")?;
            let id = id_after(&loc, "/punch-list/").ok_or_else(|| format!("no item id in {loc}"))?;
            let here = format!("{base}/{id}");
            step(c, &data, &format!("{here}/edit"), "/do/punch.update", &[("location", "Level 1, door 12")], None, "punch.update")?;
            step(c, &data, &here, &format!("{here}/upload"), &[], Some(text_file("qa-punch.txt")), "punch.attach")?;
            let finished = press_step(c, &data, &here, &["punch.complete", "punch.close", "punch.verify"])?;
            Ok(format!("item {id}: punch.create → punch.update → punch.attach → {finished}"))
        });
    }
    if let Ok(c) = user(&mut ctx.users, "superintendent") {
        let base = format!("/projects/{p}/daily-logs");
        run.check("loops", "daily log: create, edit, attach, submit (journal checked each step)", Tag::Smoke, || {
            let set = [("date", "2026-10-05"), ("work_performed", "QA regression: formed and poured footing F-3.")];
            let loc = step(c, &data, &format!("{base}/new"), "", &set, None, "daily_log.create")?;
            let id = id_after(&loc, "/daily-logs/").ok_or_else(|| format!("no log id in {loc}"))?;
            let here = format!("{base}/{id}");
            let edit = [("work_performed", "QA regression: formed, poured and stripped footing F-3.")];
            step(c, &data, &format!("{here}/edit"), "", &edit, None, "daily_log.update")?;
            step(c, &data, &here, &format!("{here}/upload"), &[], Some(text_file("qa-log.txt")), "daily_log.attach")?;
            press_step(c, &data, &here, &["daily_log.submit"])?;
            Ok(format!("log {id}: daily_log.create → daily_log.update → daily_log.attach → daily_log.submit"))
        });
    }
}

/// Every other tool: create from its "new" page, open, edit, attach (where the page offers it).
fn tool_stories(run: &mut Run, ctx: &mut Ctx) {
    let p = ctx.projects.first().copied().unwrap_or(1);
    let data = ctx.data.clone();
    let Ok(c) = user(&mut ctx.users, "project manager") else { return };
    // Each tool with what a person must choose on its "new" page (everything else is filled in as typed text).
    let tools: [(&str, &[(&str, &str)]); 14] = [
        ("submittals", &[]),
        ("observations", &[]),
        ("tasks", &[]),
        ("incidents", &[]),
        ("meetings", &[]),
        ("transmittals", &[]),
        ("correspondence", &[]),
        ("action-plans", &[]),
        ("inspections", &[("template", html::FIRST)]),
        ("commitments", &[]),
        ("change-events", &[]),
        ("prime-contracts", &[]),
        ("bidding", &[]),
        ("estimating", &[]),
    ];
    for (tool, choose) in tools {
        let base = format!("/projects/{p}/{tool}");
        run.check("tools", &format!("{tool}: create, open, edit, attach"), Tag::Release, || {
            let mut steps = Vec::new();
            let n = journal_len(&data);
            let loc = submit(c, &format!("{base}/new"), "", choose, None)?;
            let id = id_after(&loc, &format!("/{tool}/")).ok_or_else(|| format!("created, but no id in {loc}"))?;
            let made = new_cmds(&data, n);
            if made.is_empty() {
                return Err("created, but nothing in the journal".to_string());
            }
            steps.push(made.join("+"));
            let here = format!("{base}/{id}");
            let (forms, text) = page_forms(c, &here)?;
            let (edit, upload) = (format!("{here}/edit"), format!("{here}/upload"));
            if html::links(&text).contains(&edit) {
                let n = journal_len(&data);
                submit(c, &edit, "", &[], None)?;
                let edited = journal_len(&data) > n;
                steps.push(if edited { "edit".to_string() } else { "edit (no change)".to_string() });
            }
            if forms.iter().any(|f| f.action == upload) {
                let n = journal_len(&data);
                submit(c, &here, &upload, &[], Some(text_file("qa-attachment.txt")))?;
                let cmds = new_cmds(&data, n);
                if !cmds.iter().any(|c| c.ends_with(".attach")) {
                    return Err(format!("attached, but no `….attach` in the journal ({})", cmds.join(", ")));
                }
                steps.push(cmds.join("+"));
            }
            Ok(format!("{tool} {id}: {}", steps.join(" → ")))
        });
    }
    run.check("tools", "drawings: upload a sheet, open it and its viewer page", Tag::Release, || {
        let base = format!("/projects/{p}/drawings");
        let n = journal_len(&data);
        let png = FilePart::new("file", "qa-sheet.png", "image/png", PNG.to_vec());
        submit(c, &base, &format!("{base}/upload"), &[("number", "QA-101"), ("title", "QA regression sheet")], Some(png))?;
        journaled(&data, n, "drawing.add")?;
        let list = c.get(&base).text();
        let id = newest_id(&list, &format!("{base}/")).ok_or("the new sheet is not listed")?;
        for path in [format!("{base}/{id}"), format!("{base}/{id}/view")] {
            let res = c.get(&path);
            if res.status != 200 || res.text().contains(NOT_FOUND_TEXT) {
                return Err(format!("{path} answered {}", res.status));
            }
        }
        let png = c.get(&format!("{base}/{id}/view.png"));
        if png.status >= 500 {
            return Err(format!("view.png answered {}", png.status));
        }
        Ok(format!("sheet {id}; view.png {}", png.status))
    });
    run.check("tools", "schedule: add a task", Tag::Release, || {
        let base = format!("/projects/{p}/schedule");
        let n = journal_len(&data);
        submit(c, &base, "/do/schedule.add", &[("name", "QA regression: punch walk"), ("days", "2")], None)?;
        journaled(&data, n, "schedule.add")
    });
}

/// The largest id linked as `prefix<id>` on the page.
fn newest_id(page: &str, prefix: &str) -> Option<u64> {
    html::links(page).iter().filter_map(|l| l.strip_prefix(prefix)).filter_map(|r| r.parse::<u64>().ok()).max()
}

// -- imports and exports

fn exchange_stories(run: &mut Run, ctx: &mut Ctx) {
    let (p, q) = (ctx.projects.first().copied().unwrap_or(1), ctx.projects.get(1).copied().unwrap_or(2));
    let data = ctx.data.clone();
    let csvs: Vec<String> = ctx
        .routes
        .iter()
        // The forms report needs a chosen form template (`?template=`), so it is reached from its page instead.
        .filter(|r| {
            r.get && r.path.ends_with(".csv") && !r.path.contains("{iid}") && !r.path.contains("{eid}") && !r.path.ends_with("/forms/report.csv")
        })
        .map(|r| r.path.clone())
        .collect();
    let Ok(c) = user(&mut ctx.users, "admin") else { return };
    run.check("exports", "every CSV export downloads as CSV", Tag::Release, || {
        let mut bad = Vec::new();
        for t in &csvs {
            let path = expand(t, p, "1");
            let res = c.get(&path);
            let csv = res.header("content-type").is_some_and(|t| t.starts_with("text/csv"));
            if res.status != 200 || !csv || res.body.is_empty() {
                bad.push(format!("{path} → {} {}", res.status, res.header("content-type").unwrap_or("")));
            }
        }
        if bad.is_empty() { Ok(format!("{} exports", csvs.len())) } else { Err(bad.join("; ")) }
    });
    run.check("exports", "a spreadsheet formula typed into an RFI is defused in the CSV", Tag::Release, || {
        submit(c, &format!("/projects/{p}/rfis/new"), "/do/rfi.create", &[("subject", "=1+2 QA regression formula")], None)?;
        let csv = c.get(&format!("/projects/{p}/rfis.csv")).text();
        if csv.contains("'=1+2 QA regression formula") {
            Ok("written as '=1+2…".to_string())
        } else {
            Err("the formula is not defused".to_string())
        }
    });
    run.check("imports", "schedule exchange file: export from one project, import into the other", Tag::Release, || {
        let xml = c.get(&format!("/projects/{p}/schedule.xml"));
        if xml.status != 200 || !xml.text().contains('<') {
            return Err(format!("export answered {}", xml.status));
        }
        let n = journal_len(&data);
        let file = FilePart::new("file", "schedule.xml", "application/xml", xml.body.clone());
        submit(c, &format!("/projects/{q}/schedule"), &format!("/projects/{q}/schedule/import-xml"), &[], Some(file))?;
        journaled(&data, n, "schedule.import_xml")?;
        let csv = c.get(&format!("/projects/{q}/schedule.csv")).text();
        if csv.contains("Foundations") {
            Ok(format!("{} bytes round-tripped", xml.body.len()))
        } else {
            Err("the imported tasks are not in the other project".to_string())
        }
    });
    run.check("imports", "budget: export the CSV, import it into the other project", Tag::Release, || {
        let csv = c.get(&format!("/projects/{p}/budget.csv"));
        if csv.status != 200 {
            return Err(format!("export answered {}", csv.status));
        }
        let n = journal_len(&data);
        let file = FilePart::new("file", "budget.csv", "text/csv", csv.body.clone());
        submit(c, &format!("/projects/{q}/budget"), &format!("/projects/{q}/budget/import-file"), &[], Some(file))?;
        journaled(&data, n, "budget.import")
    });
}

// -- uploads

fn upload_stories(run: &mut Run, ctx: &mut Ctx) {
    let p = ctx.projects.first().copied().unwrap_or(1);
    let data = ctx.data.clone();
    let Ok(c) = user(&mut ctx.users, "project manager") else { return };
    let rfi = newest_id(&c.get(&format!("/projects/{p}/rfis")).text(), &format!("/projects/{p}/rfis/"));
    run.check("uploads", "a file over 25 MB is refused without a server error", Tag::Release, || {
        let rfi = rfi.ok_or("no RFI to attach to")?;
        let here = format!("/projects/{p}/rfis/{rfi}");
        let (forms, _) = page_forms(c, &here)?;
        let form = html::post_form(&forms, &format!("{here}/upload")).ok_or("no upload form")?;
        let n = journal_len(&data);
        let big = FilePart::new("file", "qa-big.bin", "application/octet-stream", vec![b'x'; 26 * 1024 * 1024]);
        let res = c.post_multipart(&form.action, &form.fill(&[]), &[big]);
        if res.status >= 500 {
            return Err(format!("answered {}", res.status));
        }
        if res.is_redirect() && http::query_value(res.location(), "ok").is_some() {
            return Err("the server said the file was attached".to_string());
        }
        if journal_len(&data) != n {
            return Err("something was journaled".to_string());
        }
        Ok(format!("answered {} {}", res.status, http::query_value(res.location(), "error").unwrap_or_default()))
    });
    run.check("uploads", "a photo upload must be an image", Tag::Release, || {
        let base = format!("/projects/{p}/photos");
        let n = journal_len(&data);
        let r = submit(c, &base, &format!("{base}/upload"), &[], Some(text_file("not-a-photo.jpg")));
        if new_cmds(&data, n).iter().any(|c| c.starts_with("photo.")) {
            return Err("a text file was stored as a photo".to_string());
        }
        match r {
            Err(e) if e.contains("refused") => Ok(e),
            Err(e) => Err(e),
            Ok(loc) => Err(format!("accepted: {loc}")),
        }
    });
    run.check("safety", "an uploaded web page is served as a download, never as a live page", Tag::Release, || {
        let base = format!("/projects/{p}/documents");
        let n = journal_len(&data);
        let page = FilePart::new("file", "qa-page.html", "text/html", b"<html><body><script>alert(1)</script>QA regression</body></html>".to_vec());
        submit(c, &base, &format!("{base}/upload"), &[("title", "QA regression web page")], Some(page))?;
        journaled(&data, n, "document.add")?;
        let list = c.get(&base).text();
        let id = newest_id(&list, &format!("{base}/")).ok_or("the new document is not listed")?;
        let detail = c.get(&format!("{base}/{id}")).text();
        let links = html::links(&detail);
        let download = links.iter().find(|l| l.contains("/files/")).ok_or("no download link on the document page")?;
        let res = c.get(download);
        let attachment = res.header("content-disposition").is_some_and(|d| d.starts_with("attachment"));
        let nosniff = res.header("x-content-type-options") == Some("nosniff");
        if !(attachment && nosniff) {
            return Err(format!("{download}: disposition {:?}, nosniff {nosniff}", res.header("content-disposition")));
        }
        if let Some(view) = links.iter().find(|l| l.contains("/view/")) {
            let res = c.get(view);
            let live = res.header("content-type").is_some_and(|t| t.starts_with("text/html"));
            let scripts_off = res.header("content-security-policy").is_some_and(|p| p.contains("script-src 'none'") || p.contains("sandbox"));
            if live && !scripts_off {
                return Err(format!("{view} serves the page with scripts allowed"));
            }
        }
        Ok("attachment + nosniff".to_string())
    });
}

// -- API and MCP

fn api_stories(run: &mut Run, ctx: &mut Ctx) {
    let p = ctx.projects.first().copied().unwrap_or(1);
    let data = ctx.data.clone();
    let admin_token = ctx.tokens.get("admin").cloned().unwrap_or_default();
    let sub_token = ctx.tokens.get("subcontractor").cloned().unwrap_or_default();
    let mut c = ctx.client("api");
    run.check("api", "a command with an API token works; without one it is refused", Tag::Release, || {
        c.bearer = Some(admin_token.clone());
        let res = c.post_json("/api/v1/commands/rfi.list", &format!("{{\"project\": {p}}}"));
        if res.status != 200 || !res.text().contains("\"ok\":true") {
            return Err(format!("with a token: {} {}", res.status, res.snippet()));
        }
        c.bearer = None;
        let res = c.post_json("/api/v1/commands/rfi.list", &format!("{{\"project\": {p}}}"));
        if res.status != 401 {
            return Err(format!("without a token: {}", res.status));
        }
        Ok("200 with, 401 without".to_string())
    });
    run.check("api", "a subcontractor's token cannot add people to the company", Tag::Release, || {
        c.bearer = Some(sub_token.clone());
        let n = journal_len(&data);
        let res = c.post_json("/api/v1/commands/person.create", r#"{"first_name": "Q", "last_name": "A", "email": "qa-intruder@example.com"}"#);
        c.bearer = None;
        if !(400..500).contains(&res.status) || journal_len(&data) != n {
            return Err(format!("answered {}: {}", res.status, res.snippet()));
        }
        Ok(res.status.to_string())
    });
    run.check("api", "MCP over HTTP answers initialize with a token", Tag::Release, || {
        c.bearer = Some(admin_token.clone());
        let body = r#"{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "regress", "version": "1"}}}"#;
        let res = c.post_json("/mcp", body);
        c.bearer = None;
        if res.status == 200 && res.text().contains("\"result\"") { Ok("initialize ok".to_string()) } else { Err(format!("{} {}", res.status, res.snippet())) }
    });
}

// -- sessions

fn session_stories(run: &mut Run, ctx: &mut Ctx) {
    let p = ctx.projects.first().copied().unwrap_or(1);
    let q = ctx.projects.get(1).copied().unwrap_or(2);
    let mut c = ctx.client("anonymous");
    run.check("sign-in", "repeated wrong passwords are slowed down", Tag::Release, || {
        let mut last = 0;
        for _ in 0..10 {
            last = sign_in(&mut c, "nobody@example.com", "wrong-password")?.status;
            if last == 429 {
                return Ok("429 after repeated failures".to_string());
            }
        }
        Err(format!("still {last} after 10 attempts"))
    });
    let mut c = ctx.client("admin (sign-out)");
    run.check("sign-in", "signing out ends the session (the old cookie no longer works)", Tag::Release, || {
        let res = sign_in(&mut c, "ada@example.com", PASSWORD)?;
        if !res.is_redirect() {
            return Err(format!("sign-in answered {}", res.status));
        }
        let old = c.cookies.clone();
        let (forms, _) = page_forms(&mut c, "/")?;
        let form = forms.iter().find(|f| f.action == "/logout").ok_or("no sign-out form on the home page")?;
        let res = c.post_form("/logout", &form.fill(&[]));
        if !(res.is_redirect() && res.location().starts_with("/login")) {
            return Err(format!("sign-out answered {}", res.status));
        }
        if c.get("/").location() != "/login" {
            return Err("still signed in after signing out".to_string());
        }
        c.cookies = old;
        let res = c.get("/");
        if res.status == 200 {
            Err("the session cookie from before signing out still opens the home page".to_string())
        } else {
            Ok("old cookie refused".to_string())
        }
    });
    let Ok(pm) = user(&mut ctx.users, "project manager") else { return };
    run.check("access", "a record opened under another project is not found", Tag::Release, || {
        let rfi = newest_id(&pm.get(&format!("/projects/{p}/rfis")).text(), &format!("/projects/{p}/rfis/")).ok_or("no RFI listed")?;
        let res = pm.get(&format!("/projects/{q}/rfis/{rfi}"));
        if res.status == 404 || res.text().contains(NOT_FOUND_TEXT) {
            Ok(format!("RFI {rfi} under project {q}: not found"))
        } else {
            Err(format!("answered {}", res.status))
        }
    });
}

// -- safety over everything seen

fn safety_stories(run: &mut Run, ctx: &mut Ctx) {
    let project = ctx.projects.first().copied().unwrap_or(1);
    let data = ctx.data.clone();
    let posts: Vec<String> = ctx.routes.iter().filter(|r| r.post).map(|r| r.path.clone()).collect();
    if let Ok(c) = user(&mut ctx.users, "admin") {
        run.check("safety", "every form post without its CSRF token is refused and changes nothing", Tag::Smoke, || {
            let n = journal_len(&data);
            let mut bad = Vec::new();
            for t in &posts {
                let path = expand(t, project, "1");
                let res = c.post_form(&path, &[("project".to_string(), project.to_string()), ("id".to_string(), "1".to_string())]);
                let ok_flash = res.is_redirect() && http::query_value(res.location(), "ok").is_some();
                if res.status >= 500 || res.status == 0 || (200..300).contains(&res.status) || ok_flash {
                    bad.push(format!("{path} → {} {}", res.status, res.location()));
                }
            }
            let after = journal_len(&data);
            if after != n {
                bad.push(format!("{} journal entries were written", after - n));
            }
            if bad.is_empty() { Ok(format!("{} post routes refused", posts.len())) } else { Err(bad.join("; ")) }
        });
    }
    let seen = ctx.log.borrow().clone();
    run.check("safety", "every answer carries the security headers", Tag::Smoke, || {
        let bad: Vec<String> =
            seen.iter().filter(|s| !s.missing.is_empty()).map(|s| format!("{} {} lacks {}", s.method, s.path, s.missing.join(", "))).collect();
        if bad.is_empty() {
            Ok(format!("{} answers checked", seen.len()))
        } else {
            Err(format!("{}: {}", bad.len(), bad.iter().take(20).cloned().collect::<Vec<_>>().join("; ")))
        }
    });
    run.check("safety", "no answer shows panic text, a stack trace, a source location or a secret", Tag::Smoke, || {
        let bad: Vec<String> =
            seen.iter().filter(|s| !s.leaks.is_empty()).map(|s| format!("{} {} ({}) {}", s.method, s.path, s.who, s.leaks.join(", "))).collect();
        if bad.is_empty() {
            Ok(format!("{} answers checked", seen.len()))
        } else {
            Err(format!("{}: {}", bad.len(), bad.iter().take(20).cloned().collect::<Vec<_>>().join("; ")))
        }
    });
    let secrets = ctx.secrets.borrow().clone();
    run.check("safety", "the journal holds no password, token or session cookie", Tag::Smoke, || {
        let path = data.join("companies").join(COMPANY).join("journal.jsonl");
        let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        if secrets.iter().any(|s| text.contains(s.as_str())) || text.contains("$argon2") {
            return Err("a secret is in the journal".to_string());
        }
        Ok(format!("{} lines", text.lines().count()))
    });
}

fn server_log_checks(run: &mut Run, log: &str, secrets: &[String], when: &str) {
    run.check("safety", &format!("no panics in the server log ({when})"), Tag::Smoke, || {
        let panics: Vec<&str> = log.lines().filter(|l| l.contains("panicked")).take(10).collect();
        if panics.is_empty() { Ok(format!("{} log lines", log.lines().count())) } else { Err(panics.join(" | ")) }
    });
    run.check("safety", &format!("the server log shows no password, token or session cookie ({when})"), Tag::Smoke, || {
        if secrets.iter().any(|s| log.contains(s.as_str())) { Err("a secret is in the server log".to_string()) } else { Ok("clean".to_string()) }
    });
}

// -- restart

fn restart_stories(run: &mut Run, ctx: &mut Ctx) {
    let Some(mut first) = ctx.server.take() else { return };
    let was_running = first.stop();
    let log = first.log_text();
    let secrets = ctx.secrets.borrow().clone();
    run.check("boot", "the server was still running at the end of the run", Tag::Smoke, || {
        if was_running { Ok("yes".to_string()) } else { Err("it had stopped".to_string()) }
    });
    server_log_checks(run, &log, &secrets, "first run");
    if !run.wants(Tag::Release) {
        return;
    }
    let (bin, data) = (ctx.bin.clone(), ctx.data.clone());
    let data_s = data.to_string_lossy().into_owned();
    run.check("data", "`buildkiosk check` replays every journal after the run", Tag::Release, || {
        server::cli(&bin, &["check", "--data", data_s.as_str()]).map(|o| o.trim().to_string())
    });
    let log_path = ctx.work.join("server-2.log");
    let mut started = None;
    run.check("data", "the server starts again on the same data (behind HTTPS: --secure-cookies)", Tag::Release, || {
        let s = Server::start(&bin, &data, &["--secure-cookies"], &log_path)?;
        started = Some(s);
        Ok("started".to_string())
    });
    ctx.server = started;
    if ctx.server.is_none() {
        return;
    }
    let p = ctx.projects.first().copied().unwrap_or(1);
    let mut c = ctx.client("admin (after restart)");
    run.check("sign-in", "behind HTTPS the cookies are Secure and HSTS is sent", Tag::Release, || {
        let res = sign_in(&mut c, "ada@example.com", PASSWORD)?;
        let session = res.all("set-cookie").into_iter().find(|v| v.starts_with("bk_session=")).map(str::to_string).ok_or("no session cookie")?;
        if !session.contains("Secure") {
            return Err("the session cookie is not Secure".to_string());
        }
        if res.header("strict-transport-security").is_none() {
            return Err("no Strict-Transport-Security header".to_string());
        }
        Ok("Secure + HSTS".to_string())
    });
    run.check("data", "what the run made is still there after the restart", Tag::Release, || {
        let list = c.get(&format!("/projects/{p}/rfis"));
        if list.status == 200 && list.text().contains("QA regression: ceiling height at grid C") {
            Ok("the regression RFI is listed".to_string())
        } else {
            Err(format!("RFI list answered {}", list.status))
        }
    });
    if let Some(mut s) = ctx.server.take() {
        let was_running = s.stop();
        let log = s.log_text();
        let secrets = ctx.secrets.borrow().clone();
        run.check("boot", "the restarted server was still running at the end", Tag::Release, || {
            if was_running { Ok("yes".to_string()) } else { Err("it had stopped".to_string()) }
        });
        server_log_checks(run, &log, &secrets, "after the restart");
    }
}

// -- upgrade path

fn upgrade_story(run: &mut Run, ctx: &Ctx) {
    let name = "the previous release's data folder opens with this build";
    let prev = std::env::var_os("BK_REGRESS_PREV_BIN").map(PathBuf::from).filter(|p| p.is_file());
    let Some(prev) = prev else {
        run.skip("upgrade", name, Tag::Release, "no previous release program (BK_REGRESS_PREV_BIN)");
        return;
    };
    let dir = ctx.work.join("previous");
    let dir_s = dir.to_string_lossy().into_owned();
    let bin = ctx.bin.clone();
    let log_path = ctx.work.join("server-upgrade.log");
    let (log, secrets) = (ctx.log.clone(), ctx.secrets.clone());
    run.check("upgrade", name, Tag::Release, || {
        let version = server::cli(&prev, &["version"])?;
        server::cli(&prev, &["demo", "--data", dir_s.as_str()])?;
        server::cli(&bin, &["check", "--data", dir_s.as_str()])?;
        let mut s = Server::start(&bin, &dir, &[], &log_path)?;
        let mut c = Client::new(s.port, "admin (upgrade)", &log, &secrets);
        let res = sign_in(&mut c, "ada@example.com", PASSWORD)?;
        if !res.is_redirect() || !c.cookies.contains_key("bk_session") {
            return Err(format!("sign-in on the old data answered {}", res.status));
        }
        let home = c.get("/");
        let project = home.text().contains("Riverside Medical Office");
        let running = s.stop();
        if home.status == 200 && project && running {
            Ok(format!("from {}", version.trim()))
        } else {
            Err(format!("home {} project listed {project} running {running}", home.status))
        }
    });
}
