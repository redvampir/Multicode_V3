use once_cell::sync::Lazy;
use regex::Regex;

static PYTHON_SINGLE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*#\s*@VISUAL_META\s*(?P<json>\{.*\})\s*$").unwrap()
});

static SLASH_SINGLE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*//\s*@VISUAL_META\s*(?P<json>\{.*\})\s*$").unwrap()
});

static C_STYLE_MULTI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)/\*\s*@VISUAL_META\s*(?P<json>\{.*?\})\s*\*/").unwrap()
});

static HTML_MULTI: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<!--\s*@VISUAL_META\s*(?P<json>\{.*?\})\s*-->").unwrap()
});

pub fn extract_json(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for caps in PYTHON_SINGLE.captures_iter(content) {
        if let Some(m) = caps.name("json") {
            out.push(m.as_str().to_string());
        }
    }
    for caps in SLASH_SINGLE.captures_iter(content) {
        if let Some(m) = caps.name("json") {
            out.push(m.as_str().to_string());
        }
    }
    for caps in C_STYLE_MULTI.captures_iter(content) {
        if let Some(m) = caps.name("json") {
            out.push(m.as_str().to_string());
        }
    }
    for caps in HTML_MULTI.captures_iter(content) {
        if let Some(m) = caps.name("json") {
            out.push(m.as_str().to_string());
        }
    }
    out
}

pub fn strip(content: &str) -> String {
    let mut out = content.to_string();
    for re in [&PYTHON_SINGLE, &SLASH_SINGLE, &C_STYLE_MULTI, &HTML_MULTI] {
        out = re.replace_all(&out, "").to_string();
    }
    out
}
