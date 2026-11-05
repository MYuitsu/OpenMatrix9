// SPDX-License-Identifier: LGPL-2.1-or-later
//! Ngôn ngữ giao diện của add-on (Tiếng Việt / English).
//!
//! - **English** giữ nguyên chữ của lệnh (tên Matrix, tên tuỳ chọn, câu nhắc): không cần file.
//! - **Tiếng Việt** đọc bảng dịch `lang/vi.toml` mà add-on khai báo là asset `lang`:
//!
//! ```toml
//! [menus]     Curve = "Đường cong"
//! [commands]  CurveLineSingleLine = "Đường thẳng"
//! [options]   BothSides = "Hai phía"
//! [values]    Yes = "Có"
//! [phrases]   "Start of line" = "Điểm đầu đoạn thẳng"
//!             "{} holes" = "{} lỗ"          # {} giữ nguyên phần chữ ở chỗ đó
//! ```
//!
//! Tên tiếng Anh luôn gõ được: `BothSides`, `bothsides`, `Hai phía`, `haiphia`, `hai phia`
//! đều chọn cùng một tuỳ chọn. Câu chưa có trong bảng hiện nguyên văn tiếng Anh.
use crate::toml::{self, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Vi,
    En,
}

impl Lang {
    /// `vi`, `Tiếng Việt`, `en`, `English` …
    pub fn parse(s: &str) -> Option<Lang> {
        match fold(s).as_str() {
            "vi" | "vn" | "tiengviet" | "vietnamese" => Some(Lang::Vi),
            "en" | "english" | "tienganh" => Some(Lang::En),
            _ => None,
        }
    }
    pub fn code(self) -> &'static str {
        match self {
            Lang::Vi => "vi",
            Lang::En => "en",
        }
    }
}

/// One `{}` pattern: literal pieces between the holes.
#[derive(Debug, Clone)]
struct Pattern {
    parts: Vec<String>,
    target: String,
}

/// The active translation table.
#[derive(Debug, Clone, Default)]
pub struct Tr {
    lang: Option<Lang>,
    menus: BTreeMap<String, String>,
    commands: BTreeMap<String, String>,
    options: BTreeMap<String, String>,
    values: BTreeMap<String, String>,
    phrases: BTreeMap<String, String>,
    patterns: Vec<Pattern>,
}

fn table(t: &toml::Table, name: &str, errs: &mut Vec<String>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    match t.get(name) {
        None => {}
        Some(Value::Table(m)) => {
            for (k, v) in m {
                match v.as_str() {
                    Some(s) if !s.trim().is_empty() => {
                        out.insert(k.clone(), s.to_owned());
                    }
                    _ => errs.push(format!("[{name}] {k:?}: phải là chuỗi không rỗng")),
                }
            }
        }
        Some(_) => errs.push(format!("[{name}] phải là bảng")),
    }
    out
}

impl Tr {
    /// English: every text unchanged.
    pub fn english() -> Tr {
        Tr {
            lang: Some(Lang::En),
            ..Default::default()
        }
    }

    pub fn lang(&self) -> Lang {
        self.lang.unwrap_or(Lang::En)
    }

    /// Parse a `lang/<code>.toml` file. Every error is reported at once.
    pub fn parse(text: &str, lang: Lang) -> Result<Tr, Vec<String>> {
        let t = toml::parse(text).map_err(|e| vec![e.to_string()])?;
        let mut errs = Vec::new();
        for k in t.keys() {
            if !["menus", "commands", "options", "values", "phrases"].contains(&k.as_str()) {
                errs.push(format!(
                    "mục [{k}] không dùng (menus, commands, options, values, phrases)"
                ));
            }
        }
        let mut tr = Tr {
            lang: Some(lang),
            menus: table(&t, "menus", &mut errs),
            commands: table(&t, "commands", &mut errs),
            options: table(&t, "options", &mut errs),
            values: table(&t, "values", &mut errs),
            phrases: BTreeMap::new(),
            patterns: Vec::new(),
        };
        for (k, v) in table(&t, "phrases", &mut errs) {
            let holes = k.matches("{}").count();
            if holes == 0 {
                tr.phrases.insert(k, v);
                continue;
            }
            if v.matches("{}").count() != holes {
                errs.push(format!(
                    "[phrases] {k:?}: bản dịch phải có đúng {holes} chỗ {{}}"
                ));
                continue;
            }
            let parts: Vec<String> = k.split("{}").map(str::to_owned).collect();
            if parts.windows(2).any(|w| w[0].is_empty() && w[1].is_empty()) {
                errs.push(format!("[phrases] {k:?}: hai chỗ {{}} liền nhau"));
                continue;
            }
            tr.patterns.push(Pattern { parts, target: v });
        }
        // Longer patterns first: "{} stones selected" before "{} stones".
        tr.patterns
            .sort_by_key(|p| std::cmp::Reverse(p.parts.iter().map(String::len).sum::<usize>()));
        if errs.is_empty() { Ok(tr) } else { Err(errs) }
    }

    /// Add another add-on's table; entries already present win.
    pub fn merge(&mut self, other: Tr) {
        for (dst, src) in [
            (&mut self.menus, other.menus),
            (&mut self.commands, other.commands),
            (&mut self.options, other.options),
            (&mut self.values, other.values),
            (&mut self.phrases, other.phrases),
        ] {
            for (k, v) in src {
                dst.entry(k).or_insert(v);
            }
        }
        self.patterns.extend(other.patterns);
        self.patterns
            .sort_by_key(|p| std::cmp::Reverse(p.parts.iter().map(String::len).sum::<usize>()));
        if self.lang.is_none() {
            self.lang = other.lang;
        }
    }

    fn vi(&self) -> bool {
        self.lang() == Lang::Vi
    }

    /// Name of a main menu or category (`Curve`, `Gems` …).
    pub fn menu(&self, en: &str) -> String {
        self.pick(&self.menus, en).unwrap_or_else(|| en.to_owned())
    }

    /// Display name of a command: its catalog code looked up, else its English label.
    pub fn command(&self, code: &str, label: &str) -> String {
        self.pick(&self.commands, code)
            .unwrap_or_else(|| self.text(label))
    }

    /// Display name of a command-line option (`BothSides` → `Hai phía`).
    pub fn option(&self, name: &str) -> String {
        self.pick(&self.options, name)
            .unwrap_or_else(|| name.to_owned())
    }

    /// Display text of an option value (`Yes` → `Có`, `Uniform` → `Đều`).
    pub fn value(&self, v: &str) -> String {
        self.pick(&self.values, v)
            .or_else(|| self.pick(&self.options, v))
            .unwrap_or_else(|| v.to_owned())
    }

    /// `Name=value` or `Name` as shown above the command line.
    pub fn token(&self, name: &str, value: Option<&str>) -> String {
        match value {
            Some(v) => format!("{}={}", self.option(name), self.value(v)),
            None => self.option(name),
        }
    }

    fn pick(&self, m: &BTreeMap<String, String>, k: &str) -> Option<String> {
        if !self.vi() {
            return None;
        }
        m.get(k)
            .or_else(|| {
                // Option and value names are matched without case.
                m.iter()
                    .find(|(key, _)| key.eq_ignore_ascii_case(k))
                    .map(|(_, v)| v)
            })
            .cloned()
    }

    /// A prompt, history line or error. Whole phrases first; otherwise each part between
    /// `: `, `. `, `; ` and a trailing `<default>` / `(…)` is translated on its own.
    pub fn text(&self, s: &str) -> String {
        if !self.vi() || s.is_empty() {
            return s.to_owned();
        }
        self.text_depth(s, 0)
    }

    fn text_depth(&self, s: &str, depth: usize) -> String {
        if let Some(t) = self.whole(s, depth) {
            return t;
        }
        // Split at the first separator that gives two non-empty halves.
        for sep in [": ", ". ", "; ", ", "] {
            if let Some((a, b)) = s.split_once(sep)
                && !a.trim().is_empty()
                && !b.trim().is_empty()
            {
                return format!(
                    "{}{sep}{}",
                    self.text_depth(a, depth),
                    self.text_depth(b, depth)
                );
            }
        }
        let trimmed = s.trim_end();
        for (open, close) in [(" <", '>'), (" (", ')')] {
            if trimmed.ends_with(close)
                && let Some(i) = trimmed.rfind(open)
                && i > 0
            {
                let inner = &trimmed[i + open.len()..trimmed.len() - 1];
                let inner = if open == " (" {
                    self.text_depth(inner, depth)
                } else {
                    inner.to_owned()
                };
                return format!(
                    "{}{open}{inner}{close}",
                    self.text_depth(&trimmed[..i], depth)
                );
            }
        }
        s.to_owned()
    }

    /// The translation of `s` as one phrase or pattern, if any.
    fn whole(&self, s: &str, depth: usize) -> Option<String> {
        let key = s.trim();
        if key.is_empty() {
            return None;
        }
        let lead = &s[..s.len() - s.trim_start().len()];
        let tail = &s[s.trim_end().len()..];
        if let Some(t) = self
            .phrases
            .get(key)
            .or_else(|| self.commands.get(key))
            .or_else(|| self.menus.get(key))
        {
            return Some(format!("{lead}{t}{tail}"));
        }
        // A command label inside a prompt: "Single Line: …".
        if let Some(t) = self.options.get(key).or_else(|| self.values.get(key)) {
            return Some(format!("{lead}{t}{tail}"));
        }
        if depth > 2 {
            return None;
        }
        for p in &self.patterns {
            if let Some(holes) = capture(key, &p.parts) {
                let mut out = String::new();
                let mut it = p.target.split("{}");
                out.push_str(it.next().unwrap_or(""));
                for (h, rest) in holes.iter().zip(it) {
                    out.push_str(&self.text_depth(h, depth + 1));
                    out.push_str(rest);
                }
                return Some(format!("{lead}{out}{tail}"));
            }
        }
        None
    }

    /// Whether typed text `typed` names option `name` (English name or its translation,
    /// without case, accents or spaces).
    pub fn names(&self, typed: &str, name: &str) -> bool {
        let t = fold(typed);
        !t.is_empty()
            && (t == fold(name)
                || self
                    .pick(&self.options, name)
                    .is_some_and(|v| fold(&v) == t))
    }

    /// Whether `typed` starts option `name` (English or translated), for `bo` → `BothSides`.
    pub fn starts(&self, typed: &str, name: &str) -> bool {
        let t = fold(typed);
        !t.is_empty()
            && (fold(name).starts_with(&t)
                || self
                    .pick(&self.options, name)
                    .is_some_and(|v| fold(&v).starts_with(&t)))
    }

    /// The English value of a typed value: `Có` → `Yes`, `deu` → `Uniform`; unknown text
    /// unchanged.
    pub fn english_value(&self, typed: &str) -> String {
        if self.vi() {
            let t = fold(typed);
            for (en, vi) in self.values.iter().chain(self.options.iter()) {
                if fold(vi) == t {
                    return en.clone();
                }
            }
        }
        typed.to_owned()
    }

    /// How many entries the table has (0 for English).
    pub fn len(&self) -> usize {
        self.menus.len()
            + self.commands.len()
            + self.options.len()
            + self.values.len()
            + self.phrases.len()
            + self.patterns.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The text in each `{}` hole when `s` matches the pattern parts, else `None`.
fn capture(s: &str, parts: &[String]) -> Option<Vec<String>> {
    let first = parts.first()?;
    let mut rest = s.strip_prefix(first.as_str())?;
    let mut holes = Vec::new();
    for (i, part) in parts.iter().enumerate().skip(1) {
        let last = i == parts.len() - 1;
        let (hole, after) = if last {
            if part.is_empty() {
                (rest, "")
            } else {
                let h = rest.strip_suffix(part.as_str())?;
                (h, "")
            }
        } else {
            let at = rest.find(part.as_str())?;
            (&rest[..at], &rest[at + part.len()..])
        };
        if hole.trim().is_empty() {
            return None;
        }
        holes.push(hole.to_owned());
        rest = after;
    }
    Some(holes)
}

/// Lowercase without Vietnamese accents, spaces or punctuation: `Hai phía` → `haiphia`.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars().flat_map(char::to_lowercase) {
        let base = match c {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ'
            | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => {
                'e'
            }
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ'
            | 'ở' | 'ỡ' | 'ợ' => 'o',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => {
                'u'
            }
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            'đ' => 'd',
            c => c,
        };
        if base.is_alphanumeric() {
            out.push(base);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const VI: &str = r#"
[menus]
Curve = "Đường cong"
[commands]
CurveLineSingleLine = "Đường thẳng"
[options]
BothSides = "Hai phía"
Knots = "Nút"
Uniform = "Đều"
[values]
Yes = "Có"
No = "Không"
[phrases]
"Start of line" = "Điểm đầu đoạn thẳng"
"Single Line" = "Đường thẳng"
"Radius" = "Bán kính"
"{} holes" = "{} lỗ"
"select {} stones, Enter" = "chọn {} viên đá, Enter"
"#;

    fn vi() -> Tr {
        Tr::parse(VI, Lang::Vi).unwrap()
    }

    #[test]
    fn english_keeps_every_text() {
        let tr = Tr::english();
        assert_eq!(tr.menu("Curve"), "Curve");
        assert_eq!(
            tr.command("CurveLineSingleLine", "Single Line"),
            "Single Line"
        );
        assert_eq!(tr.token("BothSides", Some("No")), "BothSides=No");
        assert_eq!(tr.text("Start of line"), "Start of line");
        assert!(tr.names("bothsides", "BothSides"));
    }

    #[test]
    fn vietnamese_menus_commands_tokens() {
        let tr = vi();
        assert_eq!(tr.menu("Curve"), "Đường cong");
        assert_eq!(tr.menu("Surface"), "Surface");
        assert_eq!(
            tr.command("CurveLineSingleLine", "Single Line"),
            "Đường thẳng"
        );
        assert_eq!(tr.command("Unknown", "Radius"), "Bán kính");
        assert_eq!(tr.token("BothSides", Some("No")), "Hai phía=Không");
        assert_eq!(tr.token("Knots", Some("Uniform")), "Nút=Đều");
        assert_eq!(tr.token("Undo", None), "Undo");
    }

    #[test]
    fn prompts_are_translated_by_parts() {
        let tr = vi();
        assert_eq!(tr.text("Start of line"), "Điểm đầu đoạn thẳng");
        assert_eq!(
            tr.text("Single Line: Start of line"),
            "Đường thẳng: Điểm đầu đoạn thẳng"
        );
        assert_eq!(tr.text("Radius <2.5>"), "Bán kính <2.5>");
        assert_eq!(tr.text("Array Hole: 12 holes"), "Array Hole: 12 lỗ");
        assert_eq!(tr.text("select 3 stones, Enter"), "chọn 3 viên đá, Enter");
        assert_eq!(tr.text("Something new"), "Something new");
    }

    #[test]
    fn typed_names_match_english_and_vietnamese() {
        let tr = vi();
        for typed in ["BothSides", "bothsides", "Hai phía", "hai phia", "HaiPhia"] {
            assert!(tr.names(typed, "BothSides"), "{typed}");
        }
        assert!(!tr.names("Hai", "BothSides"));
        assert!(tr.starts("hai", "BothSides"));
        assert!(tr.starts("bo", "BothSides"));
        assert_eq!(tr.english_value("Có"), "Yes");
        assert_eq!(tr.english_value("khong"), "No");
        assert_eq!(tr.english_value("deu"), "Uniform");
        assert_eq!(tr.english_value("2.5"), "2.5");
    }

    #[test]
    fn bad_tables_report_every_error() {
        let bad = "[phrases]\n\"{} holes\" = \"lỗ\"\n\"{}{}\" = \"{}{}\"\n[other]\na = \"b\"\n";
        let errs = Tr::parse(bad, Lang::Vi).unwrap_err();
        assert_eq!(errs.len(), 3, "{errs:?}");
    }

    #[test]
    fn fold_removes_accents() {
        assert_eq!(fold("Đường thẳng"), "duongthang");
        assert_eq!(fold("Điểm đầu"), "diemdau");
        assert_eq!(Lang::parse("Tiếng Việt"), Some(Lang::Vi));
        assert_eq!(Lang::parse("English"), Some(Lang::En));
    }
}
