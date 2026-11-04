//! The tools of the black-box regression suite (`crates/app/tests/regress`, `docs/qa.md`), as a library so other
//! suites built on the core write the same report and follow the same known-skips rules instead of keeping a copy:
//!
//! - [`report`]: checks tagged `@smoke` / `@release`, the known-skips list (`qa/known-skips.json`, each entry with a
//!   reason and an expiry at most two weeks away), the gate's decision and the report (`regress-latest.md` and
//!   `.json`).
//! - [`http`]: a small HTTP/1.1 client on `std::net` with a cookie jar and a shared request log.
//! - [`html`]: the links and forms of a server-rendered page.
//! - [`server`]: starts the program under test on a free port with a throwaway data folder.
//!
//! Test tooling only (like `buildkiosk-testkit`): never a dependency of the shipped program.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

pub mod html;
pub mod http;
pub mod report;
pub mod server;

#[cfg(test)]
mod tests {
    use super::report::{self, Inputs, Run, Tag};

    fn temp(name: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("buildkiosk-regress-kit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    fn day(offset: i64) -> String {
        let (y, m, d) = report::civil_from_days(report::today_days() + offset);
        format!("{y:04}-{m:02}-{d:02}")
    }

    fn skips(dir: &std::path::Path, json: &str) -> Result<Vec<report::KnownSkip>, String> {
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.join("known-skips.json");
        std::fs::write(&path, json).unwrap();
        report::load_skips(&path)
    }

    #[test]
    fn dates_round_trip() {
        assert_eq!(report::parse_day("1970-01-01"), Some(0));
        assert_eq!(report::parse_day("2024-02-29").map(report::civil_from_days), Some((2024, 2, 29)));
        assert_eq!(report::parse_day("2024-13-01"), None);
        assert_eq!(report::parse_day("soon"), None);
    }

    #[test]
    fn the_gate_follows_the_known_skips_rules() {
        let dir = temp("gate");
        let mut run = Run::new("smoke");
        assert!(run.check("pages", "home", Tag::Smoke, || Ok("200".into())));
        assert!(!run.check("pages", "broken", Tag::Smoke, || Err("500".into())));
        assert!(!run.check("pages", "nightly", Tag::Release, || Ok("not run in smoke".into())));
        assert_eq!(run.checks.len(), 2, "a @release check does not run in the smoke suite");

        // A filed failure, quarantined until next week: the gate passes and the report says why.
        let ok =
            format!(r#"{{"skips": [{{"check": "pages › broken", "reason": "filed", "issue": 7, "expires": "{}", "quarantine": true}}]}}"#, day(7));
        let inputs = Inputs { out_dir: dir.clone(), skips: skips(&dir, &ok), previous: None, requests: Vec::new() };
        assert!(report::finish(&run, &inputs));
        let md = std::fs::read_to_string(dir.join("regress-latest.md")).unwrap();
        assert!(md.contains("**Gate: PASS.**") && md.contains("known: #7"), "{md}");
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("regress-latest.json")).unwrap()).unwrap();
        assert_eq!(json["gate"], "pass");

        // Not on the list, expired, or more than two weeks away: the gate fails.
        for bad in [
            r#"{"skips": []}"#.to_string(),
            format!(r#"{{"skips": [{{"check": "pages › broken", "reason": "filed", "expires": "{}", "quarantine": true}}]}}"#, day(-1)),
            format!(r#"{{"skips": [{{"check": "pages › broken", "reason": "filed", "expires": "{}", "quarantine": true}}]}}"#, day(30)),
            format!(r#"{{"skips": [{{"check": "pages › broken", "reason": " ", "expires": "{}", "quarantine": true}}]}}"#, day(7)),
        ] {
            let inputs = Inputs { out_dir: dir.clone(), skips: skips(&dir, &bad), previous: None, requests: Vec::new() };
            assert!(!report::finish(&run, &inputs), "{bad}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
