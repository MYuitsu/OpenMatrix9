//! TOML tối giản đủ cho manifest và preferences.
//!
//! Hỗ trợ: bảng `[a.b]`, mảng bảng `[[a]]`, khoá chấm `a.b = 1`, khoá trích dẫn,
//! chuỗi `"…"` (escape \" \\ \n \t \r \uXXXX \UXXXXXXXX), chuỗi literal `'…'`,
//! chuỗi nhiều dòng `"""…"""`, số nguyên, số thực, bool, mảng (nhiều dòng, dấu phẩy cuối),
//! bảng inline `{ k = v }`, chú thích `#`.
//! Không hỗ trợ: ngày giờ, số hex/oct/bin, inf/nan — gặp sẽ báo lỗi rõ ràng.

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Table(Table),
}

pub type Table = BTreeMap<String, Value>;

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        if let Value::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
    pub fn as_table(&self) -> Option<&Table> {
        if let Value::Table(t) = self {
            Some(t)
        } else {
            None
        }
    }
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Str(_) => "chuỗi",
            Value::Int(_) => "số nguyên",
            Value::Float(_) => "số thực",
            Value::Bool(_) => "bool",
            Value::Array(_) => "mảng",
            Value::Table(_) => "bảng",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TomlError {
    pub line: usize,
    pub msg: String,
}

impl fmt::Display for TomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dòng {}: {}", self.line, self.msg)
    }
}

pub fn parse(src: &str) -> Result<Table, TomlError> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
    Parser {
        s: src.as_bytes(),
        i: 0,
        line: 1,
    }
    .document()
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    line: usize,
}

fn descend<'t>(
    mut t: &'t mut Table,
    path: &[String],
    line: usize,
) -> Result<&'t mut Table, TomlError> {
    for k in path {
        let v = t
            .entry(k.clone())
            .or_insert_with(|| Value::Table(Table::new()));
        t = match v {
            Value::Table(x) => x,
            Value::Array(a) => match a.last_mut() {
                Some(Value::Table(x)) => x,
                _ => {
                    return Err(TomlError {
                        line,
                        msg: format!("khoá {k:?} là mảng giá trị, không phải bảng"),
                    });
                }
            },
            other => {
                return Err(TomlError {
                    line,
                    msg: format!("khoá {k:?} đã là {} , không phải bảng", other.type_name()),
                });
            }
        };
    }
    Ok(t)
}

impl<'a> Parser<'a> {
    fn err<T>(&self, msg: impl Into<String>) -> Result<T, TomlError> {
        Err(TomlError {
            line: self.line,
            msg: msg.into(),
        })
    }
    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }
    fn peek_at(&self, k: usize) -> Option<u8> {
        self.s.get(self.i + k).copied()
    }
    fn bump(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.i += 1;
        if c == b'\n' {
            self.line += 1;
        }
        Some(c)
    }
    fn spaces(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.i += 1;
        }
    }
    fn comment(&mut self) {
        if self.peek() == Some(b'#') {
            while !matches!(self.peek(), None | Some(b'\n')) {
                self.i += 1;
            }
        }
    }
    /// khoảng trắng, chú thích và xuống dòng (dùng giữa các phần tử mảng / giữa dòng)
    fn blank(&mut self) {
        loop {
            self.spaces();
            self.comment();
            match self.peek() {
                Some(b'\n') => {
                    self.bump();
                }
                Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                    self.i += 1;
                    self.bump();
                }
                _ => break,
            }
        }
    }
    fn eol(&mut self) -> Result<(), TomlError> {
        self.spaces();
        self.comment();
        match self.peek() {
            None => Ok(()),
            Some(b'\n') => {
                self.bump();
                Ok(())
            }
            Some(b'\r') if self.peek_at(1) == Some(b'\n') => {
                self.i += 1;
                self.bump();
                Ok(())
            }
            Some(c) => self.err(format!("ký tự thừa sau giá trị: {:?}", c as char)),
        }
    }

    fn document(mut self) -> Result<Table, TomlError> {
        let mut root = Table::new();
        let mut cur: Vec<String> = Vec::new();
        let mut defined: Vec<Vec<String>> = Vec::new();
        loop {
            self.blank();
            let Some(c) = self.peek() else { break };
            let line = self.line;
            if c == b'[' {
                if self.peek_at(1) == Some(b'[') {
                    self.i += 2;
                    self.spaces();
                    let path = self.key_path()?;
                    self.spaces();
                    if !(self.peek() == Some(b']') && self.peek_at(1) == Some(b']')) {
                        return self.err("thiếu ']]'");
                    }
                    self.i += 2;
                    self.eol()?;
                    let (last, parent) = path.split_last().unwrap();
                    let pt = descend(&mut root, parent, line)?;
                    let v = pt
                        .entry(last.clone())
                        .or_insert_with(|| Value::Array(Vec::new()));
                    match v {
                        Value::Array(a) if a.iter().all(|x| matches!(x, Value::Table(_))) => {
                            a.push(Value::Table(Table::new()))
                        }
                        _ => {
                            return Err(TomlError {
                                line,
                                msg: format!("{last:?} không phải mảng bảng"),
                            });
                        }
                    }
                    // mảng bảng mới: các bảng con của phần tử trước không còn tính là đã định nghĩa
                    defined.retain(|d| !d.starts_with(&path));
                    cur = path;
                } else {
                    self.i += 1;
                    self.spaces();
                    let path = self.key_path()?;
                    self.spaces();
                    if self.peek() != Some(b']') {
                        return self.err("thiếu ']'");
                    }
                    self.i += 1;
                    self.eol()?;
                    if defined.contains(&path) {
                        return Err(TomlError {
                            line,
                            msg: format!("bảng [{}] khai báo hai lần", path.join(".")),
                        });
                    }
                    descend(&mut root, &path, line)?;
                    defined.push(path.clone());
                    cur = path;
                }
            } else {
                let key = self.key_path()?;
                self.spaces();
                if self.peek() != Some(b'=') {
                    return self.err("thiếu '=' sau khoá");
                }
                self.i += 1;
                self.spaces();
                let v = self.value()?;
                self.eol()?;
                let t = descend(&mut root, &cur, line)?;
                insert(t, &key, v, line)?;
            }
        }
        Ok(root)
    }

    fn key_path(&mut self) -> Result<Vec<String>, TomlError> {
        let mut out = Vec::new();
        loop {
            self.spaces();
            let k = match self.peek() {
                Some(b'"') => {
                    self.i += 1;
                    self.basic_string()?
                }
                Some(b'\'') => {
                    self.i += 1;
                    self.literal_string()?
                }
                _ => {
                    let st = self.i;
                    while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
                    {
                        self.i += 1;
                    }
                    if st == self.i {
                        return self.err("thiếu khoá");
                    }
                    String::from_utf8(self.s[st..self.i].to_vec()).unwrap()
                }
            };
            out.push(k);
            self.spaces();
            if self.peek() == Some(b'.') {
                self.i += 1;
            } else {
                return Ok(out);
            }
        }
    }

    fn value(&mut self) -> Result<Value, TomlError> {
        match self.peek() {
            Some(b'"') => {
                if self.peek_at(1) == Some(b'"') && self.peek_at(2) == Some(b'"') {
                    self.i += 3;
                    if self.peek() == Some(b'\n') {
                        self.bump();
                    } else if self.peek() == Some(b'\r') && self.peek_at(1) == Some(b'\n') {
                        self.i += 1;
                        self.bump();
                    }
                    return self.multiline_string().map(Value::Str);
                }
                self.i += 1;
                self.basic_string().map(Value::Str)
            }
            Some(b'\'') => {
                self.i += 1;
                self.literal_string().map(Value::Str)
            }
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                loop {
                    self.blank();
                    if self.peek() == Some(b']') {
                        self.i += 1;
                        return Ok(Value::Array(items));
                    }
                    items.push(self.value()?);
                    self.blank();
                    match self.peek() {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Value::Array(items));
                        }
                        _ => return self.err("mảng thiếu ',' hoặc ']'"),
                    }
                }
            }
            Some(b'{') => {
                self.i += 1;
                let mut t = Table::new();
                self.spaces();
                if self.peek() == Some(b'}') {
                    self.i += 1;
                    return Ok(Value::Table(t));
                }
                loop {
                    let line = self.line;
                    let k = self.key_path()?;
                    self.spaces();
                    if self.peek() != Some(b'=') {
                        return self.err("bảng inline thiếu '='");
                    }
                    self.i += 1;
                    self.spaces();
                    let v = self.value()?;
                    insert(&mut t, &k, v, line)?;
                    self.spaces();
                    match self.peek() {
                        Some(b',') => {
                            self.i += 1;
                            self.spaces();
                        }
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Value::Table(t));
                        }
                        _ => return self.err("bảng inline thiếu ',' hoặc '}'"),
                    }
                }
            }
            Some(b't') if self.s[self.i..].starts_with(b"true") => {
                self.i += 4;
                Ok(Value::Bool(true))
            }
            Some(b'f') if self.s[self.i..].starts_with(b"false") => {
                self.i += 5;
                Ok(Value::Bool(false))
            }
            Some(c) if c.is_ascii_digit() || c == b'-' || c == b'+' => self.number(),
            Some(c) => self.err(format!("giá trị không hỗ trợ bắt đầu bằng {:?}", c as char)),
            None => self.err("thiếu giá trị"),
        }
    }

    fn number(&mut self) -> Result<Value, TomlError> {
        let st = self.i;
        while matches!(self.peek(), Some(c) if c.is_ascii_alphanumeric() || matches!(c, b'+' | b'-' | b'.' | b'_'))
        {
            self.i += 1;
        }
        let raw = std::str::from_utf8(&self.s[st..self.i]).unwrap();
        if raw.contains("__") || raw.starts_with('_') || raw.ends_with('_') {
            return self.err(format!("số không hợp lệ: {raw}"));
        }
        let txt: String = raw.chars().filter(|&c| c != '_').collect();
        let body = txt.trim_start_matches(['+', '-']);
        if body.starts_with("0x")
            || body.starts_with("0o")
            || body.starts_with("0b")
            || body == "inf"
            || body == "nan"
        {
            return self.err(format!("dạng số chưa hỗ trợ: {raw}"));
        }
        let int_part = body.split(['.', 'e', 'E']).next().unwrap_or("");
        if int_part.len() > 1 && int_part.starts_with('0') {
            return self.err(format!("số không được bắt đầu bằng 0: {raw}"));
        }
        if txt.contains(['.', 'e', 'E']) {
            if txt.contains('.') {
                let mut sp = body.splitn(2, '.');
                let (a, b) = (sp.next().unwrap(), sp.next().unwrap_or(""));
                if a.is_empty() || b.is_empty() || !b.as_bytes()[0].is_ascii_digit() {
                    return self.err(format!("số thực không hợp lệ: {raw}"));
                }
            }
            match txt.parse::<f64>() {
                Ok(f) if f.is_finite() => Ok(Value::Float(f)),
                _ => self.err(format!("số thực không hợp lệ: {raw}")),
            }
        } else {
            txt.parse::<i64>()
                .map(Value::Int)
                .or_else(|_| self.err(format!("số nguyên không hợp lệ: {raw}")))
        }
    }

    fn escape(&mut self, out: &mut Vec<u8>) -> Result<(), TomlError> {
        let Some(e) = self.bump() else {
            return self.err("escape dở dang");
        };
        let ch = match e {
            b'"' => '"',
            b'\\' => '\\',
            b'n' => '\n',
            b't' => '\t',
            b'r' => '\r',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'u' | b'U' => {
                let n = if e == b'u' { 4 } else { 8 };
                let hex = self.s.get(self.i..self.i + n).ok_or(TomlError {
                    line: self.line,
                    msg: "\\u thiếu chữ số".into(),
                })?;
                let hex = std::str::from_utf8(hex).map_err(|_| TomlError {
                    line: self.line,
                    msg: "\\u sai".into(),
                })?;
                let cp = u32::from_str_radix(hex, 16).map_err(|_| TomlError {
                    line: self.line,
                    msg: format!("\\u sai: {hex}"),
                })?;
                self.i += n;
                char::from_u32(cp).ok_or(TomlError {
                    line: self.line,
                    msg: format!("mã Unicode không hợp lệ: {hex}"),
                })?
            }
            c => return self.err(format!("escape không hỗ trợ: \\{}", c as char)),
        };
        let mut buf = [0u8; 4];
        out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
        Ok(())
    }

    fn utf8(&self, v: Vec<u8>) -> Result<String, TomlError> {
        String::from_utf8(v).map_err(|_| TomlError {
            line: self.line,
            msg: "chuỗi không phải UTF-8".into(),
        })
    }

    fn basic_string(&mut self) -> Result<String, TomlError> {
        let mut out = Vec::new();
        loop {
            match self.bump() {
                None | Some(b'\n') => return self.err("chuỗi chưa đóng"),
                Some(b'"') => return self.utf8(out),
                Some(b'\\') => self.escape(&mut out)?,
                Some(c) => out.push(c),
            }
        }
    }

    fn multiline_string(&mut self) -> Result<String, TomlError> {
        let mut out = Vec::new();
        loop {
            if self.peek() == Some(b'"')
                && self.peek_at(1) == Some(b'"')
                && self.peek_at(2) == Some(b'"')
            {
                self.i += 3;
                return self.utf8(out);
            }
            match self.bump() {
                None => return self.err("chuỗi nhiều dòng chưa đóng"),
                Some(b'\\') => self.escape(&mut out)?,
                Some(c) => out.push(c),
            }
        }
    }

    fn literal_string(&mut self) -> Result<String, TomlError> {
        let st = self.i;
        loop {
            match self.bump() {
                None | Some(b'\n') => return self.err("chuỗi chưa đóng"),
                Some(b'\'') => return self.utf8(self.s[st..self.i - 1].to_vec()),
                _ => {}
            }
        }
    }
}

fn insert(t: &mut Table, key: &[String], v: Value, line: usize) -> Result<(), TomlError> {
    let (last, parent) = key.split_last().unwrap();
    let t = descend(t, parent, line)?;
    if t.contains_key(last) {
        return Err(TomlError {
            line,
            msg: format!("khoá {:?} bị lặp", key.join(".")),
        });
    }
    t.insert(last.clone(), v);
    Ok(())
}

// ---------------- ghi ----------------

fn bare(k: &str) -> bool {
    !k.is_empty()
        && k.bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

fn key(k: &str) -> String {
    if bare(k) { k.to_string() } else { quote(k) }
}

pub fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\t' => o.push_str("\\t"),
            '\r' => o.push_str("\\r"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => o.push_str(&format!("\\u{:04X}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn inline(v: &Value) -> String {
    match v {
        Value::Str(s) => quote(s),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => {
            let s = format!("{f:?}");
            if s.contains(['.', 'e', 'E']) {
                s
            } else {
                format!("{s}.0")
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => format!("[{}]", a.iter().map(inline).collect::<Vec<_>>().join(", ")),
        Value::Table(t) => {
            if t.is_empty() {
                "{}".into()
            } else {
                format!(
                    "{{ {} }}",
                    t.iter()
                        .map(|(k, v)| format!("{} = {}", key(k), inline(v)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

fn is_aot(a: &[Value]) -> bool {
    !a.is_empty() && a.iter().all(|v| matches!(v, Value::Table(_)))
}

pub fn to_string(t: &Table) -> String {
    let mut out = String::new();
    write_table(&mut out, t, &mut Vec::new());
    out.trim_start_matches('\n').to_string()
}

fn write_table(out: &mut String, t: &Table, path: &mut Vec<String>) {
    for (k, v) in t {
        let nested = matches!(v, Value::Table(_)) || matches!(v, Value::Array(a) if is_aot(a));
        if !nested {
            out.push_str(&format!("{} = {}\n", key(k), inline(v)));
        }
    }
    for (k, v) in t {
        match v {
            Value::Table(sub) => {
                path.push(key(k));
                out.push_str(&format!("\n[{}]\n", path.join(".")));
                write_table(out, sub, path);
                path.pop();
            }
            Value::Array(a) if is_aot(a) => {
                path.push(key(k));
                for item in a {
                    out.push_str(&format!("\n[[{}]]\n", path.join(".")));
                    if let Value::Table(sub) = item {
                        write_table(out, sub, path);
                    }
                }
                path.pop();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_like_document() {
        let src = r#"
# chú thích
schema_version = "1.0.0"
id = "gem_on_curve"   # cuối dòng
license = ["SPDX:LGPL-2.1-or-later",
  "SPDX:MIT", # nhiều dòng
]
n = -12
f = 1.5e-1
ok = true
"key with space" = 'C:\raw\path'
tagline = "Đá \"tròn\" \u00e9"
permissions = { files = "Đọc thư viện đá", network = "Tải cập nhật" }
desc = """
dòng 1
dòng 2"""

[build]
paths_exclude_pattern = ["__pycache__/", "*.pyc"]

[[commands]]
id = "BuilderGemOnCrv"
ui.group = "Gems"

[[commands]]
id = "BuilderGemCountOnCrv"
"#;
        let t = parse(src).unwrap();
        assert_eq!(t["id"], Value::Str("gem_on_curve".into()));
        assert_eq!(t["n"], Value::Int(-12));
        assert_eq!(t["f"], Value::Float(0.15));
        assert_eq!(t["ok"], Value::Bool(true));
        assert_eq!(t["key with space"].as_str(), Some("C:\\raw\\path"));
        assert_eq!(t["tagline"].as_str(), Some("Đá \"tròn\" é"));
        assert_eq!(t["desc"].as_str(), Some("dòng 1\ndòng 2"));
        let Value::Array(lic) = &t["license"] else {
            panic!()
        };
        assert_eq!(lic.len(), 2);
        assert_eq!(
            t["permissions"].as_table().unwrap()["files"].as_str(),
            Some("Đọc thư viện đá")
        );
        let Value::Array(cmds) = &t["commands"] else {
            panic!()
        };
        assert_eq!(cmds.len(), 2);
        let c0 = cmds[0].as_table().unwrap();
        assert_eq!(c0["ui"].as_table().unwrap()["group"].as_str(), Some("Gems"));
        assert!(
            t["build"]
                .as_table()
                .unwrap()
                .contains_key("paths_exclude_pattern")
        );
    }

    #[test]
    fn crlf_and_bom() {
        let t = parse("\u{feff}a = 1\r\n[b]\r\nc = \"x\"\r\n").unwrap();
        assert_eq!(t["a"], Value::Int(1));
        assert_eq!(t["b"].as_table().unwrap()["c"].as_str(), Some("x"));
    }

    #[test]
    fn errors_have_lines() {
        let cases = [
            ("a = 1\na = 2", 2, "lặp"),
            ("a = \"x", 1, "chưa đóng"),
            ("[t]\n[t]", 2, "hai lần"),
            ("a = 0x10", 1, "chưa hỗ trợ"),
            ("a = 1 2", 1, "thừa"),
            ("a = 1.", 1, "số thực"),
            ("a = 01", 1, "bắt đầu bằng 0"),
            ("a = [1 2]", 1, "mảng"),
            ("x = 1\n[x]\n", 2, "không phải bảng"),
            ("a = 2020-01-01", 1, "số"),
        ];
        for (src, line, msg) in cases {
            let e = parse(src).unwrap_err();
            assert_eq!(e.line, line, "{src:?} -> {e}");
            assert!(e.msg.contains(msg), "{src:?} -> {e}");
        }
    }

    #[test]
    fn roundtrip() {
        let src = r#"
online_access = false
dev_paths = ["D:/dev/addons"]
[keymap]
BuilderGemOnCrv = "Ctrl+Shift+G"
[addons.gem_on_curve]
enabled = true
granted = []
[addons.gem_on_curve.settings]
gap = 0.2
diameter = 1.5
count = 3
name = "Tên \"đá\"\n"
[[list]]
a = 1
[[list]]
a = 2
"#;
        let t = parse(src).unwrap();
        let s = to_string(&t);
        assert_eq!(parse(&s).unwrap(), t, "{s}");
        // số thực luôn giữ dấu chấm
        let mut x = Table::new();
        x.insert("v".into(), Value::Float(3.0));
        assert_eq!(to_string(&x), "v = 3.0\n");
    }
}
