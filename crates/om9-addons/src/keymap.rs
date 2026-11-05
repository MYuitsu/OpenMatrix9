//! Chuẩn hoá phím tắt: "ctrl + shift+g" → "Ctrl+Shift+G".
//!
//! Quy tắc OM9: phím chữ/số/Space không có Ctrl/Alt bị từ chối vì sẽ cướp ô CMD
//! (người dùng gõ lệnh bằng bàn phím như Rhino).

const NAMED: &[(&str, &str)] = &[
    ("space", "Space"),
    ("enter", "Enter"),
    ("return", "Enter"),
    ("tab", "Tab"),
    ("backspace", "Backspace"),
    ("delete", "Delete"),
    ("del", "Delete"),
    ("insert", "Insert"),
    ("ins", "Insert"),
    ("home", "Home"),
    ("end", "End"),
    ("pageup", "PageUp"),
    ("pagedown", "PageDown"),
    ("up", "Up"),
    ("down", "Down"),
    ("left", "Left"),
    ("right", "Right"),
    ("esc", "Esc"),
    ("escape", "Esc"),
];

pub fn normalize(s: &str) -> Result<String, String> {
    let parts: Vec<String> = s
        .split('+')
        .map(|p| p.trim().to_ascii_lowercase())
        .collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(format!("phím tắt không hợp lệ: {s:?}"));
    }
    let (mut ctrl, mut alt, mut shift, mut meta) = (false, false, false, false);
    let mut key: Option<String> = None;
    for p in &parts {
        let flag = match p.as_str() {
            "ctrl" | "control" => Some(&mut ctrl),
            "alt" | "option" => Some(&mut alt),
            "shift" => Some(&mut shift),
            "meta" | "cmd" | "win" | "super" => Some(&mut meta),
            _ => None,
        };
        if let Some(f) = flag {
            if *f {
                return Err(format!("phím bổ trợ lặp trong {s:?}"));
            }
            *f = true;
            continue;
        }
        if key.is_some() {
            return Err(format!("chỉ được một phím chính trong {s:?}"));
        }
        let k = if p.len() == 1 && p.as_bytes()[0].is_ascii_alphanumeric() {
            p.to_ascii_uppercase()
        } else if let Some(n) = p.strip_prefix('f').and_then(|n| n.parse::<u8>().ok()) {
            if !(1..=24).contains(&n) {
                return Err(format!("phím F không hợp lệ: {p}"));
            }
            format!("F{n}")
        } else if let Some((_, v)) = NAMED.iter().find(|(k, _)| k == p) {
            v.to_string()
        } else {
            return Err(format!("phím không hỗ trợ: {p:?}"));
        };
        key = Some(k);
    }
    let key = key.ok_or_else(|| format!("thiếu phím chính trong {s:?}"))?;
    let printable = key.len() == 1 || key == "Space";
    if printable && !ctrl && !alt && !meta {
        return Err(format!(
            "{s:?} sẽ chặn việc gõ lệnh ở CMD; cần Ctrl hoặc Alt"
        ));
    }
    let mut out = Vec::new();
    if ctrl {
        out.push("Ctrl".to_string());
    }
    if alt {
        out.push("Alt".to_string());
    }
    if shift {
        out.push("Shift".to_string());
    }
    if meta {
        out.push("Meta".to_string());
    }
    out.push(key);
    Ok(out.join("+"))
}

/// Phím tắt OM9 giữ cho chính nó; add-on không được chiếm.
pub fn default_reserved() -> Vec<String> {
    [
        "Ctrl+Z",
        "Ctrl+Y",
        "Ctrl+S",
        "Ctrl+Shift+S",
        "Ctrl+O",
        "Ctrl+N",
        "Ctrl+C",
        "Ctrl+V",
        "Ctrl+X",
        "Ctrl+A",
        "Esc",
        "Delete",
        "Enter",
        "F1",
        "F6",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize;
    #[test]
    fn normalizes() {
        assert_eq!(normalize("shift + ctrl+g").unwrap(), "Ctrl+Shift+G");
        assert_eq!(normalize("ALT+f4").unwrap(), "Alt+F4");
        assert_eq!(normalize("F8").unwrap(), "F8");
        assert_eq!(normalize("ctrl+pagedown").unwrap(), "Ctrl+PageDown");
        assert_eq!(normalize("cmd+shift+1").unwrap(), "Shift+Meta+1");
    }
    #[test]
    fn rejects() {
        for bad in [
            "G",
            "shift+g",
            "Space",
            "ctrl+ctrl+g",
            "ctrl+g+h",
            "ctrl+",
            "F25",
            "ctrl+é",
            "ctrl",
        ] {
            assert!(normalize(bad).is_err(), "{bad}");
        }
    }
}
