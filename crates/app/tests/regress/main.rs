//! The black-box regression suite (the QA bot, issue #3; `docs/qa.md`). It starts the real `buildkiosk` program on a
//! free port with a throwaway data folder and the fictional demo company, and drives it over HTTP like a person:
//! every role on every page, the main tool loops with the journal checked after each step, imports and exports,
//! safety checks and the upgrade path. It writes `regress-latest.md` and `regress-latest.json`.
//!
//!   BK_REGRESS=smoke    the fast checks (every pull request)
//!   BK_REGRESS=release  everything (nightly and release tags)
//!
//! Without `BK_REGRESS` it does nothing, so `cargo test --workspace` (the gate) stays as fast as before.
//! Other settings: `BK_REGRESS_OUT` (report folder, default `target/regress`), `BK_REGRESS_SKIPS` (default
//! `qa/known-skips.json`), `BK_REGRESS_PREVIOUS` (the last run's JSON, for "new failures"), `BK_REGRESS_BIN` (test
//! another build of the program), `BK_REGRESS_PREV_BIN` (the previous release, for the upgrade check).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

mod html;
mod http;
mod report;
mod server;
mod stories;

use std::path::PathBuf;

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).filter(|v| !v.is_empty()).map(PathBuf::from)
}

fn main() {
    let suite = match std::env::var("BK_REGRESS").unwrap_or_default().as_str() {
        "smoke" => "smoke",
        "release" => "release",
        "" => {
            println!("regress: not run (set BK_REGRESS=smoke or BK_REGRESS=release; see docs/qa.md)");
            return;
        }
        other => {
            println!("regress: BK_REGRESS={other} is not a suite (smoke or release)");
            std::process::exit(2);
        }
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out_dir = env_path("BK_REGRESS_OUT").unwrap_or_else(|| root.join("target/regress"));
    let skips = env_path("BK_REGRESS_SKIPS").unwrap_or_else(|| root.join("qa/known-skips.json"));
    let previous = env_path("BK_REGRESS_PREVIOUS").and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| serde_json::from_str(&t).ok());
    println!("regress: {suite} suite against {}", server::bin().display());
    let mut run = report::Run::new(suite);
    let requests = stories::all(&mut run, &root);
    let inputs = report::Inputs { out_dir, skips: report::load_skips(&skips), previous, requests };
    if !report::finish(&run, &inputs) {
        std::process::exit(1);
    }
}
