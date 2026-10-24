//! Versioned synchronous C ABI. JSON is UTF-8, at most 8 MiB per message.
//! Callees call the reply callback exactly once, before returning. Successful
//! replies contain the requested JSON value; STATUS_ERROR replies contain
//! `SdkError`. The receiver copies bytes during the callback. Neither side may
//! retain contexts/pointers, transfer allocator ownership, or unwind over C ABI.

use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
};

use serde::Serialize;

use crate::{Host, Plugin, PluginRequest, Result, SdkError};

pub const ABI_VERSION: u32 = 1;
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const STATUS_OK: i32 = 0;
pub const STATUS_ERROR: i32 = 1;

pub type ReplyFn = unsafe extern "C" fn(context: *mut c_void, data: *const u8, len: usize) -> i32;
pub type CallFn = unsafe extern "C" fn(
    context: *mut c_void,
    data: *const u8,
    len: usize,
    reply_context: *mut c_void,
    reply: ReplyFn,
) -> i32;
pub type EntryFn = unsafe extern "C" fn(
    host: *const HostApi,
    data: *const u8,
    len: usize,
    reply_context: *mut c_void,
    reply: ReplyFn,
) -> i32;

#[repr(C)]
pub struct HostApi {
    pub abi_version: u32,
    pub struct_size: u32,
    pub context: *mut c_void,
    pub call: Option<CallFn>,
}

/// Copy-only reply storage. Keep it alive and stationary during the foreign call.
#[derive(Default)]
pub struct ReplyCapture {
    data: Option<Vec<u8>>,
    invalid: bool,
}

impl ReplyCapture {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn context(&mut self) -> *mut c_void {
        (self as *mut Self).cast()
    }

    /// Check transport status and decode a remote SDK error when present.
    pub fn finish(self, status: i32) -> Result<Vec<u8>> {
        if self.invalid {
            return Err(SdkError::new(
                "invalid_reply",
                "Reply callback rejected invalid, oversized or repeated data",
            ));
        }
        let bytes = self
            .data
            .ok_or_else(|| SdkError::new("missing_reply", "Callee did not send a reply"))?;
        match status {
            STATUS_OK => Ok(bytes),
            STATUS_ERROR => Err(
                serde_json::from_slice::<SdkError>(&bytes).unwrap_or_else(|_| {
                    SdkError::new("remote_error", "Callee returned an invalid SDK error")
                }),
            ),
            _ => Err(SdkError::new(
                "invalid_status",
                "Callee returned an unknown status",
            )),
        }
    }
}

/// Bounded synchronous reply callback, also usable by a native host.
///
/// # Safety
/// `context` must point to a live, uniquely borrowed `ReplyCapture` on this
/// thread. Nonempty `data` must be readable for `len` bytes, and must not overlap
/// that capture. The callback and context cannot be retained or used concurrently.
pub unsafe extern "C" fn capture_reply(context: *mut c_void, data: *const u8, len: usize) -> i32 {
    if context.is_null() || !(context as usize).is_multiple_of(std::mem::align_of::<ReplyCapture>())
    {
        return STATUS_ERROR;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let capture = unsafe { &mut *context.cast::<ReplyCapture>() };
        if capture.invalid
            || capture.data.is_some()
            || len > MAX_MESSAGE_BYTES
            || (data.is_null() && len != 0)
        {
            capture.invalid = true;
            return STATUS_ERROR;
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(data, len) }
        };
        capture.data = Some(bytes.to_vec());
        STATUS_OK
    }));
    match result {
        Ok(status) => status,
        Err(_) => {
            unsafe {
                (*context.cast::<ReplyCapture>()).invalid = true;
            }
            STATUS_ERROR
        }
    }
}

pub(crate) fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    struct BoundedWriter(Vec<u8>);
    impl std::io::Write for BoundedWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_MESSAGE_BYTES - self.0.len() {
                return Err(std::io::Error::other("Message exceeds 8 MiB"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = BoundedWriter(Vec::new());
    serde_json::to_writer(&mut writer, value).map_err(|error| {
        SdkError::new(
            if error.is_io() {
                "message_size"
            } else {
                "json_encode"
            },
            error.to_string(),
        )
    })?;
    Ok(writer.0)
}

/// Execute one request and contain Rust unwinding panics before returning to C.
/// Abort-mode panics, access violations and native-code faults are not contained.
///
/// # Safety
/// For Execute, the host table must satisfy `Host::from_raw` for this invocation;
/// Describe does not access the host table, which may be null. Nonempty
/// request bytes must be readable for `len`, and `reply` must be a live,
/// non-unwinding callback accepting `reply_context`. Reply data is borrowed only
/// until the callback returns. All calls must stay on the invoking thread and
/// the host and callbacks must not retain any borrowed pointer.
pub unsafe fn dispatch<P: Plugin>(
    host: *const HostApi,
    data: *const u8,
    len: usize,
    reply_context: *mut c_void,
    reply: ReplyFn,
) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Vec<u8>> {
        if len > MAX_MESSAGE_BYTES || (data.is_null() && len != 0) {
            return Err(SdkError::new(
                "invalid_request",
                "Missing request bytes or request exceeds 8 MiB",
            ));
        }
        let bytes = if len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(data, len) }
        };
        let request: PluginRequest = serde_json::from_slice(bytes)
            .map_err(|error| SdkError::new("invalid_request", error.to_string()))?;
        match request {
            PluginRequest::Describe => encode(&P::descriptor()),
            PluginRequest::Execute { command, arguments } => {
                let mut host = unsafe { Host::from_raw(host)? };
                let result = P::default().execute(&mut host, &command, arguments)?;
                encode(&result)
            }
        }
    }));
    let result = result.unwrap_or_else(|_| {
        Err(SdkError::new(
            "plugin_panic",
            "Plugin panicked during invocation",
        ))
    });
    let (status, bytes) = match result {
        Ok(bytes) => (STATUS_OK, bytes),
        Err(error) => {
            // Plugin-supplied errors can also exceed the transport limit.
            let bytes = encode(&error).unwrap_or_else(|_| {
                br#"{"code":"message_size","message":"Error exceeds transport limit"}"#.to_vec()
            });
            (STATUS_ERROR, bytes)
        }
    };
    if unsafe { reply(reply_context, bytes.as_ptr(), bytes.len()) } != STATUS_OK {
        STATUS_ERROR
    } else {
        status
    }
}
