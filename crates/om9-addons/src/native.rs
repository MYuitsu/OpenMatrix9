// SPDX-License-Identifier: LGPL-2.1-or-later
//! Runtime add-on native: nạp DLL/.so theo `om9-addon-abi`.
//!
//! Ranh giới an toàn: mọi dữ liệu add-on trả về đi qua callback, được sao chép và kiểm tra
//! (độ dài, chỉ số, số hữu hạn) trước khi tới host. Thư viện chỉ được giải phóng khi add-on tắt,
//! sau khi mọi lời gọi đã trả về. Add-on native chạy cùng tiến trình nên không phải sandbox:
//! chỉ dùng cho add-on tin cậy (mặc định của OM9 cho kho system/user).

use crate::runtime::{
    CommandInput, CommandOutput, LoadRequest, MAX_MESSAGES, MAX_OUTPUT_MESHES, MAX_OUTPUT_VERTICES,
    MeshOutput, Runtime,
};
use crate::settings::SettingValue;
use om9_addon_abi::*;
use std::collections::BTreeMap;
use std::ffi::c_void;

struct Loaded {
    // Giữ thư viện sống; con trỏ hàm bên dưới chỉ hợp lệ khi `_lib` còn.
    _lib: libloading::Library,
    invoke: InvokeFn,
    session: Option<Om9SessionFn>,
}

#[derive(Default)]
pub struct NativeRuntime {
    loaded: BTreeMap<String, Loaded>,
}

impl NativeRuntime {
    pub fn new() -> NativeRuntime {
        NativeRuntime::default()
    }
}

#[derive(Default)]
struct Collector {
    meshes: Vec<MeshOutput>,
    messages: Vec<String>,
    vertices: usize,
    error: Option<String>,
}

unsafe extern "C" fn cb_message(host: *mut c_void, text: Om9Str) {
    if host.is_null() {
        return;
    }
    // SAFETY: `host` là `&mut Collector` do `invoke` cấp, chỉ dùng trong lời gọi.
    let c = unsafe { &mut *(host as *mut Collector) };
    if c.messages.len() >= MAX_MESSAGES {
        return;
    }
    // SAFETY: chuỗi do add-on cấp, còn sống suốt callback.
    let t = unsafe { text.to_str() }.unwrap_or("(thông điệp không phải UTF-8)");
    c.messages.push(t.chars().take(2000).collect());
}

unsafe extern "C" fn cb_add_mesh(
    host: *mut c_void,
    name: Om9Str,
    vertices: *const [f64; 3],
    vertex_count: usize,
    triangles: *const [u32; 3],
    triangle_count: usize,
    meta: Om9Str,
) -> i32 {
    if host.is_null() {
        return -1;
    }
    // SAFETY: như `cb_message`.
    let c = unsafe { &mut *(host as *mut Collector) };
    let fail = |c: &mut Collector, e: String| {
        c.error.get_or_insert(e);
        -1
    };
    if c.meshes.len() >= MAX_OUTPUT_MESHES {
        return fail(c, format!("quá {MAX_OUTPUT_MESHES} mesh"));
    }
    if vertex_count < 3 || triangle_count == 0 || vertices.is_null() || triangles.is_null() {
        return fail(c, "mesh rỗng".into());
    }
    if vertex_count > MAX_OUTPUT_VERTICES - c.vertices || triangle_count > 4 * MAX_OUTPUT_VERTICES {
        return fail(c, format!("quá {MAX_OUTPUT_VERTICES} đỉnh"));
    }
    // SAFETY: add-on cấp mảng đúng độ dài, còn sống suốt callback; ta sao chép ngay.
    let v = unsafe { std::slice::from_raw_parts(vertices, vertex_count) }.to_vec();
    let t = unsafe { std::slice::from_raw_parts(triangles, triangle_count) }.to_vec();
    let name = unsafe { name.to_str() }
        .unwrap_or("Gem")
        .chars()
        .take(120)
        .collect::<String>();
    let meta_json = unsafe { meta.to_str() }.unwrap_or("").to_owned();
    if v.iter().flatten().any(|x| !x.is_finite()) {
        return fail(c, format!("mesh {name:?} có toạ độ không hữu hạn"));
    }
    if t.iter().flatten().any(|&i| i as usize >= vertex_count) {
        return fail(c, format!("mesh {name:?} có chỉ số đỉnh vượt phạm vi"));
    }
    c.vertices += vertex_count;
    c.meshes.push(MeshOutput {
        name,
        vertices: v,
        triangles: t,
        meta_json,
    });
    0
}

#[derive(Default)]
struct SessionSink {
    bytes: Vec<u8>,
    overflow: bool,
}

unsafe extern "C" fn cb_session_write(ctx: *mut c_void, data: *const u8, len: usize) {
    if ctx.is_null() || (data.is_null() && len > 0) {
        return;
    }
    // SAFETY: `ctx` is the `SessionSink` of the running call.
    let sink = unsafe { &mut *(ctx as *mut SessionSink) };
    if sink.bytes.len().saturating_add(len) > crate::runtime::MAX_SESSION_JSON {
        sink.overflow = true;
        return;
    }
    if len > 0 {
        // SAFETY: the add-on passes `len` readable bytes for this callback; copied at once.
        sink.bytes
            .extend_from_slice(unsafe { std::slice::from_raw_parts(data, len) });
    }
}

/// Stack of the thread that runs an add-on step. Geometry add-ons recurse deeply (BSP booleans
/// over thousands of faces); the UI thread that drives the session has only 1 MiB on Windows,
/// and overflowing it ends OM9 at once, without a message.
const ADDON_STACK: usize = 256 << 20;

/// Run one add-on call on a thread with [`ADDON_STACK`] and wait for it; on the caller's thread
/// when no thread can be started.
fn on_large_stack<T: Send, R: Send>(arg: &mut T, call: impl Fn(&mut T) -> R + Sync) -> R {
    let call = &call;
    let worker = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("om9-addon".into())
            .stack_size(ADDON_STACK)
            .spawn_scoped(scope, || call(arg))
            .map(|h| h.join())
    });
    match worker {
        Ok(Ok(r)) => r,
        // The add-on catches its own panics; one escaping the FFI boundary is re-raised here.
        Ok(Err(panic)) => std::panic::resume_unwind(panic),
        Err(_) => call(arg),
    }
}

impl Runtime for NativeRuntime {
    fn load(&mut self, req: &LoadRequest) -> Result<Vec<String>, String> {
        let entry = req
            .manifest
            .entry
            .as_deref()
            .ok_or("manifest thiếu entry")?;
        let path = req.dir.join(entry);
        if !path.is_file() {
            return Err(format!("không tìm thấy thư viện {entry}"));
        }
        // SAFETY: nạp thư viện native chạy hàm khởi tạo của nó; đây là ranh giới tin cậy của add-on
        // native (đã được người dùng cài và bật).
        let lib = unsafe { libloading::Library::new(&path) }
            .map_err(|e| format!("không nạp được {entry}: {e}"))?;
        // SAFETY: kiểu ký hiệu theo om9-addon-abi; sai kiểu sẽ bị chặn bởi kiểm tra ABI ngay sau.
        let (abi, count, name, invoke) = unsafe {
            let abi = *lib.get::<AbiVersionFn>(SYM_ABI_VERSION).map_err(|_| {
                format!("{entry} không phải add-on OM9 (thiếu om9_addon_abi_version)")
            })?;
            let count = *lib
                .get::<CommandCountFn>(SYM_COMMAND_COUNT)
                .map_err(|_| "thiếu om9_addon_command_count".to_string())?;
            let name = *lib
                .get::<CommandNameFn>(SYM_COMMAND_NAME)
                .map_err(|_| "thiếu om9_addon_command_name".to_string())?;
            let invoke = *lib
                .get::<InvokeFn>(SYM_INVOKE)
                .map_err(|_| "thiếu om9_addon_invoke".to_string())?;
            (abi, count, name, invoke)
        };
        // SAFETY: các hàm không nhận con trỏ từ host.
        let version = unsafe { abi() };
        if version != ABI_VERSION {
            return Err(format!(
                "{entry} dùng ABI {version}, OM9 này hỗ trợ ABI {ABI_VERSION}"
            ));
        }
        let n = unsafe { count() };
        if n > 4096 {
            return Err("add-on khai báo quá nhiều lệnh".into());
        }
        let mut commands = Vec::with_capacity(n);
        for i in 0..n {
            // SAFETY: add-on trả chuỗi tĩnh của chính nó; ta sao chép khi thư viện còn nạp.
            let s = unsafe { name(i).to_str() }.ok_or("tên lệnh không phải UTF-8")?;
            commands.push(s.to_owned());
        }
        // Optional: interactive commands.
        // SAFETY: the symbol type is fixed by om9-addon-abi.
        let session = unsafe { lib.get::<Om9SessionFn>(SYM_SESSION).ok().map(|f| *f) };
        self.loaded.insert(
            req.id.to_owned(),
            Loaded {
                _lib: lib,
                invoke,
                session,
            },
        );
        Ok(commands)
    }

    fn unload(&mut self, id: &str) {
        self.loaded.remove(id);
    }

    fn session(&mut self, id: &str, event: &[u8]) -> Result<Vec<u8>, String> {
        let loaded = self
            .loaded
            .get(id)
            .ok_or_else(|| format!("add-on {id} chưa nạp"))?;
        let f = loaded
            .session
            .ok_or_else(|| format!("add-on {id} không có lệnh tương tác"))?;
        if event.len() > crate::runtime::MAX_SESSION_JSON {
            return Err("sự kiện quá lớn".into());
        }
        let mut sink = SessionSink::default();
        let call = |sink: &mut SessionSink| {
            // SAFETY: `event` and `sink` live until the call returns; the add-on copies nothing
            // back except through `cb_session_write`, which copies into `sink`.
            unsafe {
                f(
                    event.as_ptr(),
                    event.len(),
                    cb_session_write,
                    sink as *mut SessionSink as *mut c_void,
                )
            }
        };
        let code = on_large_stack(&mut sink, call);
        if sink.overflow {
            return Err(format!("{id}: trả lời quá lớn"));
        }
        match code {
            INVOKE_OK => Ok(sink.bytes),
            INVOKE_PANIC => Err(format!("{id}: add-on bị panic; lệnh đã huỷ")),
            _ => Err(String::from_utf8_lossy(&sink.bytes)
                .trim_matches('"')
                .chars()
                .take(2000)
                .collect()),
        }
    }

    fn invoke(&mut self, id: &str, input: &CommandInput) -> Result<CommandOutput, String> {
        let loaded = self
            .loaded
            .get(id)
            .ok_or_else(|| format!("add-on {id} chưa nạp"))?;
        let curves: Vec<Om9Curve> = input
            .curves
            .iter()
            .map(|c| Om9Curve {
                points: c.points.as_ptr(),
                count: c.points.len(),
                closed: c.closed as u8,
            })
            .collect();
        let texts: Vec<(String, Option<String>)> = input
            .settings
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    if let SettingValue::Str(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    },
                )
            })
            .collect();
        let settings: Vec<Om9Setting> = input
            .settings
            .values()
            .zip(&texts)
            .map(|(v, (k, t))| {
                let (kind, number) = match v {
                    SettingValue::Bool(b) => (SETTING_BOOL, *b as u8 as f64),
                    SettingValue::Int(i) => (SETTING_NUMBER, *i as f64),
                    SettingValue::Float(f) => (SETTING_NUMBER, *f),
                    SettingValue::Str(_) => (SETTING_TEXT, 0.0),
                };
                Om9Setting {
                    key: Om9Str::new(k),
                    kind,
                    number,
                    text: t.as_deref().map(Om9Str::new).unwrap_or(Om9Str::EMPTY),
                }
            })
            .collect();
        let mut collector = Collector::default();
        let inv = Om9Invocation {
            abi_version: ABI_VERSION,
            command: Om9Str::new(&input.command),
            curves: curves.as_ptr(),
            curve_count: curves.len(),
            settings: settings.as_ptr(),
            setting_count: settings.len(),
            plane_normal: input.plane_normal,
            host: Om9HostApi {
                host: &mut collector as *mut Collector as *mut c_void,
                add_mesh: cb_add_mesh,
                message: cb_message,
            },
        };
        // SAFETY: mọi con trỏ trong `inv` trỏ vào dữ liệu sống tới hết lời gọi; add-on không được giữ lại.
        let code = unsafe { (loaded.invoke)(&inv) };
        if let Some(e) = collector.error.take() {
            return Err(format!("kết quả add-on bị từ chối: {e}"));
        }
        let out = CommandOutput {
            meshes: collector.meshes,
            messages: collector.messages,
        };
        match code {
            INVOKE_OK => {
                out.validate()?;
                Ok(out)
            }
            _ => {
                let why = out
                    .messages
                    .last()
                    .cloned()
                    .unwrap_or_else(|| format!("mã lỗi {code}"));
                Err(why)
            }
        }
    }
}
