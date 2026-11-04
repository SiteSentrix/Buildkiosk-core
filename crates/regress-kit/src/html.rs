//! Reads the server-rendered pages the way a browser would for this suite: the links on a page, and its forms with
//! the values a browser would send (filling in required fields a person would have to type).

/// In [`Form::fill`]: choose the first choice the form offers for that field (a select's first option with a value, or
/// the first checkbox or radio button of that name), as a person would tick the first box.
pub const FIRST: &str = "\u{1}first";

/// In [`Form::fill`]: `LABEL` followed by some text chooses the select option whose text contains it (a person picks
/// a name from a list).
pub const LABEL: &str = "\u{1}label:";

pub fn unescape(s: &str) -> String {
    s.replace("&quot;", "\"").replace("&#39;", "'").replace("&#x27;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

/// Every `href` on the page.
pub fn links(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(i) = rest.find("href=\"") {
        rest = &rest[i + 6..];
        let Some(j) = rest.find('"') else { break };
        out.push(unescape(&rest[..j]));
        rest = &rest[j..];
    }
    out
}

/// A tag's name (lowercase) and attributes, from the text between `<` and `>`.
fn tag(text: &str) -> (String, Vec<(String, String)>) {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut name = String::new();
    while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '/' {
        name.push(chars[i].to_ascii_lowercase());
        i += 1;
    }
    let mut attrs = Vec::new();
    loop {
        while i < chars.len() && (chars[i].is_whitespace() || chars[i] == '/') {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let mut key = String::new();
        while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '=' && chars[i] != '/' {
            key.push(chars[i].to_ascii_lowercase());
            i += 1;
        }
        let mut value = String::new();
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != quote {
                    value.push(chars[i]);
                    i += 1;
                }
                i += 1;
            } else {
                while i < chars.len() && !chars[i].is_whitespace() {
                    value.push(chars[i]);
                    i += 1;
                }
            }
        }
        if key.is_empty() {
            i += 1;
        } else {
            attrs.push((key, unescape(&value)));
        }
    }
    (name, attrs)
}

fn attr<'a>(attrs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attrs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

#[derive(Debug, Clone)]
pub struct Field {
    /// `text`, `hidden`, `checkbox`, … for an input; `select`, `textarea` or `button` otherwise.
    pub kind: String,
    pub name: String,
    pub value: String,
    pub required: bool,
    pub checked: bool,
    pub multiple: bool,
    /// For a select: (value, selected).
    pub options: Vec<(String, bool)>,
    /// For a select: each option's text, in the same order as `options`.
    pub labels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Form {
    pub action: String,
    pub method: String,
    pub multipart: bool,
    pub fields: Vec<Field>,
}

/// Every form on the page.
pub fn forms(page: &str) -> Vec<Form> {
    let mut out = Vec::new();
    let mut rest = page;
    while let Some(i) = rest.find("<form") {
        rest = &rest[i..];
        let Some(open_end) = rest.find('>') else { break };
        let (_, attrs) = tag(&rest[1..open_end]);
        let close = rest.find("</form>").unwrap_or(rest.len());
        let inner = rest.get(open_end + 1..close).unwrap_or("");
        out.push(Form {
            action: attr(&attrs, "action").unwrap_or("").to_string(),
            method: attr(&attrs, "method").unwrap_or("get").to_ascii_lowercase(),
            multipart: attr(&attrs, "enctype").is_some_and(|e| e.contains("multipart")),
            fields: fields(inner),
        });
        rest = rest.get(close..).unwrap_or("");
        rest = rest.strip_prefix("</form>").unwrap_or(rest);
    }
    out
}

fn fields(inner: &str) -> Vec<Field> {
    let mut out = Vec::new();
    let mut pos = 0;
    while let Some(i) = inner[pos..].find('<') {
        let start = pos + i;
        let Some(j) = inner[start..].find('>') else { break };
        let end = start + j;
        let (name, attrs) = tag(&inner[start + 1..end]);
        pos = end + 1;
        let field = |kind: &str, value: String, options: Vec<(String, bool)>| Field {
            kind: kind.to_string(),
            name: attr(&attrs, "name").unwrap_or("").to_string(),
            value,
            required: attr(&attrs, "required").is_some(),
            checked: attr(&attrs, "checked").is_some(),
            multiple: attr(&attrs, "multiple").is_some(),
            options,
            labels: Vec::new(),
        };
        match name.as_str() {
            "input" => {
                let kind = attr(&attrs, "type").unwrap_or("text").to_ascii_lowercase();
                out.push(field(&kind, attr(&attrs, "value").unwrap_or("").to_string(), Vec::new()));
            }
            "button" => {
                if attr(&attrs, "name").is_some() {
                    out.push(field("button", attr(&attrs, "value").unwrap_or("").to_string(), Vec::new()));
                }
            }
            "textarea" => {
                let close = inner[pos..].find("</textarea>").map_or(inner.len(), |c| pos + c);
                out.push(field("textarea", unescape(&inner[pos..close]), Vec::new()));
                pos = close;
            }
            "select" => {
                let close = inner[pos..].find("</select>").map_or(inner.len(), |c| pos + c);
                let mut options = Vec::new();
                let mut labels = Vec::new();
                let body = &inner[pos..close];
                let mut at = 0;
                while let Some(k) = body[at..].find("<option") {
                    let s = at + k;
                    let Some(e) = body[s..].find('>') else { break };
                    let (_, oattrs) = tag(&body[s + 1..s + e]);
                    let text_end = body[s + e + 1..].find('<').map_or(body.len(), |t| s + e + 1 + t);
                    let value = match attr(&oattrs, "value") {
                        Some(v) => v.to_string(),
                        None => unescape(body[s + e + 1..text_end].trim()),
                    };
                    options.push((value, attr(&oattrs, "selected").is_some()));
                    labels.push(unescape(body[s + e + 1..text_end].trim()));
                    at = s + e + 1;
                }
                let mut select = field("select", String::new(), options);
                select.labels = labels;
                out.push(select);
                pos = close;
            }
            _ => {}
        }
    }
    out
}

/// What a person would type into a required field that is empty.
fn typed(kind: &str, name: &str) -> String {
    match kind {
        "date" => "2026-11-20".to_string(),
        "datetime-local" => "2026-11-20T09:00".to_string(),
        "time" => "09:00".to_string(),
        "month" => "2026-11".to_string(),
        "number" | "range" => "1".to_string(),
        "email" => "qa-regress@example.com".to_string(),
        "url" => "https://example.com/qa".to_string(),
        "tel" => "555-0100".to_string(),
        "password" => "qa-regress-password-1".to_string(),
        _ => format!("QA regression {name}"),
    }
}

impl Form {
    /// The fields a browser would send, with `set` replacing values by name (only fields the form has). A field set
    /// to `""` in `set` is left out. Buttons are sent only when named in `set`.
    pub fn fill(&self, set: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        let mut radios: Vec<String> = Vec::new();
        for f in &self.fields {
            if f.name.is_empty() || set.iter().any(|(k, _)| *k == f.name) {
                continue;
            }
            match f.kind.as_str() {
                "button" | "submit" | "reset" | "image" | "file" => {}
                "checkbox" => {
                    if f.checked {
                        out.push((f.name.clone(), if f.value.is_empty() { "on".to_string() } else { f.value.clone() }));
                    }
                }
                "radio" => {
                    if f.checked && !radios.contains(&f.name) {
                        radios.push(f.name.clone());
                        out.push((f.name.clone(), f.value.clone()));
                    }
                }
                "select" => {
                    let selected: Vec<&String> = f.options.iter().filter(|(_, s)| *s).map(|(v, _)| v).collect();
                    if !selected.is_empty() {
                        for v in selected {
                            out.push((f.name.clone(), v.clone()));
                        }
                    } else if f.required || !f.multiple {
                        let pick = if f.required { f.options.iter().find(|(v, _)| !v.is_empty()) } else { f.options.first() };
                        if let Some((v, _)) = pick {
                            out.push((f.name.clone(), v.clone()));
                        }
                    }
                }
                kind => {
                    let value = if f.value.is_empty() && f.required { typed(kind, &f.name) } else { f.value.clone() };
                    out.push((f.name.clone(), value));
                }
            }
        }
        // A required radio group with nothing chosen: the first choice.
        for f in &self.fields {
            if f.kind == "radio" && f.required && !radios.contains(&f.name) && !set.iter().any(|(k, _)| *k == f.name) {
                radios.push(f.name.clone());
                out.push((f.name.clone(), f.value.clone()));
            }
        }
        // Only what the form has: a person can't send a field the page doesn't offer.
        for (k, v) in set {
            if v.is_empty() || !self.has(k) {
                continue;
            }
            if *v == FIRST {
                if let Some(first) = self.first_choice(k) {
                    out.push(((*k).to_string(), first));
                }
            } else if let Some(text) = v.strip_prefix(LABEL) {
                if let Some(value) = self.labelled(k, text) {
                    out.push(((*k).to_string(), value));
                }
            } else {
                out.push(((*k).to_string(), (*v).to_string()));
            }
        }
        out
    }

    /// The value of the first option of select `name` whose text contains `text`.
    fn labelled(&self, name: &str, text: &str) -> Option<String> {
        self.fields
            .iter()
            .filter(|f| f.name == name)
            .find_map(|f| f.labels.iter().position(|l| l.contains(text)).and_then(|i| f.options.get(i)).map(|(v, _)| v.clone()))
    }

    /// The first choice offered for field `name`: a select's first option with a value, or the first checkbox or
    /// radio button of that name.
    fn first_choice(&self, name: &str) -> Option<String> {
        self.fields.iter().filter(|f| f.name == name).find_map(|f| match f.kind.as_str() {
            "select" => f.options.iter().find(|(v, _)| !v.is_empty()).map(|(v, _)| v.clone()),
            "checkbox" | "radio" if !f.value.is_empty() => Some(f.value.clone()),
            _ => None,
        })
    }

    pub fn has(&self, name: &str) -> bool {
        self.fields.iter().any(|f| f.name == name)
    }
}

/// The post form whose action is `part`, else the first whose action contains it (any post form but sign-out for
/// `""`).
pub fn post_form<'a>(forms: &'a [Form], part: &str) -> Option<&'a Form> {
    let posts = || forms.iter().filter(|f| f.method == "post" && f.action != "/logout" && !f.action.starts_with("/language"));
    posts().find(|f| f.action == part).or_else(|| posts().find(|f| f.action.contains(part)))
}
