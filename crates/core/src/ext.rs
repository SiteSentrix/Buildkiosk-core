//! Extension points: small traits that let another build add behaviour around the single-company
//! product without forking it.
//!
//! Each trait has a default implementation here that does exactly what BuildKiosk does today (one
//! company per server, local passwords, local delivery), so nothing changes unless a caller passes a
//! different implementation. The traits are pure (no I/O in this crate): the server, store and mail
//! code call them and do the I/O. Which call sites use which trait, and which are still to be wired,
//! is listed in `docs/core-vs-enterprise.md`.

use crate::Timestamp;

/// Which company a request belongs to.
///
/// The server today serves one company. A resolver lets a build choose the company per request (for
/// example from the `Host` header) without the rest of the server knowing how.
pub trait TenantResolver: Send + Sync {
    /// The company slug for a request to `host` (the `Host` header, if any), or `None` when no company
    /// answers there.
    fn company_for(&self, host: Option<&str>) -> Option<String>;
}

/// The default: every request goes to the one company this server was started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleCompany {
    pub slug: String,
}

impl TenantResolver for SingleCompany {
    fn company_for(&self, _host: Option<&str>) -> Option<String> {
        Some(self.slug.clone())
    }
}

/// A sign-in attempt, as typed by the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignIn<'a> {
    pub company: &'a str,
    pub email: &'a str,
    pub secret: &'a str,
}

/// What an [`Authenticator`] decided about a sign-in attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthDecision {
    /// Check the company's own stored password, as today.
    Local,
    /// Signed in as this person (the raw person id in that company).
    Accept(u64),
    /// Refused, without checking the local password.
    Reject,
}

/// A pluggable sign-in check that runs before the local password check.
pub trait Authenticator: Send + Sync {
    fn authenticate(&self, attempt: &SignIn<'_>) -> AuthDecision;
}

/// The default: local passwords only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalOnly;

impl Authenticator for LocalOnly {
    fn authenticate(&self, _attempt: &SignIn<'_>) -> AuthDecision {
        AuthDecision::Local
    }
}

/// An invitation that a person accepted (from an emailed link).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedInvite {
    pub company: String,
    pub email: String,
    /// The project the invitation was for, if it was for one.
    pub project: Option<u64>,
}

/// What happens when an invitation is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AcceptDecision {
    /// The person sets a password for this company (the local account).
    LocalAccount,
    /// The person is linked to an account kept elsewhere, named by this key.
    LinkAccount(String),
}

/// A pluggable accept step for invitations.
pub trait InviteAccept: Send + Sync {
    fn on_accept(&self, invite: &AcceptedInvite) -> AcceptDecision;
}

/// The default: an accepted invitation makes a local account.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalAccounts;

impl InviteAccept for LocalAccounts {
    fn on_accept(&self, _invite: &AcceptedInvite) -> AcceptDecision {
        AcceptDecision::LocalAccount
    }
}

/// An email ready to send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutgoingMail {
    pub company: String,
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
}

/// Sends email. The server's SMTP sender is one implementation.
pub trait MailSender: Send + Sync {
    fn send(&self, mail: &OutgoingMail) -> Result<(), String>;
}

/// Stores file contents by key (documents, drawings, photos). The data folder is one
/// implementation; object storage can be another.
pub trait BlobStore: Send + Sync {
    fn put(&self, key: &str, bytes: &[u8]) -> Result<(), String>;
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}

/// Work that runs on a timer (mail queue, reminders, recurring forms, scheduled backups).
pub trait BackgroundJob: Send + Sync {
    /// A short name for logs.
    fn name(&self) -> &'static str;
    /// Runs one pass for one company. An error is logged and the next company or job still runs.
    fn run(&self, company: &str, now: Timestamp) -> Result<(), String>;
}

/// On/off switches for features a build may add or hide.
pub trait FeatureFlags: Send + Sync {
    fn enabled(&self, company: &str, flag: &str) -> bool;
}

/// The default: no optional features are on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFlags;

impl FeatureFlags for NoFlags {
    fn enabled(&self, _company: &str, _flag: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_todays_behaviour() {
        let one = SingleCompany { slug: "summit-ridge".to_owned() };
        assert_eq!(one.company_for(None).as_deref(), Some("summit-ridge"));
        assert_eq!(one.company_for(Some("other.example")).as_deref(), Some("summit-ridge"));

        let attempt = SignIn { company: "summit-ridge", email: "ada@example.com", secret: "x" };
        assert_eq!(LocalOnly.authenticate(&attempt), AuthDecision::Local);

        let invite = AcceptedInvite { company: "summit-ridge".to_owned(), email: "ada@example.com".to_owned(), project: Some(3) };
        assert_eq!(LocalAccounts.on_accept(&invite), AcceptDecision::LocalAccount);

        assert!(!NoFlags.enabled("summit-ridge", "anything"));
    }

    #[test]
    fn traits_are_object_safe() {
        let _: Box<dyn TenantResolver> = Box::new(SingleCompany { slug: "a".to_owned() });
        let _: Box<dyn Authenticator> = Box::new(LocalOnly);
        let _: Box<dyn InviteAccept> = Box::new(LocalAccounts);
        let _: Box<dyn FeatureFlags> = Box::new(NoFlags);
        let _: Option<Box<dyn MailSender>> = None;
        let _: Option<Box<dyn BlobStore>> = None;
        let _: Option<Box<dyn BackgroundJob>> = None;
    }
}
