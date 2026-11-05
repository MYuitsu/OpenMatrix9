//! Phiên bản X.Y.Z (không hỗ trợ pre-release; giống ràng buộc của Blender manifest).

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    pub fn parse(s: &str) -> Result<Version, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("phiên bản không hợp lệ (cần X.Y.Z): {s:?}"));
        }
        let mut n = [0u64; 3];
        for (i, p) in parts.iter().enumerate() {
            if p.is_empty()
                || !p.bytes().all(|b| b.is_ascii_digit())
                || (p.len() > 1 && p.starts_with('0'))
            {
                return Err(format!("phiên bản không hợp lệ (cần X.Y.Z): {s:?}"));
            }
            n[i] = p.parse().map_err(|_| format!("phiên bản quá lớn: {s:?}"))?;
        }
        Ok(Version(n[0], n[1], n[2]))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    #[test]
    fn parse_and_order() {
        assert_eq!(Version::parse("1.2.3").unwrap(), Version(1, 2, 3));
        assert!(Version::parse("0.10.0").unwrap() > Version::parse("0.9.9").unwrap());
        for bad in ["1.2", "1.2.3.4", "a.b.c", "01.2.3", "1.2.-3", "", "1..3"] {
            assert!(Version::parse(bad).is_err(), "{bad}");
        }
    }
}
