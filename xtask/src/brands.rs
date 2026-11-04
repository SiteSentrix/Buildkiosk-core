//! `cargo xtask brands`: no other vendor's product or company names in what users and agents read.
//!
//! BuildKiosk names its own features and never another vendor's. Scanned: string literals in Rust
//! sources under `crates/` and `apps/` (page text, command labels and params docs, MCP descriptions),
//! skipping comments and test code; the `description` of every `Cargo.toml`; every Markdown file
//! (README, ROADMAP, AGENTS, CONTRIBUTING, docs/*.md); `NOTICE`; everything under `.github/` and
//! `packaging/`; and non-Markdown files under `docs/`.
//!
//! A line that must keep a name (an integration that has to say what it connects to) says
//! `brand-ok` in a comment on that line.

use std::path::Path;

use crate::scan;

/// Vendor product and company names, lowercase, matched case-insensitively as whole words.
const DENY: &[&str] = &[
    "procore",
    "autodesk",
    "plangrid",
    "bluebeam",
    "buildertrend",
    "aconex",
    "primavera",
    "textura",
    "fieldwire",
    "revit",
    "navisworks",
    "bim 360",
    "newforma",
    "raken",
    "jobtread",
    "buildxact",
    "houzz",
    "coconstruct",
    "trimble",
    "sage",
    "viewpoint",
    "oracle",
    // The predecessor app's demo used a real company's name; never again.
    "bechtel",
];

const ALLOW: &str = "brand-ok";

/// This file lists the names it rejects.
const SELF: &str = "xtask/src/brands.rs";

pub fn run(root: &Path) -> Result<(), String> {
    let files = crate::repo_files(root)?;
    let mut hits = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(root.join(path)) else { continue };
        let lines: Vec<&str> = text.lines().collect();
        let allowed = |line: usize| lines.get(line.saturating_sub(1)).is_some_and(|l| l.contains(ALLOW));
        if is_rust_to_scan(path) {
            for lit in scan::string_literals(&text) {
                if allowed(lit.line) {
                    continue;
                }
                for term in scan::terms_in(&lit.text, DENY) {
                    hits.push(format!("{path}:{}: \"{term}\" in \"{}\"", lit.line, short(&lit.text)));
                }
            }
        } else if path == "Cargo.toml" || path.ends_with("/Cargo.toml") {
            for (n, l) in lines.iter().enumerate() {
                if l.trim_start().starts_with("description") && !l.contains(ALLOW) {
                    for term in scan::terms_in(l, DENY) {
                        hits.push(format!("{path}:{}: \"{term}\" in the package description", n + 1));
                    }
                }
            }
        } else if is_text_to_scan(path) {
            for (n, l) in lines.iter().enumerate() {
                if !l.contains(ALLOW) {
                    for term in scan::terms_in(l, DENY) {
                        hits.push(format!("{path}:{}: \"{term}\"", n + 1));
                    }
                }
            }
        }
    }
    if hits.is_empty() {
        println!("brands: no vendor names in user-visible text");
        Ok(())
    } else {
        Err(format!(
            "{} vendor name(s) in user-visible text (say \"the reference product\" or name the feature; mark a required integration name `{ALLOW}`):\n  {}",
            hits.len(),
            hits.join("\n  ")
        ))
    }
}

fn is_rust_to_scan(path: &str) -> bool {
    if !path.ends_with(".rs") || path == SELF || !(path.starts_with("crates/") || path.starts_with("apps/")) {
        return false;
    }
    let mut parts = path.split('/').rev();
    let file = parts.next().unwrap_or_default();
    let test_file = file == "tests.rs" || file.starts_with("tests_") || file.ends_with("_tests.rs");
    let test_dir = parts.any(|d| d == "tests" || d == "benches" || d == "testkit");
    !test_file && !test_dir
}

/// Plain-text files scanned line by line (everything people read about the project).
fn is_text_to_scan(path: &str) -> bool {
    path.ends_with(".md") || path == "NOTICE" || path.starts_with(".github/") || path.starts_with("packaging/") || path.starts_with("docs/")
}

fn short(s: &str) -> String {
    let one_line: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if one_line.chars().count() > 80 { format!("{}…", one_line.chars().take(80).collect::<String>()) } else { one_line }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_and_xtask_are_not_scanned() {
        assert!(is_rust_to_scan("crates/web/src/pages/rfi.rs"));
        assert!(!is_rust_to_scan("crates/engine/tests/rfi.rs"));
        assert!(!is_rust_to_scan("crates/testkit/src/lib.rs"));
        assert!(!is_rust_to_scan("xtask/src/brands.rs"));
        assert!(!is_rust_to_scan("crates/web/src/style.css"));
    }

    #[test]
    fn markdown_and_notices_are_scanned() {
        assert!(is_text_to_scan("README.md"));
        assert!(is_text_to_scan("docs/gaps.md"));
        assert!(is_text_to_scan("NOTICE"));
        assert!(is_text_to_scan(".github/pull_request_template.md"));
        assert!(is_text_to_scan(".github/workflows/ci.yml"));
        assert!(!is_text_to_scan("crates/web/src/pages/rfi.rs"));
        assert!(!is_text_to_scan("Cargo.lock"));
    }

    #[test]
    fn new_vendor_names_are_denied() {
        for name in ["JobTread", "Buildxact", "Houzz", "CoConstruct", "Trimble", "Sage", "Viewpoint", "Oracle"] {
            assert_eq!(scan::terms_in(&format!("works like {name} does"), DENY).len(), 1, "{name}");
        }
        assert!(scan::terms_in("usage and viewpoints", DENY).is_empty());
    }
}
