//! Giải nén DEFLATE (RFC 1951) dạng raw, như dữ liệu method 8 trong ZIP.
//!
//! Giải mã Huffman chính tắc theo độ dài mã (cách của zlib/contrib/puff), có giới hạn
//! kích thước đầu ra để chống zip bomb.

#[derive(Debug, PartialEq)]
pub enum InflateError {
    Truncated,
    BadBlockType,
    StoredLengthMismatch,
    BadCode(&'static str),
    TooLarge(usize),
    DistanceTooFar,
}

impl std::fmt::Display for InflateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InflateError::Truncated => write!(f, "dữ liệu nén bị cụt"),
            InflateError::BadBlockType => write!(f, "kiểu block DEFLATE không hợp lệ"),
            InflateError::StoredLengthMismatch => write!(f, "block stored sai độ dài"),
            InflateError::BadCode(w) => write!(f, "mã Huffman không hợp lệ ({w})"),
            InflateError::TooLarge(n) => write!(f, "dữ liệu giải nén vượt giới hạn {n} byte"),
            InflateError::DistanceTooFar => {
                write!(f, "khoảng cách tham chiếu vượt dữ liệu đã giải")
            }
        }
    }
}

struct Bits<'a> {
    src: &'a [u8],
    pos: usize,
    buf: u32,
    cnt: u32,
}

impl<'a> Bits<'a> {
    fn need(&mut self, n: u32) -> Result<u32, InflateError> {
        while self.cnt < n {
            let b = *self.src.get(self.pos).ok_or(InflateError::Truncated)?;
            self.pos += 1;
            self.buf |= (b as u32) << self.cnt;
            self.cnt += 8;
        }
        let v = self.buf & ((1u32 << n) - 1);
        self.buf >>= n;
        self.cnt -= n;
        Ok(v)
    }
    fn bits(&mut self, n: u32) -> Result<u32, InflateError> {
        if n == 0 { Ok(0) } else { self.need(n) }
    }
}

const MAXBITS: usize = 15;

struct Huffman {
    count: [u16; MAXBITS + 1],
    symbol: Vec<u16>,
}

impl Huffman {
    /// Trả về (bảng, mã có đầy đủ hay không). Lỗi nếu bị over-subscribed.
    fn new(lengths: &[u8]) -> Result<(Huffman, bool), InflateError> {
        let mut count = [0u16; MAXBITS + 1];
        for &l in lengths {
            count[l as usize] += 1;
        }
        let mut left: i32 = 1;
        for &c in &count[1..] {
            left <<= 1;
            left -= c as i32;
            if left < 0 {
                return Err(InflateError::BadCode("over-subscribed"));
            }
        }
        let mut offs = [0u16; MAXBITS + 1];
        for len in 1..MAXBITS {
            offs[len + 1] = offs[len] + count[len];
        }
        let mut symbol = vec![0u16; lengths.len()];
        for (sym, &l) in lengths.iter().enumerate() {
            if l != 0 {
                symbol[offs[l as usize] as usize] = sym as u16;
                offs[l as usize] += 1;
            }
        }
        let complete = left == 0;
        Ok((Huffman { count, symbol }, complete))
    }

    fn decode(&self, b: &mut Bits) -> Result<u16, InflateError> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..=MAXBITS {
            code |= b.need(1)? as i32;
            let count = self.count[len] as i32;
            if code - count < first {
                return Ok(self.symbol[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err(InflateError::BadCode("không có mã khớp"))
    }
}

const LBASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEXT: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DBASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DEXT: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn codes(
    b: &mut Bits,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
    max: usize,
) -> Result<(), InflateError> {
    loop {
        let sym = lit.decode(b)? as usize;
        if sym < 256 {
            if out.len() >= max {
                return Err(InflateError::TooLarge(max));
            }
            out.push(sym as u8);
        } else if sym == 256 {
            return Ok(());
        } else {
            let s = sym - 257;
            if s >= 29 {
                return Err(InflateError::BadCode("mã độ dài"));
            }
            let len = LBASE[s] as usize + b.bits(LEXT[s] as u32)? as usize;
            let ds = dist.decode(b)? as usize;
            if ds >= 30 {
                return Err(InflateError::BadCode("mã khoảng cách"));
            }
            let d = DBASE[ds] as usize + b.bits(DEXT[ds] as u32)? as usize;
            if d > out.len() {
                return Err(InflateError::DistanceTooFar);
            }
            if out.len() + len > max {
                return Err(InflateError::TooLarge(max));
            }
            let start = out.len() - d;
            for k in 0..len {
                let c = out[start + k]; // có thể chồng lấp: đọc từng byte
                out.push(c);
            }
        }
    }
}

/// Giải nén raw DEFLATE, tối đa `max` byte đầu ra.
pub fn inflate(src: &[u8], max: usize) -> Result<Vec<u8>, InflateError> {
    let mut b = Bits {
        src,
        pos: 0,
        buf: 0,
        cnt: 0,
    };
    let mut out = Vec::new();
    loop {
        let last = b.need(1)?;
        match b.need(2)? {
            0 => {
                b.buf = 0;
                b.cnt = 0; // bỏ các bit lẻ tới biên byte
                if b.pos + 4 > src.len() {
                    return Err(InflateError::Truncated);
                }
                let len = u16::from_le_bytes([src[b.pos], src[b.pos + 1]]) as usize;
                let nlen = u16::from_le_bytes([src[b.pos + 2], src[b.pos + 3]]) as usize;
                if len != (!nlen & 0xFFFF) {
                    return Err(InflateError::StoredLengthMismatch);
                }
                b.pos += 4;
                if b.pos + len > src.len() {
                    return Err(InflateError::Truncated);
                }
                if out.len() + len > max {
                    return Err(InflateError::TooLarge(max));
                }
                out.extend_from_slice(&src[b.pos..b.pos + len]);
                b.pos += len;
            }
            1 => {
                let mut l = [0u8; 288];
                l[..144].fill(8);
                l[144..256].fill(9);
                l[256..280].fill(7);
                l[280..].fill(8);
                let (lit, _) = Huffman::new(&l)?;
                let (dist, _) = Huffman::new(&[5u8; 30])?;
                codes(&mut b, &mut out, &lit, &dist, max)?;
            }
            2 => {
                let nlen = b.need(5)? as usize + 257;
                let ndist = b.need(5)? as usize + 1;
                let ncode = b.need(4)? as usize + 4;
                if nlen > 286 || ndist > 30 {
                    return Err(InflateError::BadCode("số mã vượt giới hạn"));
                }
                const ORDER: [usize; 19] = [
                    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
                ];
                let mut cl = [0u8; 19];
                for &o in ORDER.iter().take(ncode) {
                    cl[o] = b.need(3)? as u8;
                }
                let (lencode, complete) = Huffman::new(&cl)?;
                if !complete {
                    return Err(InflateError::BadCode("bảng code-length không đầy đủ"));
                }
                let mut lengths = vec![0u8; nlen + ndist];
                let mut i = 0;
                while i < nlen + ndist {
                    let sym = lencode.decode(&mut b)?;
                    if sym < 16 {
                        lengths[i] = sym as u8;
                        i += 1;
                    } else {
                        let (val, rep) = match sym {
                            16 => {
                                if i == 0 {
                                    return Err(InflateError::BadCode("lặp khi chưa có độ dài"));
                                }
                                (lengths[i - 1], 3 + b.need(2)? as usize)
                            }
                            17 => (0, 3 + b.need(3)? as usize),
                            _ => (0, 11 + b.need(7)? as usize),
                        };
                        if i + rep > nlen + ndist {
                            return Err(InflateError::BadCode("lặp vượt số mã"));
                        }
                        lengths[i..i + rep].fill(val);
                        i += rep;
                    }
                }
                if lengths[256] == 0 {
                    return Err(InflateError::BadCode("thiếu mã kết thúc block"));
                }
                let (lit, lc) = Huffman::new(&lengths[..nlen])?;
                let lit_used = lengths[..nlen].iter().filter(|&&x| x != 0).count();
                if !lc && lit_used != 1 {
                    return Err(InflateError::BadCode("bảng literal/length không đầy đủ"));
                }
                let (dist, dc) = Huffman::new(&lengths[nlen..])?;
                let dist_used = lengths[nlen..].iter().filter(|&&x| x != 0).count();
                if !dc && dist_used > 1 {
                    return Err(InflateError::BadCode("bảng distance không đầy đủ"));
                }
                codes(&mut b, &mut out, &lit, &dist, max)?;
            }
            _ => return Err(InflateError::BadBlockType),
        }
        if last == 1 {
            return Ok(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stored_block() {
        // BFINAL=1, BTYPE=00, LEN=5, NLEN=!5, "hello"
        let d = [0x01, 0x05, 0x00, 0xFA, 0xFF, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(inflate(&d, 100).unwrap(), b"hello");
        assert_eq!(inflate(&d, 3), Err(InflateError::TooLarge(3)));
    }
    #[test]
    fn fixed_huffman_from_zlib() {
        // python: zlib.compressobj(9, zlib.DEFLATED, -15).compress(b"abcabcabcabc")+flush()
        let d = [0x4b, 0x4c, 0x4a, 0x4e, 0x84, 0x21, 0x00];
        assert_eq!(inflate(&d, 100).unwrap(), b"abcabcabcabc");
    }
    #[test]
    fn rejects_garbage() {
        assert!(inflate(&[0x07], 100).is_err()); // BTYPE=11
        assert!(inflate(&[], 100).is_err());
        assert!(inflate(&[0x01, 0x05, 0x00, 0x00, 0x00], 100).is_err());
    }
}
