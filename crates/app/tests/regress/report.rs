//! Checks, the known-skips list, the gate's decision and the report (`regress-latest.md` and `.json`).

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::http::Seen;

/// Longest a known-skip may stay on the list (days from today).
const MAX_SKIP_DAYS: i64 = 14;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Pass,
    Fail,
    Skip,
}

impl Outcome {
    fn word(self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::Fail => "fail",
            Outcome::Skip => "skip",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// Fast checks, run on every pull request (and in the release suite).
    Smoke,
    /// Everything else, run nightly and for a release tag.
    Release,
}

impl Tag {
    fn word(self) -> &'static str {
        match self {
            Tag::Smoke => "@smoke",
            Tag::Release => "@release",
        }
    }
}

pub struct Check {
    pub area: String,
    pub name: String,
    pub tag: Tag,
    pub outcome: Outcome,
    pub detail: String,
    pub ms: u128,
}

impl Check {
    pub fn id(&self) -> String {
        format!("{} › {}", self.area, self.name)
    }
}

pub struct Run {
    pub suite: String,
    pub checks: Vec<Check>,
    started: Instant,
}

impl Run {
    pub fn new(suite: &str) -> Run {
        Run { suite: suite.to_string(), checks: Vec::new(), started: Instant::now() }
    }

    pub fn wants(&self, tag: Tag) -> bool {
        self.suite == "release" || tag == Tag::Smoke
    }

    /// Runs one check (if this suite includes its tag): `Ok(detail)` passes, `Err(detail)` or a panic fails.
    /// Returns whether it ran and passed.
    pub fn check(&mut self, area: &str, name: &str, tag: Tag, f: impl FnOnce() -> Result<String, String>) -> bool {
        if !self.wants(tag) {
            return false;
        }
        let start = Instant::now();
        let result = catch_unwind(AssertUnwindSafe(f));
        let ms = start.elapsed().as_millis();
        let (outcome, detail) = match result {
            Ok(Ok(d)) => (Outcome::Pass, d),
            Ok(Err(d)) => (Outcome::Fail, d),
            Err(p) => {
                let msg = p.downcast_ref::<String>().cloned().or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_string())).unwrap_or_default();
                (Outcome::Fail, format!("the check itself stopped: {msg}"))
            }
        };
        println!("{:<5} {area} › {name} ({ms} ms){}", outcome.word(), if outcome == Outcome::Pass { String::new() } else { format!(": {detail}") });
        self.checks.push(Check { area: area.to_string(), name: name.to_string(), tag, outcome, detail, ms });
        outcome == Outcome::Pass
    }

    /// Records a check that could not run here.
    pub fn skip(&mut self, area: &str, name: &str, tag: Tag, reason: &str) {
        if !self.wants(tag) {
            return;
        }
        println!("skip  {area} › {name}: {reason}");
        self.checks.push(Check { area: area.to_string(), name: name.to_string(), tag, outcome: Outcome::Skip, detail: reason.to_string(), ms: 0 });
    }
}

// -- dates (UTC, no dependency)

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { yoe + era * 400 + 1 } else { yoe + era * 400 }, m, d)
}

fn now_secs() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| i64::try_from(d.as_secs()).unwrap_or(0)).unwrap_or(0)
}

pub fn today_days() -> i64 {
    now_secs().div_euclid(86_400)
}

/// `YYYY-MM-DD` as days since 1970-01-01.
pub fn parse_day(s: &str) -> Option<i64> {
    let mut parts = s.trim().split('-').map(|p| p.parse::<i64>().ok());
    let (y, m, d) = (parts.next()??, parts.next()??, parts.next()??);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

fn now_iso() -> String {
    let secs = now_secs();
    let (y, m, d) = civil_from_days(secs.div_euclid(86_400));
    let t = secs.rem_euclid(86_400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", t / 3600, (t / 60) % 60, t % 60)
}

// -- known skips

pub struct KnownSkip {
    pub check: String,
    pub reason: String,
    pub expires: String,
    pub issue: Option<u64>,
    /// A failing check that is ignored until it expires (a filed bug).
    pub quarantine: bool,
}

pub fn load_skips(path: &Path) -> Result<Vec<KnownSkip>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let list = v.get("skips").and_then(Value::as_array).ok_or_else(|| format!("{}: no `skips` list", path.display()))?;
    let mut out = Vec::new();
    for s in list {
        let text = |k: &str| s.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        out.push(KnownSkip {
            check: text("check"),
            reason: text("reason"),
            expires: text("expires"),
            issue: s.get("issue").and_then(Value::as_u64),
            quarantine: s.get("quarantine").and_then(Value::as_bool).unwrap_or(false),
        });
    }
    Ok(out)
}

// -- the report

pub struct Inputs {
    pub out_dir: PathBuf,
    pub skips: Result<Vec<KnownSkip>, String>,
    pub previous: Option<Value>,
    pub requests: Vec<Seen>,
}

fn cell(s: &str) -> String {
    let one: String = s.split_whitespace().collect::<Vec<_>>().join(" ").replace('|', "\\|");
    if one.chars().count() > 400 { format!("{}…", one.chars().take(400).collect::<String>()) } else { one }
}

/// Writes the report and decides the gate; true when it passes.
pub fn finish(run: &Run, inputs: &Inputs) -> bool {
    let today = today_days();
    let mut problems: Vec<String> = Vec::new();
    let skips: &[KnownSkip] = match &inputs.skips {
        Ok(s) => s,
        Err(e) => {
            problems.push(format!("the known-skips list cannot be read: {e}"));
            &[]
        }
    };
    for s in skips {
        match parse_day(&s.expires) {
            None => problems.push(format!("known-skip `{}` has no valid `expires` date (YYYY-MM-DD)", s.check)),
            Some(d) if d < today => problems.push(format!("known-skip `{}` expired on {}: fix the check or renew it with a reason", s.check, s.expires)),
            Some(d) if d > today + MAX_SKIP_DAYS => {
                problems.push(format!("known-skip `{}` expires {}, more than {MAX_SKIP_DAYS} days away", s.check, s.expires))
            }
            Some(_) => {}
        }
        if s.reason.trim().is_empty() {
            problems.push(format!("known-skip `{}` has no reason", s.check));
        }
    }
    let listed = |id: &str| skips.iter().find(|s| s.check == id);
    // Each check: its result as the gate sees it ("known" = a quarantined failure).
    let mut rows: Vec<(&Check, &'static str, String)> = Vec::new();
    for c in &run.checks {
        let id = c.id();
        let entry = listed(&id);
        let (word, note) = match (c.outcome, entry) {
            (Outcome::Pass, Some(_)) => ("pass", "passes now: take it off the known-skips list".to_string()),
            (Outcome::Pass, None) => ("pass", String::new()),
            (Outcome::Fail, Some(s)) if s.quarantine => ("known", known_note(s)),
            (Outcome::Fail, _) => {
                problems.push(format!("FAIL {id}"));
                ("fail", String::new())
            }
            (Outcome::Skip, Some(s)) => ("skip", known_note(s)),
            (Outcome::Skip, None) => {
                problems.push(format!("{id} was skipped but is not on the known-skips list"));
                ("skip", String::new())
            }
        };
        rows.push((c, word, note));
    }
    let gate = problems.is_empty();

    // New failures since the last run of this suite.
    let previous_failing: Option<Vec<String>> = inputs.previous.as_ref().map(|p| {
        p.get("checks")
            .and_then(Value::as_array)
            .map(|l| {
                l.iter()
                    .filter(|c| matches!(c.get("result").and_then(Value::as_str), Some("fail" | "known")))
                    .filter_map(|c| c.get("id").and_then(Value::as_str).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    });
    let failing_now: Vec<String> = rows.iter().filter(|(_, w, _)| *w == "fail" || *w == "known").map(|(c, _, _)| c.id()).collect();

    let count = |w: &str| rows.iter().filter(|(_, x, _)| *x == w).count();
    let commit = std::env::var("GITHUB_SHA").map(|s| s.chars().take(12).collect::<String>()).unwrap_or_else(|_| "local".to_string());
    let secs = run.started.elapsed().as_secs();
    let mut md = String::new();
    md.push_str(&format!("# QA latest: BuildKiosk regression ({})\n\n", run.suite));
    md.push_str(&format!("Generated {} · commit {commit} · {} suite · {secs} s\n\n", now_iso(), run.suite));
    md.push_str(
        "The real `buildkiosk` program, started on a throwaway data folder with the fictional demo company and driven over \
         HTTP like a person (sign in, pages, forms, uploads, downloads). No production data; no secrets printed.\n\n",
    );
    md.push_str(&format!(
        "**Gate: {}.** {} passed, {} failed, {} known (quarantined, filed), {} skipped.\n\n",
        if gate { "PASS" } else { "FAIL" },
        count("pass"),
        count("fail"),
        count("known"),
        count("skip")
    ));
    if !problems.is_empty() {
        md.push_str("## What fails the gate\n\n");
        for p in &problems {
            md.push_str(&format!("- {}\n", cell(p)));
        }
        md.push('\n');
    }
    md.push_str("## New failures since the last run\n\n");
    match &previous_failing {
        None => md.push_str("No earlier report to compare with.\n\n"),
        Some(before) => {
            let new: Vec<&String> = failing_now.iter().filter(|id| !before.contains(id)).collect();
            let fixed: Vec<&String> = before.iter().filter(|id| !failing_now.contains(id)).collect();
            if new.is_empty() {
                md.push_str("None.\n");
            }
            for id in new {
                md.push_str(&format!("- NEW: {id}\n"));
            }
            for id in fixed {
                md.push_str(&format!("- fixed since: {id}\n"));
            }
            md.push('\n');
        }
    }
    md.push_str("## By area\n\n| Area | Pass | Fail | Known | Skip |\n|---|---|---|---|---|\n");
    let mut areas: BTreeMap<&str, [usize; 4]> = BTreeMap::new();
    for (c, w, _) in &rows {
        let slot = match *w {
            "pass" => 0,
            "fail" => 1,
            "known" => 2,
            _ => 3,
        };
        areas.entry(c.area.as_str()).or_default()[slot] += 1;
    }
    for (a, n) in &areas {
        md.push_str(&format!("| {a} | {} | {} | {} | {} |\n", n[0], n[1], n[2], n[3]));
    }
    md.push_str("\n## Slowest checks\n\n| Check | Time |\n|---|---|\n");
    let mut slow: Vec<&Check> = run.checks.iter().collect();
    slow.sort_by_key(|c| std::cmp::Reverse(c.ms));
    for c in slow.iter().take(8) {
        md.push_str(&format!("| {} | {:.1} s |\n", cell(&c.id()), ms_to_s(c.ms)));
    }
    md.push_str(&format!("\n## Slowest requests (of {})\n\n| Request | Status | Time |\n|---|---|---|\n", inputs.requests.len()));
    let mut reqs: Vec<&Seen> = inputs.requests.iter().collect();
    reqs.sort_by_key(|r| std::cmp::Reverse(r.ms));
    for r in reqs.iter().take(8) {
        md.push_str(&format!("| {} {} ({}) | {} | {:.1} s |\n", r.method, cell(&r.path), r.who, r.status, ms_to_s(r.ms)));
    }
    md.push_str("\n## Every check\n\n| Area | Check | Tag | Result | Time | Detail |\n|---|---|---|---|---|---|\n");
    for (c, w, note) in &rows {
        let detail = if note.is_empty() { c.detail.clone() } else { format!("{} ({note})", c.detail) };
        md.push_str(&format!("| {} | {} | {} | {w} | {:.1} s | {} |\n", c.area, cell(&c.name), c.tag.word(), ms_to_s(c.ms), cell(&detail)));
    }
    md.push_str("\nKnown-failing checks live in `qa/known-skips.json` (reason, issue, expiry of at most two weeks); see `docs/qa.md`.\n");

    let checks: Vec<Value> = rows
        .iter()
        .map(|(c, w, note)| json!({"id": c.id(), "area": c.area, "name": c.name, "tag": c.tag.word(), "result": w, "ms": c.ms, "detail": c.detail, "note": note}))
        .collect();
    let report = json!({
        "suite": run.suite,
        "generated": now_iso(),
        "commit": commit,
        "gate": if gate { "pass" } else { "fail" },
        "problems": problems,
        "requests": inputs.requests.len(),
        "checks": checks,
    });
    let _ = std::fs::create_dir_all(&inputs.out_dir);
    let md_path = inputs.out_dir.join("regress-latest.md");
    let json_path = inputs.out_dir.join("regress-latest.json");
    if let Err(e) = std::fs::write(&md_path, &md) {
        println!("could not write {}: {e}", md_path.display());
    }
    if let Err(e) = std::fs::write(&json_path, serde_json::to_string_pretty(&report).unwrap_or_default()) {
        println!("could not write {}: {e}", json_path.display());
    }
    println!("\nreport: {}", md_path.display());
    println!(
        "gate: {} ({} passed, {} failed, {} known, {} skipped)",
        if gate { "PASS" } else { "FAIL" },
        count("pass"),
        count("fail"),
        count("known"),
        count("skip")
    );
    for p in &problems {
        println!("  {p}");
    }
    gate
}

fn known_note(s: &KnownSkip) -> String {
    match s.issue {
        Some(n) => format!("known: #{n}, until {}: {}", s.expires, s.reason),
        None => format!("known until {}: {}", s.expires, s.reason),
    }
}

fn ms_to_s(ms: u128) -> f64 {
    // Display only: a check never runs long enough for the conversion to lose anything that matters.
    u32::try_from(ms).map(f64::from).unwrap_or(f64::MAX) / 1000.0
}
