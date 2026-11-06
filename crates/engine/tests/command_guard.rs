//! A host's `CommandGuard`: asked before every command that can change something, never for queries; a refusal is a
//! `Forbidden` error and changes nothing.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]

use std::sync::{Arc, Mutex};

use buildkiosk_engine::model::CompanyState;
use buildkiosk_engine::{Actor, CommandGuard, EngineError};
use buildkiosk_testkit::World;
use serde_json::{Value, json};

/// Refuses `project.create` once the company has `max` projects, and remembers what it was asked.
struct ProjectLimit {
    max: usize,
    asked: Mutex<Vec<(String, String)>>,
}

impl CommandGuard for ProjectLimit {
    fn check(&self, company: &str, state: &CompanyState, _actor: Actor, command: &str, _params: &Value) -> Result<(), String> {
        self.asked.lock().unwrap().push((company.to_string(), command.to_string()));
        if command == "project.create" && state.projects.len() >= self.max {
            return Err(format!("your plan allows {} projects", self.max));
        }
        Ok(())
    }
}

struct Panics;

impl CommandGuard for Panics {
    fn check(&self, _: &str, _: &CompanyState, _: Actor, _: &str, _: &Value) -> Result<(), String> {
        panic!("a bug in the guard")
    }
}

#[test]
fn a_refusal_stops_the_command_and_changes_nothing() {
    let mut w = World::new();
    let limit = Arc::new(ProjectLimit { max: 1, asked: Mutex::new(Vec::new()) });
    w.engine.set_command_guard(Some(limit.clone()));
    let admin = w.admin();

    w.ok(admin, "project.create", json!({"name": "Tower One"}));
    let entries = w.journal.entries.len();
    let e = w.err(admin, "project.create", json!({"name": "Tower Two"}));
    assert!(matches!(&e, EngineError::Forbidden(m) if m == "your plan allows 1 projects"), "{e}");
    assert_eq!(e.http_status(), 403);
    assert_eq!(w.engine.state().projects.len(), 1);
    assert_eq!(w.journal.entries.len(), entries, "nothing journaled");

    // Queries aren't asked; every changing command is, with the company's slug.
    let asked = limit.asked.lock().unwrap().len();
    w.ok(admin, "project.list", json!({}));
    assert_eq!(limit.asked.lock().unwrap().len(), asked);
    let (company, command) = limit.asked.lock().unwrap().last().cloned().unwrap();
    assert_eq!(command, "project.create");
    assert_eq!(Some(company.as_str()), w.engine.state().company().map(|c| c.slug.as_str()));

    // A guard that panics refuses with an internal error instead of taking the program down.
    w.engine.set_command_guard(Some(Arc::new(Panics)));
    assert!(matches!(w.err(admin, "project.create", json!({"name": "Tower Three"})), EngineError::Internal { .. }));
    // Without a guard, everything runs as before.
    w.engine.set_command_guard(None);
    w.ok(admin, "project.create", json!({"name": "Tower Three"}));
    assert_eq!(w.engine.state().projects.len(), 2);
}
