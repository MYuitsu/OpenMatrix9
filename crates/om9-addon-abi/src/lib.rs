// SPDX-License-Identifier: LGPL-2.1-or-later
//! C ABI giữa OM9 và add-on native (DLL/.so), cộng SDK an toàn cho người viết add-on.
//!
//! Một add-on native xuất 4 hàm C:
//! - `om9_addon_abi_version() -> u32` — phải bằng [`ABI_VERSION`];
//! - `om9_addon_command_count() -> usize` và `om9_addon_command_name(i) -> Om9Str` — mã lệnh catalog
//!   add-on đăng ký (host đối chiếu với `[[commands]]` trong manifest);
//! - `om9_addon_invoke(*const Om9Invocation) -> i32` — chạy một lệnh; `0` = thành công.
//!
//! Dữ liệu vào là bản sao (polyline đã lấy mẫu, settings đã kiểm tra); dữ liệu ra chỉ đi qua
//! callback của host (`add_mesh`, `message`), host tự kiểm tra và tự ghi vào tài liệu trong một
//! transaction. Add-on không bao giờ giữ con trỏ của host sau khi `invoke` trả về.
//!
//! Người viết add-on dùng [`sdk::Addon`] + [`export_addon!`], không phải viết `unsafe`.

use std::ffi::c_void;

/// Tăng khi bố cục struct hoặc ngữ nghĩa thay đổi không tương thích.
pub const ABI_VERSION: u32 = 1;

pub const SYM_ABI_VERSION: &[u8] = b"om9_addon_abi_version\0";
pub const SYM_COMMAND_COUNT: &[u8] = b"om9_addon_command_count\0";
pub const SYM_COMMAND_NAME: &[u8] = b"om9_addon_command_name\0";
pub const SYM_INVOKE: &[u8] = b"om9_addon_invoke\0";
/// Optional: interactive commands (see [`session`]), `om9_addon_session(json, len, write, ctx)`.
pub const SYM_SESSION: &[u8] = b"om9_addon_session\0";

/// Callback through which an add-on hands its JSON reply to the host (copied before return).
pub type Om9Write = unsafe extern "C" fn(ctx: *mut c_void, data: *const u8, len: usize);
/// Signature of `om9_addon_session`.
pub type Om9SessionFn =
    unsafe extern "C" fn(input: *const u8, len: usize, write: Om9Write, ctx: *mut c_void) -> i32;

#[cfg(feature = "session")]
pub mod session;

/// Kết quả `om9_addon_invoke`.
pub const INVOKE_OK: i32 = 0;
pub const INVOKE_ERROR: i32 = 1;
pub const INVOKE_PANIC: i32 = 2;
pub const INVOKE_BAD_INPUT: i32 = 3;

/// Giới hạn đọc dữ liệu vào phía add-on (chống con trỏ/độ dài hỏng).
pub const MAX_INPUT_POINTS: usize = 10_000_000;
pub const MAX_INPUT_ITEMS: usize = 100_000;

/// Chuỗi UTF-8 mượn (không có NUL cuối).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Om9Str {
    pub ptr: *const u8,
    pub len: usize,
}

impl Om9Str {
    pub const EMPTY: Om9Str = Om9Str {
        ptr: std::ptr::null(),
        len: 0,
    };
    pub fn new(s: &str) -> Om9Str {
        Om9Str {
            ptr: s.as_ptr(),
            len: s.len(),
        }
    }
    /// # Safety
    /// `ptr` phải trỏ tới `len` byte hợp lệ còn sống trong `'a`.
    pub unsafe fn to_str<'a>(self) -> Option<&'a str> {
        if self.len == 0 {
            return Some("");
        }
        if self.ptr.is_null() || self.len > isize::MAX as usize {
            return None;
        }
        // SAFETY: theo hợp đồng của hàm.
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
        std::str::from_utf8(bytes).ok()
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Om9Curve {
    pub points: *const [f64; 3],
    pub count: usize,
    /// 1 = đường cong kín (điểm cuối không lặp lại điểm đầu).
    pub closed: u8,
}

pub const SETTING_NUMBER: u32 = 0;
pub const SETTING_TEXT: u32 = 1;
pub const SETTING_BOOL: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Om9Setting {
    pub key: Om9Str,
    pub kind: u32,
    pub number: f64,
    pub text: Om9Str,
}

pub type AddMeshFn = unsafe extern "C" fn(
    host: *mut c_void,
    name: Om9Str,
    vertices: *const [f64; 3],
    vertex_count: usize,
    triangles: *const [u32; 3],
    triangle_count: usize,
    meta_json: Om9Str,
) -> i32;
pub type MessageFn = unsafe extern "C" fn(host: *mut c_void, text: Om9Str);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Om9HostApi {
    pub host: *mut c_void,
    pub add_mesh: AddMeshFn,
    pub message: MessageFn,
}

#[repr(C)]
pub struct Om9Invocation {
    pub abi_version: u32,
    pub command: Om9Str,
    pub curves: *const Om9Curve,
    pub curve_count: usize,
    pub settings: *const Om9Setting,
    pub setting_count: usize,
    /// Pháp tuyến đơn vị của CPlane view đang hoạt động (dùng định hướng đá).
    pub plane_normal: [f64; 3],
    pub host: Om9HostApi,
}

pub type AbiVersionFn = unsafe extern "C" fn() -> u32;
pub type CommandCountFn = unsafe extern "C" fn() -> usize;
pub type CommandNameFn = unsafe extern "C" fn(index: usize) -> Om9Str;
pub type InvokeFn = unsafe extern "C" fn(invocation: *const Om9Invocation) -> i32;

pub mod sdk {
    //! Phần an toàn cho người viết add-on.
    use super::*;
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Number(f64),
        Text(String),
        Bool(bool),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct CurveData {
        pub points: Vec<[f64; 3]>,
        pub closed: bool,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Request {
        pub command: String,
        pub curves: Vec<CurveData>,
        pub settings: BTreeMap<String, Value>,
        pub plane_normal: [f64; 3],
    }

    impl Request {
        pub fn number(&self, key: &str, default: f64) -> f64 {
            match self.settings.get(key) {
                Some(Value::Number(n)) => *n,
                _ => default,
            }
        }
        pub fn text<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
            match self.settings.get(key) {
                Some(Value::Text(t)) => t,
                _ => default,
            }
        }
        pub fn flag(&self, key: &str, default: bool) -> bool {
            match self.settings.get(key) {
                Some(Value::Bool(b)) => *b,
                _ => default,
            }
        }
    }

    /// Kênh ghi kết quả về host.
    pub trait Output {
        fn add_mesh(
            &mut self,
            name: &str,
            vertices: &[[f64; 3]],
            triangles: &[[u32; 3]],
            meta_json: &str,
        ) -> Result<(), String>;
        fn message(&mut self, text: &str);
    }

    pub trait Addon {
        /// Mã lệnh catalog (phải khớp `[[commands]] id` trong manifest).
        const COMMANDS: &'static [&'static str];
        fn invoke(request: &Request, out: &mut dyn Output) -> Result<(), String>;
    }

    struct HostOutput(Om9HostApi);

    impl Output for HostOutput {
        fn add_mesh(
            &mut self,
            name: &str,
            vertices: &[[f64; 3]],
            triangles: &[[u32; 3]],
            meta_json: &str,
        ) -> Result<(), String> {
            // SAFETY: con trỏ host do host cấp cho đúng lần gọi này; slice mượn còn sống suốt lời gọi.
            let code = unsafe {
                (self.0.add_mesh)(
                    self.0.host,
                    Om9Str::new(name),
                    vertices.as_ptr(),
                    vertices.len(),
                    triangles.as_ptr(),
                    triangles.len(),
                    Om9Str::new(meta_json),
                )
            };
            if code == 0 {
                Ok(())
            } else {
                Err(format!("host từ chối mesh {name:?} (mã {code})"))
            }
        }
        fn message(&mut self, text: &str) {
            // SAFETY: như trên.
            unsafe { (self.0.message)(self.0.host, Om9Str::new(text)) }
        }
    }

    /// Đọc `Om9Invocation` thành [`Request`] sở hữu dữ liệu.
    ///
    /// # Safety
    /// `inv` là con trỏ hợp lệ do host cấp, còn sống suốt lời gọi.
    pub unsafe fn read_request(inv: &Om9Invocation) -> Result<Request, &'static str> {
        if inv.abi_version != ABI_VERSION {
            return Err("abi");
        }
        // SAFETY: hợp đồng của hàm.
        let command = unsafe { inv.command.to_str() }.ok_or("command")?.to_owned();
        if inv.curve_count > MAX_INPUT_ITEMS || inv.setting_count > MAX_INPUT_ITEMS {
            return Err("count");
        }
        let mut curves = Vec::with_capacity(inv.curve_count);
        if inv.curve_count > 0 {
            if inv.curves.is_null() {
                return Err("curves");
            }
            // SAFETY: host cấp mảng `curve_count` phần tử.
            let raw = unsafe { std::slice::from_raw_parts(inv.curves, inv.curve_count) };
            let mut total = 0usize;
            for c in raw {
                total = total.checked_add(c.count).ok_or("points")?;
                if total > MAX_INPUT_POINTS || (c.count > 0 && c.points.is_null()) {
                    return Err("points");
                }
                let points = if c.count == 0 {
                    Vec::new()
                } else {
                    // SAFETY: host cấp `count` điểm.
                    unsafe { std::slice::from_raw_parts(c.points, c.count) }.to_vec()
                };
                if points.iter().flatten().any(|v| !v.is_finite()) {
                    return Err("non-finite point");
                }
                curves.push(CurveData {
                    points,
                    closed: c.closed != 0,
                });
            }
        }
        let mut settings = BTreeMap::new();
        if inv.setting_count > 0 {
            if inv.settings.is_null() {
                return Err("settings");
            }
            // SAFETY: host cấp mảng `setting_count` phần tử.
            let raw = unsafe { std::slice::from_raw_parts(inv.settings, inv.setting_count) };
            for s in raw {
                // SAFETY: chuỗi mượn của host.
                let key = unsafe { s.key.to_str() }.ok_or("setting key")?.to_owned();
                let value = match s.kind {
                    SETTING_NUMBER => Value::Number(s.number),
                    SETTING_BOOL => Value::Bool(s.number != 0.0),
                    SETTING_TEXT => {
                        Value::Text(unsafe { s.text.to_str() }.ok_or("setting text")?.to_owned())
                    }
                    _ => return Err("setting kind"),
                };
                settings.insert(key, value);
            }
        }
        Ok(Request {
            command,
            curves,
            settings,
            plane_normal: inv.plane_normal,
        })
    }

    /// Thân của `om9_addon_invoke` do [`export_addon!`] sinh ra.
    ///
    /// # Safety
    /// `inv` là con trỏ do host cấp.
    pub unsafe fn dispatch<A: Addon>(inv: *const Om9Invocation) -> i32 {
        if inv.is_null() {
            return INVOKE_BAD_INPUT;
        }
        // SAFETY: host cấp con trỏ hợp lệ cho lời gọi này.
        let inv = unsafe { &*inv };
        let host = inv.host;
        let result = std::panic::catch_unwind(|| {
            // SAFETY: như trên.
            let request = match unsafe { read_request(inv) } {
                Ok(r) => r,
                Err(what) => {
                    return (
                        INVOKE_BAD_INPUT,
                        Some(format!("dữ liệu vào không hợp lệ: {what}")),
                    );
                }
            };
            let mut out = HostOutput(host);
            match A::invoke(&request, &mut out) {
                Ok(()) => (INVOKE_OK, None),
                Err(e) => (INVOKE_ERROR, Some(e)),
            }
        });
        let (code, msg) = result.unwrap_or((
            INVOKE_PANIC,
            Some("add-on bị panic; lệnh đã huỷ".to_owned()),
        ));
        if let Some(m) = msg {
            // SAFETY: host còn sống suốt lời gọi.
            unsafe { (host.message)(host.host, Om9Str::new(&m)) };
        }
        code
    }

    /// Chuỗi tĩnh của mã lệnh thứ `i`.
    pub fn command_name<A: Addon>(i: usize) -> Om9Str {
        A::COMMANDS
            .get(i)
            .map(|s| Om9Str::new(s))
            .unwrap_or(Om9Str::EMPTY)
    }
}

/// Xuất 4 hàm C của add-on cho kiểu cài [`sdk::Addon`].
#[macro_export]
macro_rules! export_addon {
    ($ty:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_abi_version() -> u32 {
            $crate::ABI_VERSION
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_command_count() -> usize {
            <$ty as $crate::sdk::Addon>::COMMANDS.len()
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn om9_addon_command_name(index: usize) -> $crate::Om9Str {
            $crate::sdk::command_name::<$ty>(index)
        }
        /// # Safety
        /// Chỉ host OM9 gọi, với con trỏ hợp lệ cho đúng lời gọi.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn om9_addon_invoke(invocation: *const $crate::Om9Invocation) -> i32 {
            // SAFETY: chuyển tiếp hợp đồng của host.
            unsafe { $crate::sdk::dispatch::<$ty>(invocation) }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdk::*;

    struct Echo;
    impl Addon for Echo {
        const COMMANDS: &'static [&'static str] = &["EchoCmd"];
        fn invoke(r: &Request, out: &mut dyn Output) -> Result<(), String> {
            if r.command == "Panic" {
                panic!("boom");
            }
            if r.command == "Fail" {
                return Err("cố ý lỗi".into());
            }
            out.message(&format!(
                "{} curves={} gap={}",
                r.command,
                r.curves.len(),
                r.number("gap", -1.0)
            ));
            out.add_mesh(
                "tri",
                &[[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                &[[0, 1, 2]],
                "{}",
            )
        }
    }

    #[derive(Default)]
    struct Sink {
        msgs: Vec<String>,
        meshes: usize,
    }
    unsafe extern "C" fn add_mesh(
        h: *mut c_void,
        _n: Om9Str,
        _v: *const [f64; 3],
        nv: usize,
        _t: *const [u32; 3],
        nt: usize,
        _m: Om9Str,
    ) -> i32 {
        let s = unsafe { &mut *(h as *mut Sink) };
        s.meshes += (nv == 3 && nt == 1) as usize;
        0
    }
    unsafe extern "C" fn message(h: *mut c_void, t: Om9Str) {
        let s = unsafe { &mut *(h as *mut Sink) };
        s.msgs.push(unsafe { t.to_str() }.unwrap().to_owned());
    }

    fn call(cmd: &str, sink: &mut Sink, abi: u32) -> i32 {
        let pts = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let curves = [Om9Curve {
            points: pts.as_ptr(),
            count: 2,
            closed: 0,
        }];
        let key = "gap";
        let settings = [Om9Setting {
            key: Om9Str::new(key),
            kind: SETTING_NUMBER,
            number: 0.2,
            text: Om9Str::EMPTY,
        }];
        let inv = Om9Invocation {
            abi_version: abi,
            command: Om9Str::new(cmd),
            curves: curves.as_ptr(),
            curve_count: 1,
            settings: settings.as_ptr(),
            setting_count: 1,
            plane_normal: [0.0, 0.0, 1.0],
            host: Om9HostApi {
                host: sink as *mut Sink as *mut c_void,
                add_mesh,
                message,
            },
        };
        unsafe { dispatch::<Echo>(&inv) }
    }

    #[test]
    fn roundtrip_errors_and_panics() {
        let mut s = Sink::default();
        assert_eq!(call("EchoCmd", &mut s, ABI_VERSION), INVOKE_OK);
        assert_eq!(s.msgs, vec!["EchoCmd curves=1 gap=0.2"]);
        assert_eq!(s.meshes, 1);
        assert_eq!(call("Fail", &mut s, ABI_VERSION), INVOKE_ERROR);
        assert_eq!(s.msgs.last().unwrap(), "cố ý lỗi");
        let prev = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        assert_eq!(call("Panic", &mut s, ABI_VERSION), INVOKE_PANIC);
        std::panic::set_hook(prev);
        assert_eq!(call("EchoCmd", &mut s, ABI_VERSION + 1), INVOKE_BAD_INPUT);
        assert_eq!(
            unsafe { dispatch::<Echo>(std::ptr::null()) },
            INVOKE_BAD_INPUT
        );
        assert_eq!(unsafe { command_name::<Echo>(0).to_str() }, Some("EchoCmd"));
        assert_eq!(command_name::<Echo>(5).len, 0);
    }
}
