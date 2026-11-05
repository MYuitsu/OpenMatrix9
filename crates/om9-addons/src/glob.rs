//! Khớp mẫu loại trừ khi build gói (giống `paths_exclude_pattern` của Blender, kiểu gitignore rút gọn).
//!
//! - `name/`  : khớp thư mục `name` ở bất kỳ cấp nào
//! - `*.pyc`  : không có `/` → khớp tên từng thành phần đường dẫn
//! - `/docs/*`: có `/` → khớp từ gốc gói
//! - Hỗ trợ `*` và `?` (không vượt qua `/`).

pub const DEFAULT_EXCLUDES: &[&str] = &[".*", "__pycache__/", "*.pyc", "*.zip"];

pub fn wildcard(pat: &str, s: &str) -> bool {
    fn wm(p: &[char], t: &[char]) -> bool {
        match p.first() {
            None => t.is_empty(),
            Some('*') => {
                let mut i = 0;
                loop {
                    if wm(&p[1..], &t[i..]) {
                        return true;
                    }
                    if i >= t.len() || t[i] == '/' {
                        return false;
                    }
                    i += 1;
                }
            }
            Some('?') => !t.is_empty() && t[0] != '/' && wm(&p[1..], &t[1..]),
            Some(c) => !t.is_empty() && t[0] == *c && wm(&p[1..], &t[1..]),
        }
    }
    let (p, t): (Vec<char>, Vec<char>) = (pat.chars().collect(), s.chars().collect());
    wm(&p, &t)
}

/// `rel` là đường dẫn tương đối kiểu POSIX của một file.
pub fn excluded(rel: &str, patterns: &[String]) -> bool {
    let parts: Vec<&str> = rel.split('/').collect();
    for pat in patterns {
        let pat = pat.trim();
        if pat.is_empty() {
            continue;
        }
        if let Some(dir) = pat.strip_suffix('/') {
            let dir = dir.trim_start_matches('/');
            // chỉ thành phần thư mục (không tính tên file cuối)
            if dir.contains('/') {
                if rel.starts_with(&format!("{dir}/")) {
                    return true;
                }
            } else if parts[..parts.len() - 1].iter().any(|c| wildcard(dir, c)) {
                return true;
            }
        } else if pat.contains('/') {
            if wildcard(pat.trim_start_matches('/'), rel) {
                return true;
            }
        } else if parts.iter().any(|c| wildcard(pat, c)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d() -> Vec<String> {
        DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn defaults() {
        assert!(excluded("__pycache__/a.cpython-313.pyc", &d()));
        assert!(excluded("sub/__pycache__/x.py", &d()));
        assert!(excluded(".git/config", &d()));
        assert!(excluded("icons/.DS_Store", &d()));
        assert!(excluded("x.pyc", &d()));
        assert!(excluded("old.zip", &d()));
        assert!(!excluded("__init__.py", &d()));
        assert!(!excluded("icons/gem.svg", &d()));
        assert!(!excluded("om9_addon.toml", &d()));
    }
    #[test]
    fn rooted_and_wildcards() {
        let p = vec!["/docs/*".to_string(), "test?.py".to_string()];
        assert!(excluded("docs/a.md", &p));
        assert!(!excluded("docs/sub/a.md", &p)); // * không vượt '/'
        assert!(!excluded("src/docs/a.md", &p));
        assert!(excluded("test1.py", &p));
        assert!(!excluded("test10.py", &p));
        assert!(wildcard("*.svg", "a.svg"));
        assert!(!wildcard("*.svg", "a/b.svg"));
    }
}
