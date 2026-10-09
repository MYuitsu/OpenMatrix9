// SPDX-License-Identifier: LGPL-2.1-or-later
//! OM9-HISTORY-001 durable recipes. No native pointers, evaluation, or implicit solver.
use std::collections::BTreeMap;
use std::ffi::{CStr, c_char};
const LIMIT: usize = 1_048_576;
#[cfg(test)]
mod json_document_tests {
    use super::*;
    #[test]
    fn document_parser_bounds_bytes_nodes_depth_and_trailing_content() {
        assert!(matches!(parse_document("[1,2]", 5, 3), Ok(Json::Array(_))));
        assert!(parse_document("[1,2]", 4, 3).is_err());
        assert!(parse_document("[1,2]", 5, 2).is_err());
        assert!(parse_document("null", 4, 0).is_err());
        assert!(parse_document("{} null", 7, 2).is_err());
        let nested = |depth: usize| format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        assert!(parse_document(&nested(64), 1024, 100).is_ok());
        assert!(parse_document(&nested(65), 1024, 100).is_err());
        let recipe = |value: &str| {
            format!(
                r#"{{"schema":1,"evaluator":"om9.affine-template","version":1,"settings":{{"nested":{value}}}}}"#,
            )
        };
        assert!(validate_recipe(&recipe(&nested(30))).is_ok());
        assert!(validate_recipe(&recipe(&nested(31))).is_err());
        let nodes = |count: usize| format!("[{}]", vec!["0"; count].join(","));
        assert!(validate_recipe(&recipe(&nodes(16378))).is_ok());
        assert!(validate_recipe(&recipe(&nodes(16379))).is_err());
        let overhead = recipe(r#""""#).len();
        let exact_bytes = recipe(&format!("\"{}\"", "x".repeat(LIMIT - overhead)));
        assert_eq!(exact_bytes.len(), LIMIT);
        assert!(validate_recipe(&exact_bytes).is_ok());
        assert!(validate_recipe(&(exact_bytes + " ")).is_err());
    }
}
pub fn valid_replay_budget(records: usize, templates: usize, targets: usize) -> bool {
    records > 0
        && templates >= records
        && targets > 0
        && records.checked_mul(targets).is_some_and(|n| n <= 4096)
        && templates.checked_mul(targets).is_some_and(|n| n <= 16384)
}
#[unsafe(no_mangle)]
pub extern "C" fn om9_builder_replay_budget(
    records: usize,
    templates: usize,
    targets: usize,
) -> bool {
    valid_replay_budget(records, templates, targets)
}
/// Exact syntactic feature identity, not a promise of registered builder capability.
pub fn valid_feature_id(id: &str) -> bool {
    let Some(rest) = id.strip_prefix("OM9-") else {
        return false;
    };
    let Some((domain, number)) = rest.split_once('-') else {
        return false;
    };
    !domain.is_empty()
        && domain.len() <= 16
        && domain.bytes().all(|b| b.is_ascii_uppercase())
        && number.len() == 3
        && number.bytes().all(|b| b.is_ascii_digit())
        && number != "000"
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_builder_feature_id_valid(id: *const c_char) -> bool {
    if id.is_null() {
        return false;
    }
    let Ok(s) = unsafe { CStr::from_ptr(id) }.to_str() else {
        return false;
    };
    valid_feature_id(s)
}
#[derive(Debug, Clone)]
pub(crate) enum Json {
    Object(BTreeMap<String, Json>),
    Array(Vec<Json>),
    String(String),
    Number(f64),
    Boolean,
    Other,
}
struct Parser<'a> {
    s: &'a str,
    i: usize,
    nodes: usize,
    max_nodes: usize,
    max_depth: usize,
}
impl Parser<'_> {
    fn ws(&mut self) {
        while self
            .s
            .as_bytes()
            .get(self.i)
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.i += 1;
        }
    }
    fn take(&mut self, b: u8) -> bool {
        self.ws();
        if self.s.as_bytes().get(self.i) == Some(&b) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn hex(&mut self) -> Result<u32, &'static str> {
        let end = self.i.checked_add(4).ok_or("escape")?;
        let digits = self.s.get(self.i..end).ok_or("escape")?;
        if !digits.as_bytes().iter().all(u8::is_ascii_hexdigit) {
            return Err("escape");
        }
        let v = u32::from_str_radix(digits, 16).map_err(|_| "escape")?;
        self.i = end;
        Ok(v)
    }
    fn string(&mut self) -> Result<String, &'static str> {
        if !self.take(b'"') {
            return Err("string");
        }
        let mut out = String::new();
        loop {
            let c = self
                .s
                .get(self.i..)
                .and_then(|s| s.chars().next())
                .ok_or("unterminated string")?;
            self.i += c.len_utf8();
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let e = self.s.as_bytes().get(self.i).copied().ok_or("escape")?;
                    self.i += 1;
                    out.push(match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut u = self.hex()?;
                            if (0xd800..=0xdbff).contains(&u) {
                                if !self.s[self.i..].starts_with("\\u") {
                                    return Err("surrogate");
                                }
                                self.i += 2;
                                let low = self.hex()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("surrogate");
                                }
                                u = 0x10000 + ((u - 0xd800) << 10) + (low - 0xdc00);
                            }
                            char::from_u32(u).ok_or("unicode")?
                        }
                        _ => return Err("escape"),
                    });
                }
                c if c < ' ' => return Err("control character"),
                _ => out.push(c),
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Json, &'static str> {
        self.nodes += 1;
        if depth > self.max_depth || self.nodes > self.max_nodes {
            return Err("JSON budget");
        }
        self.ws();
        match self.s.as_bytes().get(self.i).copied().ok_or("value")? {
            b'{' => {
                self.i += 1;
                let mut m = BTreeMap::new();
                if self.take(b'}') {
                    return Ok(Json::Object(m));
                }
                loop {
                    let k = self.string()?;
                    if !self.take(b':') {
                        return Err("colon");
                    }
                    let v = self.value(depth + 1)?;
                    if m.insert(k, v).is_some() {
                        return Err("duplicate field");
                    }
                    if self.take(b'}') {
                        break;
                    }
                    if !self.take(b',') {
                        return Err("comma");
                    }
                }
                Ok(Json::Object(m))
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                if self.take(b']') {
                    return Ok(Json::Array(a));
                }
                loop {
                    a.push(self.value(depth + 1)?);
                    if self.take(b']') {
                        break;
                    }
                    if !self.take(b',') {
                        return Err("comma");
                    }
                }
                Ok(Json::Array(a))
            }
            b'"' => Ok(Json::String(self.string()?)),
            b't' | b'f' | b'n' => {
                let word = match self.s.as_bytes()[self.i] {
                    b't' => "true",
                    b'f' => "false",
                    _ => "null",
                };
                if !self.s[self.i..].starts_with(word) {
                    return Err("literal");
                }
                self.i += word.len();
                Ok(if word == "null" { Json::Other } else { Json::Boolean })
            }
            _ => {
                let start = self.i;
                if self.s.as_bytes().get(self.i) == Some(&b'-') {
                    self.i += 1;
                }
                let digit = self.s.as_bytes().get(self.i).ok_or("number")?;
                if *digit == b'0' {
                    self.i += 1;
                } else if (b'1'..=b'9').contains(digit) {
                    while self
                        .s
                        .as_bytes()
                        .get(self.i)
                        .is_some_and(u8::is_ascii_digit)
                    {
                        self.i += 1;
                    }
                } else {
                    return Err("number");
                }
                if self.s.as_bytes().get(self.i) == Some(&b'.') {
                    self.i += 1;
                    let d = self.i;
                    while self
                        .s
                        .as_bytes()
                        .get(self.i)
                        .is_some_and(u8::is_ascii_digit)
                    {
                        self.i += 1;
                    }
                    if d == self.i {
                        return Err("fraction");
                    }
                }
                if self
                    .s
                    .as_bytes()
                    .get(self.i)
                    .is_some_and(|c| *c == b'e' || *c == b'E')
                {
                    self.i += 1;
                    if self
                        .s
                        .as_bytes()
                        .get(self.i)
                        .is_some_and(|c| *c == b'+' || *c == b'-')
                    {
                        self.i += 1;
                    }
                    let d = self.i;
                    while self
                        .s
                        .as_bytes()
                        .get(self.i)
                        .is_some_and(u8::is_ascii_digit)
                    {
                        self.i += 1;
                    }
                    if d == self.i {
                        return Err("exponent");
                    }
                }
                let n = self.s[start..self.i].parse::<f64>().map_err(|_| "number")?;
                if !n.is_finite() {
                    return Err("nonfinite number");
                }
                Ok(Json::Number(n))
            }
        }
    }
}
/// Bounded owned JSON for portable metadata, with no retained native pointers.
pub(crate) fn parse_document(
    raw: &str,
    max_bytes: usize,
    max_nodes: usize,
) -> Result<Json, &'static str> {
    parse_document_at_depth(raw, max_bytes, max_nodes, 64)
}
fn parse_document_at_depth(
    raw: &str,
    max_bytes: usize,
    max_nodes: usize,
    max_depth: usize,
) -> Result<Json, &'static str> {
    if raw.len() > max_bytes {
        return Err("JSON byte budget");
    }
    let mut parser = Parser {
        s: raw,
        i: 0,
        nodes: 0,
        max_nodes,
        max_depth,
    };
    let value = parser.value(0)?;
    parser.ws();
    if parser.i != raw.len() {
        return Err("trailing JSON");
    }
    Ok(value)
}
#[derive(Debug, Clone)]
pub struct Recipe {
    pub raw: String,
    pub scale: [f64; 3],
    pub offset: [f64; 3],
}
fn triple(value: Option<&Json>, default: [f64; 3]) -> Result<[f64; 3], &'static str> {
    match value {
        None => Ok(default),
        Some(Json::Array(a)) if a.len() == 3 => {
            let mut out = [0.; 3];
            for (i, v) in a.iter().enumerate() {
                if let Json::Number(n) = v {
                    out[i] = *n;
                } else {
                    return Err("vector number");
                }
            }
            Ok(out)
        }
        _ => Err("vector length"),
    }
}
pub fn validate_recipe(raw: &str) -> Result<Recipe, &'static str> {
    if raw.len() > LIMIT {
        return Err("recipe byte budget");
    }
    let Json::Object(m) = parse_document_at_depth(raw, LIMIT, 16384, 32)? else {
        return Err("recipe object");
    };
    if !matches!(m.get("schema"), Some(Json::Number(1.)))
        || !matches!(m.get("version"), Some(Json::Number(1.)))
    {
        return Err("unsupported schema or evaluator version");
    }
    if !matches!(m.get("evaluator"),Some(Json::String(s)) if s=="om9.affine-template") {
        return Err("unregistered evaluator");
    }
    if !matches!(m.get("settings"), Some(Json::Object(_))) {
        return Err("settings object required");
    }
    if m.keys().any(|k| {
        !matches!(
            k.as_str(),
            "schema" | "version" | "evaluator" | "settings" | "scale" | "offset"
        )
    }) {
        return Err("unknown evaluator field");
    }
    let scale = triple(m.get("scale"), [1.; 3])?;
    let offset = triple(m.get("offset"), [0.; 3])?;
    if scale.iter().any(|n| *n < 1e-9 || *n > 1e9) || offset.iter().any(|n| n.abs() > 1e9) {
        return Err("affine bounds");
    }
    Ok(Recipe {
        raw: raw.into(),
        scale,
        offset,
    })
}
pub fn affine_plan(
    recipe: &Recipe,
    initial: [f64; 3],
    current: [f64; 3],
    scale_to_gem: bool,
) -> Result<[f64; 6], &'static str> {
    if initial
        .iter()
        .chain(current.iter())
        .any(|n| !n.is_finite() || *n < 1e-9 || *n > 1e9)
    {
        return Err("gem dimensions");
    }
    let mut out = [0.; 6];
    for i in 0..3 {
        out[i] = recipe.scale[i]
            * if scale_to_gem {
                current[i] / initial[i]
            } else {
                1.
            };
        out[i + 3] = recipe.offset[i];
        if !out[i].is_finite() || out[i] < 1e-9 || out[i] > 1e9 {
            return Err("mapping bounds");
        }
    }
    Ok(out)
}
pub fn compatible_shapes(source: &str, targets: &[&str]) -> Result<(), &'static str> {
    if source.trim().is_empty()
        || source.chars().any(char::is_control)
        || source.len() > 128
        || targets.is_empty()
        || targets.len() > 1024
        || targets.iter().any(|s| *s != source)
    {
        Err("explicit same gem shape required")
    } else {
        Ok(())
    }
}
/// Native supplies readable NUL-terminated UTF-8 and writable 6-double buffer.
/// No allocations cross this ABI; no native objects or pointers are retained.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_builder_recipe_plan(
    raw: *const c_char,
    initial: *const f64,
    current: *const f64,
    scale: bool,
    out: *mut f64,
) -> bool {
    if raw.is_null() || initial.is_null() || current.is_null() || out.is_null() {
        return false;
    }
    let Ok(s) = unsafe { CStr::from_ptr(raw) }.to_str() else {
        return false;
    };
    let Ok(r) = validate_recipe(s) else {
        return false;
    };
    let mut a = [0.; 3];
    let mut b = [0.; 3];
    a.copy_from_slice(unsafe { std::slice::from_raw_parts(initial, 3) });
    b.copy_from_slice(unsafe { std::slice::from_raw_parts(current, 3) });
    let Ok(plan) = affine_plan(&r, a, b, scale) else {
        return false;
    };
    unsafe { std::ptr::copy_nonoverlapping(plan.as_ptr(), out, 6) };
    true
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn om9_builder_shapes_compatible(
    source: *const c_char,
    target: *const c_char,
) -> bool {
    if source.is_null() || target.is_null() {
        return false;
    }
    let (Ok(s), Ok(t)) = (
        unsafe { CStr::from_ptr(source) }.to_str(),
        unsafe { CStr::from_ptr(target) }.to_str(),
    ) else {
        return false;
    };
    compatible_shapes(s, &[t]).is_ok()
}
