//! The list of the web server's routes, kept by the same calls that register them.
//!
//! Every route goes through [`Routes::route`], which adds it to the axum router and to the list at once, so
//! [`crate::route_patterns`] can't drift from what [`crate::router`] serves. Tools outside the server (tests that
//! visit every page, for example) read the list instead of the source.

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::handler::Handler;
use axum::routing::MethodRouter;

use super::Shared;

/// One route the server answers: its pattern (with `{name}` for each path parameter, as in
/// `/projects/{id}/rfis/{rid}`) and the HTTP methods it accepts. A route that accepts `GET` also answers `HEAD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePattern {
    pub path: String,
    pub methods: Vec<&'static str>,
}

/// A router being built, with the list of what was registered so far.
pub(crate) struct Routes {
    router: Router<Shared>,
    list: Vec<RoutePattern>,
}

impl Routes {
    pub(crate) fn new() -> Self {
        Self { router: Router::new(), list: Vec::new() }
    }

    pub(crate) fn route(mut self, path: &str, method: Method) -> Self {
        self.list.push(RoutePattern { path: path.to_owned(), methods: method.methods });
        self.router = self.router.route(path, method.inner);
        self
    }

    pub(crate) fn into_router(self) -> Router<Shared> {
        self.router
    }

    pub(crate) fn into_list(self) -> Vec<RoutePattern> {
        self.list
    }
}

/// The handlers of one route, by method, and the names of those methods.
pub(crate) struct Method {
    inner: MethodRouter<Shared>,
    methods: Vec<&'static str>,
}

/// A route answering `GET` (and `HEAD`), like `axum::routing::get`.
pub(crate) fn get<H, T>(handler: H) -> Method
where
    H: Handler<T, Shared>,
    T: 'static,
{
    Method { inner: axum::routing::get(handler), methods: vec!["GET"] }
}

/// A route answering `POST`, like `axum::routing::post`.
pub(crate) fn post<H, T>(handler: H) -> Method
where
    H: Handler<T, Shared>,
    T: 'static,
{
    Method { inner: axum::routing::post(handler), methods: vec!["POST"] }
}

impl Method {
    pub(crate) fn get<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, Shared>,
        T: 'static,
    {
        self.inner = self.inner.get(handler);
        self.methods.push("GET");
        self
    }

    pub(crate) fn post<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, Shared>,
        T: 'static,
    {
        self.inner = self.inner.post(handler);
        self.methods.push("POST");
        self
    }

    pub(crate) fn delete<H, T>(mut self, handler: H) -> Self
    where
        H: Handler<T, Shared>,
        T: 'static,
    {
        self.inner = self.inner.delete(handler);
        self.methods.push("DELETE");
        self
    }

    /// A body size limit for this route (uploads allow more than the default).
    pub(crate) fn layer(mut self, limit: DefaultBodyLimit) -> Self {
        self.inner = self.inner.layer(limit);
        self
    }
}
