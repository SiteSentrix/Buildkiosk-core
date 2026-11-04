//! A small HTTP/1.1 client on `std::net`, so the suite needs no new dependencies: one request per connection, a
//! cookie jar, redirects never followed by itself. Every answer is also written to the shared request log, where the
//! safety checks (headers, error text, leaked secrets, slow requests) look at all of them at once.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Headers every answer must carry (the server's `security_headers` layer).
pub const SECURITY_HEADERS: [&str; 4] = ["content-security-policy", "x-content-type-options", "x-frame-options", "referrer-policy"];

pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub ms: u128,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }

    pub fn all(&self, name: &str) -> Vec<&str> {
        self.headers.iter().filter(|(k, _)| k == name).map(|(_, v)| v.as_str()).collect()
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    pub fn location(&self) -> &str {
        self.header("location").unwrap_or("")
    }

    pub fn is_html(&self) -> bool {
        self.header("content-type").is_some_and(|c| c.starts_with("text/html"))
    }

    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// The start of the body, on one line, for a report.
    pub fn snippet(&self) -> String {
        let text = self.text();
        let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
        one.chars().take(160).collect()
    }
}

/// One request, as the request log keeps it.
#[derive(Clone)]
pub struct Seen {
    pub who: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub ms: u128,
    /// Security headers the answer did not carry.
    pub missing: Vec<&'static str>,
    /// Error-page problems: panic text, a stack trace, a source location, a secret.
    pub leaks: Vec<String>,
}

pub type Log = Rc<RefCell<Vec<Seen>>>;

/// Words that must never reach a page: what a crash or a debug build would print.
const CRASH_TEXT: [&str; 4] = ["panicked at", "stack backtrace", "RUST_BACKTRACE", "called `Result::unwrap()`"];

pub struct Client {
    pub port: u16,
    pub who: String,
    pub cookies: BTreeMap<String, String>,
    pub bearer: Option<String>,
    pub log: Log,
    /// Values that must never appear in an answer (passwords, API tokens).
    pub secrets: Rc<RefCell<Vec<String>>>,
}

pub struct FilePart {
    pub field: String,
    pub name: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl FilePart {
    pub fn new(field: &str, name: &str, mime: &str, bytes: Vec<u8>) -> FilePart {
        FilePart { field: field.to_string(), name: name.to_string(), mime: mime.to_string(), bytes }
    }
}

impl Client {
    pub fn new(port: u16, who: &str, log: &Log, secrets: &Rc<RefCell<Vec<String>>>) -> Client {
        Client { port, who: who.to_string(), cookies: BTreeMap::new(), bearer: None, log: log.clone(), secrets: secrets.clone() }
    }

    pub fn get(&mut self, path: &str) -> Response {
        self.request("GET", path, None, &[])
    }

    pub fn post_form(&mut self, path: &str, fields: &[(String, String)]) -> Response {
        let body: Vec<String> = fields.iter().map(|(k, v)| format!("{}={}", enc(k), enc(v))).collect();
        self.request("POST", path, Some("application/x-www-form-urlencoded"), body.join("&").as_bytes())
    }

    pub fn post_multipart(&mut self, path: &str, fields: &[(String, String)], files: &[FilePart]) -> Response {
        let boundary = "----buildkiosk-regress-7d1f0a93";
        let mut body = Vec::new();
        for (k, v) in fields {
            body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes());
        }
        for f in files {
            body.extend_from_slice(
                format!(
                    "--{boundary}\r\nContent-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\nContent-Type: {}\r\n\r\n",
                    f.field, f.name, f.mime
                )
                .as_bytes(),
            );
            body.extend_from_slice(&f.bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        self.request("POST", path, Some(format!("multipart/form-data; boundary={boundary}").as_str()), &body)
    }

    pub fn post_json(&mut self, path: &str, json: &str) -> Response {
        self.request("POST", path, Some("application/json"), json.as_bytes())
    }

    pub fn request(&mut self, method: &str, path: &str, content_type: Option<&str>, body: &[u8]) -> Response {
        let start = Instant::now();
        let raw = self.exchange(method, path, content_type, body);
        let ms = start.elapsed().as_millis();
        // No answer at all (the server closed the connection) is status 0 in the report.
        let res = parse(&raw, ms).unwrap_or(Response { status: 0, headers: Vec::new(), body: Vec::new(), ms });
        for v in res.all("set-cookie") {
            let first = v.split(';').next().unwrap_or("");
            if let Some((k, val)) = first.split_once('=') {
                let gone = val.is_empty() || v.split(';').any(|a| a.trim().eq_ignore_ascii_case("max-age=0"));
                if gone {
                    self.cookies.remove(k.trim());
                } else {
                    self.cookies.insert(k.trim().to_string(), val.to_string());
                }
            }
        }
        self.record(method, path, &res);
        res
    }

    fn exchange(&self, method: &str, path: &str, content_type: Option<&str>, body: &[u8]) -> Vec<u8> {
        let Ok(mut stream) = TcpStream::connect(("127.0.0.1", self.port)) else { return Vec::new() };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(180)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(180)));
        let mut head =
            format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\nUser-Agent: buildkiosk-regress\r\n", self.port);
        if !self.cookies.is_empty() {
            let jar: Vec<String> = self.cookies.iter().map(|(k, v)| format!("{k}={v}")).collect();
            head.push_str(&format!("Cookie: {}\r\n", jar.join("; ")));
        }
        if let Some(t) = &self.bearer {
            head.push_str(&format!("Authorization: Bearer {t}\r\n"));
        }
        if let Some(ct) = content_type {
            head.push_str(&format!("Content-Type: {ct}\r\n"));
        }
        if method != "GET" {
            head.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        head.push_str("\r\n");
        // A server that refuses a large body may answer and close before reading it all: the write then fails, and
        // the answer (if any) is still read below.
        if stream.write_all(head.as_bytes()).is_ok() {
            let _ = stream.write_all(body);
        }
        let mut raw = Vec::new();
        let _ = stream.read_to_end(&mut raw);
        raw
    }

    fn record(&self, method: &str, path: &str, res: &Response) {
        let missing: Vec<&'static str> =
            if res.status == 0 { Vec::new() } else { SECURITY_HEADERS.into_iter().filter(|h| res.header(h).is_none()).collect() };
        let mut leaks = Vec::new();
        let textual = res.header("content-type").is_none_or(|c| c.starts_with("text/") || c.contains("json"));
        if textual {
            let text = res.text();
            for w in CRASH_TEXT {
                if text.contains(w) {
                    leaks.push(format!("shows `{w}`"));
                }
            }
            if has_source_location(&text) {
                leaks.push("shows a Rust source location".to_string());
            }
            for s in self.secrets.borrow().iter() {
                if !s.is_empty() && text.contains(s.as_str()) {
                    leaks.push("shows a password or an API token".to_string());
                }
            }
        }
        let seen = Seen { who: self.who.clone(), method: method.to_string(), path: path.to_string(), status: res.status, ms: res.ms, missing, leaks };
        self.log.borrow_mut().push(seen);
    }
}

/// `src/….rs:123` as a panic message or a backtrace prints it.
fn has_source_location(text: &str) -> bool {
    let mut rest = text;
    while let Some(i) = rest.find(".rs:") {
        let before = rest.get(..i).unwrap_or("");
        let after = rest.get(i + 4..).unwrap_or("");
        if after.starts_with(|c: char| c.is_ascii_digit()) && before.rsplit(|c: char| c.is_whitespace()).next().is_some_and(|w| w.contains("src/")) {
            return true;
        }
        rest = after;
    }
    false
}

fn parse(raw: &[u8], ms: u128) -> Option<Response> {
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = String::from_utf8_lossy(raw.get(..end)?).into_owned();
    let mut lines = head.split("\r\n");
    let status: u16 = lines.next()?.split(' ').nth(1)?.parse().ok()?;
    let headers: Vec<(String, String)> =
        lines.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string())).collect();
    let mut body = raw.get(end + 4..)?.to_vec();
    let chunked = headers.iter().any(|(k, v)| k == "transfer-encoding" && v.eq_ignore_ascii_case("chunked"));
    if chunked {
        body = dechunk(&body);
    } else if let Some(n) = headers.iter().find(|(k, _)| k == "content-length").and_then(|(_, v)| v.parse::<usize>().ok()) {
        body.truncate(n);
    }
    Some(Response { status, headers, body, ms })
}

fn dechunk(mut b: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    while let Some(nl) = b.windows(2).position(|w| w == b"\r\n") {
        let size_text = String::from_utf8_lossy(&b[..nl]).into_owned();
        let size = usize::from_str_radix(size_text.split(';').next().unwrap_or("").trim(), 16).unwrap_or(0);
        if size == 0 {
            break;
        }
        let start = nl + 2;
        let end = (start + size).min(b.len());
        out.extend_from_slice(&b[start..end]);
        b = b.get(end + 2..).unwrap_or(&[]);
    }
    out
}

/// Percent-encodes a form value (the same set the server uses).
pub fn enc(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Decodes a percent-encoded query value (`+` is a space).
pub fn dec(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => match u8::from_str_radix(std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("zz"), 16) {
                Ok(v) => {
                    out.push(v);
                    i += 2;
                }
                Err(_) => out.push(b'%'),
            },
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The value of query parameter `key` in a path or address.
pub fn query_value(url: &str, key: &str) -> Option<String> {
    let (_, query) = url.split_once('?')?;
    query.split('&').filter_map(|p| p.split_once('=')).find(|(k, _)| *k == key).map(|(_, v)| dec(v))
}
