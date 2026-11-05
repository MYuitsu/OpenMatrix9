//! Đọc/ghi ZIP tối giản, an toàn cho gói add-on.
//!
//! Đọc: method 0 (stored) và 8 (deflate), kiểm tra CRC-32, từ chối mã hoá, ZIP64,
//! tên không phải UTF-8. Ghi: method 0, thứ tự và thời gian cố định → build tất định.

use crate::crc32::crc32;
use crate::inflate::inflate;

#[derive(Debug, Clone)]
pub struct Entry {
    pub name: String,
    pub method: u16,
    pub flags: u16,
    pub crc: u32,
    pub compressed: u64,
    pub size: u64,
    pub local_offset: u64,
    pub unix_mode: u32,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        self.name.ends_with('/')
    }
    pub fn is_symlink(&self) -> bool {
        self.unix_mode & 0o170000 == 0o120000
    }
}

pub struct ZipArchive {
    data: Vec<u8>,
    pub entries: Vec<Entry>,
}

fn u16_at(d: &[u8], o: usize) -> Result<u16, String> {
    d.get(o..o + 2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .ok_or_else(|| "zip bị cụt".to_string())
}
fn u32_at(d: &[u8], o: usize) -> Result<u32, String> {
    d.get(o..o + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or_else(|| "zip bị cụt".to_string())
}

impl ZipArchive {
    pub fn open(data: Vec<u8>) -> Result<ZipArchive, String> {
        if data.len() < 22 {
            return Err("không phải file zip hợp lệ".into());
        }
        let min = data.len().saturating_sub(22 + 0xFFFF);
        let mut eocd = None;
        let mut p = data.len() - 22;
        loop {
            if u32_at(&data, p)? == 0x0605_4b50 {
                eocd = Some(p);
                break;
            }
            if p == min {
                break;
            }
            p -= 1;
        }
        let e = eocd.ok_or("không phải file zip hợp lệ (không thấy EOCD)")?;
        let n = u16_at(&data, e + 10)? as usize;
        let cd_size = u32_at(&data, e + 12)? as usize;
        let cd_off = u32_at(&data, e + 16)? as usize;
        if n == 0xFFFF || cd_off == 0xFFFF_FFFF || cd_size == 0xFFFF_FFFF {
            return Err("ZIP64 chưa được hỗ trợ".into());
        }
        if u16_at(&data, e + 4)? != 0 || u16_at(&data, e + 6)? != 0 {
            return Err("zip nhiều phần chưa được hỗ trợ".into());
        }
        if cd_off.checked_add(cd_size).is_none_or(|end| end > e) {
            return Err("central directory nằm ngoài file".into());
        }
        let mut entries = Vec::with_capacity(n);
        let mut o = cd_off;
        for _ in 0..n {
            if u32_at(&data, o)? != 0x0201_4b50 {
                return Err("central directory hỏng".into());
            }
            let flags = u16_at(&data, o + 8)?;
            let method = u16_at(&data, o + 10)?;
            let crc = u32_at(&data, o + 16)?;
            let csize = u32_at(&data, o + 20)?;
            let usize_ = u32_at(&data, o + 24)?;
            let nlen = u16_at(&data, o + 28)? as usize;
            let xlen = u16_at(&data, o + 30)? as usize;
            let clen = u16_at(&data, o + 32)? as usize;
            let made_by_os = (u16_at(&data, o + 4)? >> 8) as u8;
            let ext = u32_at(&data, o + 38)?;
            let lofs = u32_at(&data, o + 42)?;
            if csize == 0xFFFF_FFFF || usize_ == 0xFFFF_FFFF || lofs == 0xFFFF_FFFF {
                return Err("ZIP64 chưa được hỗ trợ".into());
            }
            let raw = data.get(o + 46..o + 46 + nlen).ok_or("tên file bị cụt")?;
            let name = String::from_utf8(raw.to_vec())
                .map_err(|_| "tên file trong zip không phải UTF-8".to_string())?;
            // chỉ tin mode unix khi file tạo trên Unix (3) hoặc macOS (19)
            let unix_mode = if made_by_os == 3 || made_by_os == 19 {
                ext >> 16
            } else {
                0
            };
            entries.push(Entry {
                name,
                method,
                flags,
                crc,
                compressed: csize as u64,
                size: usize_ as u64,
                local_offset: lofs as u64,
                unix_mode,
            });
            o += 46 + nlen + xlen + clen;
        }
        Ok(ZipArchive { data, entries })
    }

    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }

    pub fn read(&self, e: &Entry) -> Result<Vec<u8>, String> {
        if e.flags & 1 != 0 {
            return Err(format!("{}: file mã hoá không được hỗ trợ", e.name));
        }
        let o = e.local_offset as usize;
        if u32_at(&self.data, o)? != 0x0403_4b50 {
            return Err(format!("{}: local header hỏng", e.name));
        }
        let nlen = u16_at(&self.data, o + 26)? as usize;
        let xlen = u16_at(&self.data, o + 28)? as usize;
        let start = o + 30 + nlen + xlen;
        let comp = self
            .data
            .get(start..start + e.compressed as usize)
            .ok_or_else(|| format!("{}: dữ liệu bị cụt", e.name))?;
        let out = match e.method {
            0 => {
                if e.compressed != e.size {
                    return Err(format!("{}: kích thước stored không khớp", e.name));
                }
                comp.to_vec()
            }
            8 => inflate(comp, e.size as usize).map_err(|x| format!("{}: {x}", e.name))?,
            m => return Err(format!("{}: phương thức nén {m} chưa hỗ trợ", e.name)),
        };
        if out.len() as u64 != e.size {
            return Err(format!("{}: kích thước giải nén không khớp", e.name));
        }
        if crc32(&out) != e.crc {
            return Err(format!("{}: sai CRC (file hỏng)", e.name));
        }
        Ok(out)
    }
}

/// Ghi zip stored, tất định (thời gian DOS 2020-01-01 00:00, mode 0644).
pub fn write_stored(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    const DOS_TIME: u16 = 0;
    const DOS_DATE: u16 = ((2020 - 1980) << 9) | (1 << 5) | 1;
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let crc = crc32(data);
        let off = out.len() as u32;
        let n = name.as_bytes();
        // local header
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // flag: tên UTF-8
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&DOS_TIME.to_le_bytes());
        out.extend_from_slice(&DOS_DATE.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(n.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(n);
        out.extend_from_slice(data);
        // central directory
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&((3u16 << 8) | 20).to_le_bytes()); // made by Unix, v2.0
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0x0800u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&DOS_TIME.to_le_bytes());
        central.extend_from_slice(&DOS_DATE.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(n.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0u8; 8]); // extra, comment, disk, internal attr
        central.extend_from_slice(&((0o100644u32) << 16).to_le_bytes());
        central.extend_from_slice(&off.to_le_bytes());
        central.extend_from_slice(n);
    }
    let cd_off = out.len() as u32;
    let cd_len = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_len.to_le_bytes());
    out.extend_from_slice(&cd_off.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_then_read() {
        let files = vec![
            ("a.txt".to_string(), b"hello".to_vec()),
            ("dir/b.bin".to_string(), vec![0, 1, 2, 255]),
        ];
        let z = write_stored(&files);
        assert_eq!(z, write_stored(&files), "tất định");
        let a = ZipArchive::open(z).unwrap();
        assert_eq!(a.entries.len(), 2);
        assert_eq!(
            a.read(a.find("dir/b.bin").unwrap()).unwrap(),
            vec![0, 1, 2, 255]
        );
        assert!(!a.entries[0].is_symlink());
    }
    #[test]
    fn crc_mismatch_detected() {
        let mut z = write_stored(&[("a.txt".to_string(), b"hello".to_vec())]);
        let p = z.windows(5).position(|w| w == b"hello").unwrap();
        z[p] = b'j';
        let a = ZipArchive::open(z).unwrap();
        assert!(a.read(&a.entries[0]).unwrap_err().contains("CRC"));
    }
    #[test]
    fn not_a_zip() {
        assert!(ZipArchive::open(b"PK not really a zip file at all....".to_vec()).is_err());
        assert!(ZipArchive::open(vec![]).is_err());
    }
}
