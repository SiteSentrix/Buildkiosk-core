//! Choosing the company for each request.
//!
//! Every request goes through a [`TenantResolver`] first: it names the company from the `Host` header, and the
//! request is handled by that company's own [`App`] (its own state, journal, files, credentials, sessions and
//! drafts). The default, [`SingleCompany`], sends every request to the one company the server was started with,
//! so a single-company server behaves exactly as before. Another build can pass its own resolver and a way to
//! open companies the first time a request names them.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header::HOST;
use axum::response::Response;
use buildkiosk_core::ext::{SingleCompany, TenantResolver};
use tower::ServiceExt;
use tower_http::catch_panic::CatchPanicLayer;

use crate::{App, Shared, plain, router, start_jobs};

/// Opens a company the first time a request names it: `Ok(None)` when there is no such company. Called off the
/// async runtime (it may read the company's journal from disk).
pub type Opener = Box<dyn Fn(&str) -> Result<Option<App>, String> + Send + Sync>;

/// Most companies one server keeps open.
const MAX_TENANTS: usize = 10_000;

/// The companies a server answers for, and how it picks one per request.
pub struct Tenants {
    resolver: Box<dyn TenantResolver>,
    open: Option<Opener>,
    /// Companies opened so far, by slug, each with its own pages.
    loaded: RwLock<HashMap<String, (Shared, Router)>>,
    /// Set once the server is running: companies opened after that start their timed work straight away.
    serving: AtomicBool,
}

impl Tenants {
    /// One company (or first-run setup): every request goes to `app`, whatever its host.
    pub fn single(app: Shared) -> Tenants {
        // Before first-run setup there is no slug yet; the empty key keeps pointing at the same app after it.
        let slug = app.company().unwrap_or_default();
        let tenants = Tenants::new(Box::new(SingleCompany { slug: slug.clone() }), None);
        tenants.insert(&slug, app);
        tenants
    }

    /// Companies chosen by `resolver`. Companies not added with [`Tenants::insert`] are opened with `open` the first
    /// time a request names them; without it, a request for a company not added answers 404.
    pub fn new(resolver: Box<dyn TenantResolver>, open: Option<Opener>) -> Tenants {
        Tenants { resolver, open, loaded: RwLock::new(HashMap::new()), serving: AtomicBool::new(false) }
    }

    /// Adds (or replaces) the app that serves `slug`.
    pub fn insert(&self, slug: &str, app: Shared) {
        let pages = router(app.clone());
        if self.serving.load(Ordering::SeqCst) {
            start_jobs(&app);
        }
        self.loaded.write().unwrap_or_else(|e| e.into_inner()).insert(slug.to_string(), (app, pages));
    }

    /// Closes `slug` without a restart (for example after the company was deleted or restored on disk): its timed work
    /// stops and the next request for it opens it again (with the opener) or answers 404. Requests already running
    /// finish on the app returned here. `None` when it wasn't open.
    pub fn remove(&self, slug: &str) -> Option<Shared> {
        let (app, _) = self.loaded.write().unwrap_or_else(|e| e.into_inner()).remove(slug)?;
        app.stop_jobs();
        Some(app)
    }

    /// The company a request to `host` belongs to, as the resolver names it.
    pub fn company_for(&self, host: Option<&str>) -> Option<String> {
        self.resolver.company_for(host)
    }

    /// The app serving `slug`, if it is open.
    pub fn get(&self, slug: &str) -> Option<Shared> {
        self.loaded_pages(slug).map(|(app, _)| app)
    }

    fn loaded_pages(&self, slug: &str) -> Option<(Shared, Router)> {
        self.loaded.read().unwrap_or_else(|e| e.into_inner()).get(slug).cloned()
    }

    /// The app and pages for `slug`, opening the company if needed. Blocking: may read from disk.
    fn open_pages(&self, slug: &str) -> Result<Option<(Shared, Router)>, String> {
        if let Some(found) = self.loaded_pages(slug) {
            return Ok(Some(found));
        }
        let Some(open) = &self.open else { return Ok(None) };
        if self.loaded.read().unwrap_or_else(|e| e.into_inner()).len() >= MAX_TENANTS {
            return Err("this server has as many companies open as it can hold".into());
        }
        let Some(app) = open(slug)? else { return Ok(None) };
        let app = Arc::new(app);
        let pages = router(app.clone());
        let mut loaded = self.loaded.write().unwrap_or_else(|e| e.into_inner());
        // Two first requests at once: keep the one that got here first.
        if let Some(found) = loaded.get(slug) {
            return Ok(Some(found.clone()));
        }
        loaded.insert(slug.to_string(), (app.clone(), pages.clone()));
        drop(loaded);
        if self.serving.load(Ordering::SeqCst) {
            start_jobs(&app);
        }
        Ok(Some((app, pages)))
    }

    /// Starts the timed work of every open company, and of every company opened from now on. Needs a running Tokio
    /// runtime.
    pub fn start_jobs(&self) {
        if self.serving.swap(true, Ordering::SeqCst) {
            return;
        }
        let apps: Vec<Shared> = self.loaded.read().unwrap_or_else(|e| e.into_inner()).values().map(|(app, _)| app.clone()).collect();
        for app in &apps {
            start_jobs(app);
        }
    }

    /// The router that resolves the company of each request and hands it to that company's pages.
    pub fn into_router(self: Arc<Self>) -> Router {
        Router::new().fallback(dispatch).layer(CatchPanicLayer::new()).with_state(self)
    }
}

/// The host a request was sent to: the `Host` header, else the authority in the request line.
fn host_of(req: &Request) -> Option<String> {
    req.headers().get(HOST).and_then(|v| v.to_str().ok()).map(str::to_string).or_else(|| req.uri().authority().map(|a| a.as_str().to_string()))
}

async fn dispatch(State(tenants): State<Arc<Tenants>>, req: Request) -> Response {
    let host = host_of(&req);
    let Some(slug) = tenants.company_for(host.as_deref()) else { return nobody_here() };
    let found = match tenants.loaded_pages(&slug) {
        Some(found) => Some(found),
        None => {
            let t = tenants.clone();
            match tokio::task::spawn_blocking(move || t.open_pages(&slug)).await {
                Ok(Ok(found)) => found,
                Ok(Err(e)) => {
                    log::warn!("could not open a company: {e}");
                    return plain(StatusCode::SERVICE_UNAVAILABLE, "This company can't be opened right now. Try again in a moment.");
                }
                Err(e) => {
                    log::warn!("opening a company stopped: {e}");
                    return plain(StatusCode::SERVICE_UNAVAILABLE, "This company can't be opened right now. Try again in a moment.");
                }
            }
        }
    };
    let Some((_, pages)) = found else { return nobody_here() };
    match pages.oneshot(req).await {
        Ok(res) => res,
        Err(never) => match never {},
    }
}

fn nobody_here() -> Response {
    plain(StatusCode::NOT_FOUND, "No company is served at this address.")
}
