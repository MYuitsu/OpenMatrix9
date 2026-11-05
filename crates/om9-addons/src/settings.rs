//! Settings khai báo trong manifest (`[[settings]]`).
//!
//! Khác Blender (AddonPreferences phải viết bằng Python), OM9 đọc schema từ manifest nên:
//! - vẽ được UI cài đặt ngay cả khi add-on đang tắt hoặc runtime chưa có,
//! - kiểm tra kiểu/khoảng giá trị trước khi lưu,
//! - di trú giá trị khi nâng cấp add-on (giữ giá trị còn hợp lệ, trả về mặc định nếu không).

use crate::toml::{Table, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
}

impl SettingValue {
    pub fn from_toml(v: &Value) -> Option<SettingValue> {
        Some(match v {
            Value::Bool(b) => SettingValue::Bool(*b),
            Value::Int(i) => SettingValue::Int(*i),
            Value::Float(f) => SettingValue::Float(*f),
            Value::Str(s) => SettingValue::Str(s.clone()),
            _ => return None,
        })
    }
    pub fn to_toml(&self) -> Value {
        match self {
            SettingValue::Bool(b) => Value::Bool(*b),
            SettingValue::Int(i) => Value::Int(*i),
            SettingValue::Float(f) => Value::Float(*f),
            SettingValue::Str(s) => Value::Str(s.clone()),
        }
    }
    pub fn display(&self) -> String {
        match self {
            SettingValue::Bool(b) => b.to_string(),
            SettingValue::Int(i) => i.to_string(),
            SettingValue::Float(f) => format!("{f}"),
            SettingValue::Str(s) => s.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    None,
    Mm,
    Deg,
    Percent,
}

impl Unit {
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::None => "",
            Unit::Mm => " mm",
            Unit::Deg => "°",
            Unit::Percent => " %",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Bool,
    Int {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        unit: Unit,
    },
    Enum {
        choices: Vec<String>,
    },
    Str {
        max_len: usize,
    },
    Path {
        dir: bool,
    },
    Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingSpec {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub kind: Kind,
    pub default: SettingValue,
    /// Command-line option name (`Size` in `Size=1.5`). Defaults to the key in PascalCase.
    pub option: String,
    /// Display text for each enum choice, same order as `choices` (empty = show the choice).
    pub choice_labels: Vec<String>,
    /// The commands this setting belongs to (their builder panel and command-line options).
    /// Empty: every command of the add-on.
    pub commands: Vec<String>,
    /// `scope = "addon"`: a setting of the whole add-on (its language …), shown only in
    /// Preferences › Add-ons, never in a command's panel or command line.
    pub addon_wide: bool,
}

impl SettingSpec {
    /// Whether this setting is one of `command`'s parameters.
    pub fn applies_to(&self, command: &str) -> bool {
        !self.addon_wide && (self.commands.is_empty() || self.commands.iter().any(|c| c == command))
    }
}

/// `z_offset` → `ZOffset`.
pub fn pascal(key: &str) -> String {
    key.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            c.next()
                .map(|f| f.to_ascii_uppercase().to_string() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

fn valid_option(o: &str) -> bool {
    let b = o.as_bytes();
    !b.is_empty()
        && b.len() <= 32
        && b[0].is_ascii_alphabetic()
        && b.iter().all(u8::is_ascii_alphanumeric)
}

pub const TYPES: &[&str] = &[
    "bool", "int", "float", "length", "angle", "percent", "enum", "string", "path", "dir", "color",
];

fn valid_key(k: &str) -> bool {
    let b = k.as_bytes();
    !b.is_empty()
        && b.len() <= 48
        && b[0].is_ascii_lowercase()
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}

fn num(t: &Table, k: &str, errs: &mut Vec<String>, ctx: &str) -> Option<f64> {
    match t.get(k) {
        None => None,
        Some(Value::Int(i)) => Some(*i as f64),
        Some(Value::Float(f)) => Some(*f),
        Some(v) => {
            errs.push(format!(
                "{ctx}: {k} phải là số, không phải {}",
                v.type_name()
            ));
            None
        }
    }
}

impl SettingSpec {
    pub fn from_table(t: &Table, idx: usize, errs: &mut Vec<String>) -> Option<SettingSpec> {
        let key = t
            .get("key")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let ctx = if key.is_empty() {
            format!("settings[{idx}]")
        } else {
            format!("settings.{key}")
        };
        let n0 = errs.len();
        if !valid_key(&key) {
            errs.push(format!(
                "{ctx}: key không hợp lệ {key:?} (chữ thường, số, _; ≤48 ký tự)"
            ));
        }
        let ty = t.get("type").and_then(Value::as_str).unwrap_or("");
        let (minf, maxf) = (num(t, "min", errs, &ctx), num(t, "max", errs, &ctx));
        if let (Some(a), Some(b)) = (minf, maxf)
            && a > b
        {
            errs.push(format!("{ctx}: min > max"));
        }
        let kind = match ty {
            "bool" => Kind::Bool,
            "int" => {
                let as_int = |f: Option<f64>, n: &str, errs: &mut Vec<String>| match f {
                    Some(x) if x.fract() != 0.0 => {
                        errs.push(format!("{ctx}: {n} của kiểu int phải là số nguyên"));
                        None
                    }
                    Some(x) => Some(x as i64),
                    None => None,
                };
                Kind::Int {
                    min: as_int(minf, "min", errs),
                    max: as_int(maxf, "max", errs),
                }
            }
            "float" | "length" | "angle" | "percent" => {
                let step = num(t, "step", errs, &ctx);
                if step.is_some_and(|s| s <= 0.0) {
                    errs.push(format!("{ctx}: step phải > 0"));
                }
                let unit = match ty {
                    "length" => Unit::Mm,
                    "angle" => Unit::Deg,
                    "percent" => Unit::Percent,
                    _ => Unit::None,
                };
                Kind::Float {
                    min: minf,
                    max: maxf,
                    step,
                    unit,
                }
            }
            "enum" => {
                let choices: Vec<String> = match t.get("choices") {
                    Some(Value::Array(a)) => a
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect(),
                    _ => Vec::new(),
                };
                let mut uniq = choices.clone();
                uniq.sort();
                uniq.dedup();
                if choices.is_empty()
                    || uniq.len() != choices.len()
                    || choices.iter().any(|c| c.is_empty())
                {
                    errs.push(format!(
                        "{ctx}: enum cần choices là danh sách chuỗi không rỗng, không trùng"
                    ));
                }
                Kind::Enum { choices }
            }
            "string" => Kind::Str {
                max_len: maxf.map(|m| m as usize).unwrap_or(256),
            },
            "path" => Kind::Path { dir: false },
            "dir" => Kind::Path { dir: true },
            "color" => Kind::Color,
            other => {
                errs.push(format!(
                    "{ctx}: type {other:?} không hợp lệ, cần một trong {}",
                    TYPES.join(", ")
                ));
                Kind::Bool
            }
        };
        let label = t
            .get("label")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| key.clone());
        let description = t
            .get("description")
            .and_then(Value::as_str)
            .map(str::to_string);
        let option = match t.get("option") {
            None => pascal(&key),
            Some(Value::Str(o)) if valid_option(o) => o.clone(),
            Some(_) => {
                errs.push(format!(
                    "{ctx}: option phải là tên lệnh-dòng 1–32 ký tự chữ/số, bắt đầu bằng chữ"
                ));
                String::new()
            }
        };
        let choice_labels: Vec<String> = match t.get("choice_labels") {
            None => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            Some(_) => {
                errs.push(format!("{ctx}: choice_labels phải là danh sách chuỗi"));
                Vec::new()
            }
        };
        let commands: Vec<String> = match t.get("commands") {
            None => Vec::new(),
            Some(Value::Array(a)) if a.iter().all(|v| v.as_str().is_some()) => a
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            Some(_) => {
                errs.push(format!("{ctx}: commands phải là danh sách mã lệnh"));
                Vec::new()
            }
        };
        let addon_wide = match t.get("scope").map(|v| v.as_str()) {
            None | Some(Some("command")) => false,
            Some(Some("addon")) => {
                if !commands.is_empty() {
                    errs.push(format!("{ctx}: scope = \"addon\" không đi cùng commands"));
                }
                true
            }
            _ => {
                errs.push(format!("{ctx}: scope phải là \"command\" hoặc \"addon\""));
                false
            }
        };
        if !choice_labels.is_empty()
            && !matches!(&kind, Kind::Enum { choices } if choices.len() == choice_labels.len())
        {
            errs.push(format!(
                "{ctx}: choice_labels chỉ dùng cho enum và phải có đúng số phần tử như choices"
            ));
        }
        let mut spec = SettingSpec {
            key,
            label,
            description,
            kind,
            default: SettingValue::Bool(false),
            option,
            choice_labels,
            commands,
            addon_wide,
        };
        match t.get("default").and_then(SettingValue::from_toml) {
            None => errs.push(format!("{ctx}: thiếu default")),
            Some(d) => match spec.validate(&d) {
                Ok(v) => spec.default = v,
                Err(e) => errs.push(format!("{ctx}: default {e}")),
            },
        }
        if errs.len() > n0 { None } else { Some(spec) }
    }

    /// Kiểm tra và chuẩn hoá một giá trị.
    pub fn validate(&self, v: &SettingValue) -> Result<SettingValue, String> {
        use SettingValue as V;
        match (&self.kind, v) {
            (Kind::Bool, V::Bool(b)) => Ok(V::Bool(*b)),
            (Kind::Int { min, max }, V::Int(i)) => {
                if min.is_some_and(|m| *i < m) || max.is_some_and(|m| *i > m) {
                    return Err(format!(
                        "{i} ngoài khoảng {}",
                        range(min.map(|x| x as f64), max.map(|x| x as f64), Unit::None)
                    ));
                }
                Ok(V::Int(*i))
            }
            (Kind::Float { min, max, unit, .. }, V::Int(_) | V::Float(_)) => {
                let f = match v {
                    V::Int(i) => *i as f64,
                    V::Float(f) => *f,
                    _ => unreachable!(),
                };
                if !f.is_finite() {
                    return Err("phải là số hữu hạn".into());
                }
                if min.is_some_and(|m| f < m) || max.is_some_and(|m| f > m) {
                    return Err(format!(
                        "{f}{} ngoài khoảng {}",
                        unit.suffix(),
                        range(*min, *max, *unit)
                    ));
                }
                Ok(V::Float(f))
            }
            (Kind::Enum { choices }, V::Str(s)) => {
                if choices.contains(s) {
                    Ok(V::Str(s.clone()))
                } else {
                    Err(format!("{s:?} không thuộc {choices:?}"))
                }
            }
            (Kind::Str { max_len }, V::Str(s)) => {
                if s.chars().count() > *max_len {
                    Err(format!("dài quá {max_len} ký tự"))
                } else if s.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
                    Err("chứa ký tự điều khiển".into())
                } else {
                    Ok(V::Str(s.clone()))
                }
            }
            (Kind::Path { .. }, V::Str(s)) => {
                if s.len() > 1024 || s.contains('\0') {
                    Err("đường dẫn không hợp lệ".into())
                } else {
                    Ok(V::Str(s.clone()))
                }
            }
            (Kind::Color, V::Str(s)) => {
                let ok = s.len() == 7
                    && s.starts_with('#')
                    && s[1..].bytes().all(|b| b.is_ascii_hexdigit());
                if ok {
                    Ok(V::Str(s.to_ascii_uppercase()))
                } else {
                    Err(format!("màu phải dạng #RRGGBB: {s:?}"))
                }
            }
            (k, v) => Err(format!(
                "sai kiểu: cần {}, nhận {}",
                kind_name(k),
                v.display()
            )),
        }
    }

    /// Đọc giá trị người dùng gõ (CLI/ô nhập UI). Chấp nhận dấu phẩy thập phân "1,5".
    pub fn parse_user(&self, s: &str) -> Result<SettingValue, String> {
        let s = s.trim();
        let v = match &self.kind {
            Kind::Bool => match s.to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" | "co" | "có" => SettingValue::Bool(true),
                "false" | "0" | "no" | "off" | "khong" | "không" => SettingValue::Bool(false),
                _ => return Err(format!("{s:?} không phải bool")),
            },
            Kind::Int { .. } => SettingValue::Int(
                s.parse()
                    .map_err(|_| format!("{s:?} không phải số nguyên"))?,
            ),
            Kind::Float { .. } => {
                let t = s
                    .trim_end_matches("mm")
                    .trim_end_matches('°')
                    .trim_end_matches('%')
                    .trim()
                    .replace(',', ".");
                SettingValue::Float(t.parse().map_err(|_| format!("{s:?} không phải số"))?)
            }
            _ => SettingValue::Str(s.to_string()),
        };
        self.validate(&v)
    }
}

fn kind_name(k: &Kind) -> &'static str {
    match k {
        Kind::Bool => "bool",
        Kind::Int { .. } => "số nguyên",
        Kind::Float { .. } => "số",
        Kind::Enum { .. } => "lựa chọn",
        Kind::Str { .. } => "chuỗi",
        Kind::Path { .. } => "đường dẫn",
        Kind::Color => "màu",
    }
}

fn range(min: Option<f64>, max: Option<f64>, u: Unit) -> String {
    let f = |x: Option<f64>| x.map(|v| format!("{v}")).unwrap_or_else(|| "∞".into());
    format!("[{}, {}]{}", f(min), f(max), u.suffix())
}

/// Giá trị hiệu lực + ghi chú di trú.
pub fn resolve(
    specs: &[SettingSpec],
    stored: &BTreeMap<String, SettingValue>,
) -> (BTreeMap<String, SettingValue>, Vec<String>) {
    let mut out = BTreeMap::new();
    let mut notes = Vec::new();
    for s in specs {
        let v = match stored.get(&s.key) {
            None => s.default.clone(),
            Some(v) => match s.validate(v) {
                Ok(v) => v,
                Err(e) => {
                    notes.push(format!(
                        "{}: giá trị cũ {} không còn hợp lệ ({e}), dùng mặc định {}",
                        s.key,
                        v.display(),
                        s.default.display()
                    ));
                    s.default.clone()
                }
            },
        };
        out.insert(s.key.clone(), v);
    }
    for k in stored.keys() {
        if !specs.iter().any(|s| &s.key == k) {
            notes.push(format!("{k}: không còn trong add-on, đã bỏ"));
        }
    }
    (out, notes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toml::parse;

    fn spec(src: &str) -> Result<SettingSpec, Vec<String>> {
        let t = parse(src).unwrap();
        let mut e = Vec::new();
        SettingSpec::from_table(&t, 0, &mut e).ok_or(e)
    }

    #[test]
    fn command_line_option_names_and_choice_labels() {
        let s = spec("key='z_offset'\ntype='length'\ndefault=0").unwrap();
        assert_eq!(s.option, "ZOffset", "PascalCase of the key by default");
        let s = spec("key='diameter'\ntype='length'\ndefault=1\noption='Size'").unwrap();
        assert_eq!(s.option, "Size");
        assert!(spec("key='a'\ntype='bool'\ndefault=true\noption='bad name'").is_err());
        let s = spec("key='layout'\ntype='enum'\nchoices=['fit','fixed_gap']\nchoice_labels=['Fit','Fixed']\ndefault='fit'").unwrap();
        assert_eq!(s.choice_labels, ["Fit", "Fixed"]);
        assert!(spec("key='layout'\ntype='enum'\nchoices=['fit','fixed_gap']\nchoice_labels=['Fit']\ndefault='fit'").is_err());
        assert!(spec("key='n'\ntype='int'\ndefault=1\nchoice_labels=['x']").is_err());
    }

    #[test]
    fn length_setting() {
        let s = spec("key='gap'\ntype='length'\ndefault=0.2\nmin=0\nmax=2\nstep=0.05").unwrap();
        assert_eq!(
            s.kind,
            Kind::Float {
                min: Some(0.0),
                max: Some(2.0),
                step: Some(0.05),
                unit: Unit::Mm
            }
        );
        assert_eq!(s.parse_user("0,35 mm").unwrap(), SettingValue::Float(0.35));
        assert_eq!(
            s.validate(&SettingValue::Int(1)).unwrap(),
            SettingValue::Float(1.0)
        );
        assert!(s.parse_user("3").unwrap_err().contains("ngoài khoảng"));
        assert!(s.validate(&SettingValue::Str("x".into())).is_err());
    }

    #[test]
    fn spec_errors() {
        assert!(spec("key='Gap'\ntype='length'\ndefault=1").unwrap_err()[0].contains("key"));
        assert!(
            spec("key='g'\ntype='length'\ndefault=5\nmax=2").unwrap_err()[0].contains("default")
        );
        assert!(
            spec("key='g'\ntype='enum'\nchoices=['a','a']\ndefault='a'").unwrap_err()[0]
                .contains("trùng")
        );
        assert!(
            spec("key='g'\ntype='enum'\nchoices=['a','b']\ndefault='c'").unwrap_err()[0]
                .contains("default")
        );
        assert!(
            spec("key='g'\ntype='int'\ndefault=1\nmin=0.5").unwrap_err()[0].contains("số nguyên")
        );
        assert!(spec("key='g'\ntype='vector'\ndefault=1").unwrap_err()[0].contains("type"));
        assert!(spec("key='g'\ntype='bool'").unwrap_err()[0].contains("thiếu default"));
    }

    #[test]
    fn color_and_enum() {
        let c = spec("key='c'\ntype='color'\ndefault='#ffcc00'").unwrap();
        assert_eq!(c.default, SettingValue::Str("#FFCC00".into()));
        assert!(c.parse_user("red").is_err());
        let e =
            spec("key='m'\ntype='enum'\nchoices=['tangent','normal']\ndefault='normal'").unwrap();
        assert!(e.parse_user("tangent").is_ok());
        assert!(e.parse_user("x").is_err());
        let b = spec("key='b'\ntype='bool'\ndefault=true").unwrap();
        assert_eq!(b.parse_user("không").unwrap(), SettingValue::Bool(false));
    }

    #[test]
    fn migration() {
        let specs = vec![
            spec("key='gap'\ntype='length'\ndefault=0.2\nmin=0\nmax=1").unwrap(),
            spec("key='count'\ntype='int'\ndefault=3\nmin=1").unwrap(),
        ];
        let mut stored = BTreeMap::new();
        stored.insert("gap".into(), SettingValue::Float(5.0)); // nay vượt max
        stored.insert("count".into(), SettingValue::Int(7)); // còn hợp lệ
        stored.insert("old".into(), SettingValue::Bool(true)); // khoá đã bỏ
        let (v, notes) = resolve(&specs, &stored);
        assert_eq!(v["gap"], SettingValue::Float(0.2));
        assert_eq!(v["count"], SettingValue::Int(7));
        assert!(!v.contains_key("old"));
        assert_eq!(notes.len(), 2);
    }
}
