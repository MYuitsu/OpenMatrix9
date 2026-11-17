// SPDX-License-Identifier: LGPL-2.1-or-later
//! Owned canonical source between operations. Never a document/display candidate.
use om9_core::{Error, Result, budget::Limits};
use std::ffi::{CStr, c_char, c_int, c_void};

pub(crate) type Sink = unsafe extern "C" fn(*mut c_void, *const u8, usize) -> c_int;

pub(crate) fn source_via(
    limits: &Limits,
    admit: &mut dyn FnMut(u64, u64) -> Result<()>,
    invoke: impl FnOnce(Sink, *mut c_void, *mut c_char, c_int) -> c_int,
) -> Result<String> {
    struct State<'a> {
        limits: &'a Limits,
        admit: &'a mut dyn FnMut(u64, u64) -> Result<()>,
        result: Option<Result<String>>,
    }
    unsafe extern "C" fn sink(context: *mut c_void, data: *const u8, len: usize) -> c_int {
        // SAFETY: the synchronous adapter retains the context and source until return.
        let state = unsafe { &mut *context.cast::<State<'_>>() };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if state.result.is_some()
                || data.is_null()
                || len == 0
                || len as u64 > state.limits.max_bytes
            {
                return Err(Error::Invalid("intermediate source payload"));
            }
            // Native archive/scratch remain outside managed accounting, as in
            // the snapshot adapter. Admit the Rust allocation before copying.
            (state.admit)(0, len as u64)?;
            let bytes = unsafe { std::slice::from_raw_parts(data, len) };
            let text = std::str::from_utf8(bytes)
                .map_err(|_| Error::Invalid("intermediate source encoding"))?;
            let mut owned = String::new();
            owned
                .try_reserve_exact(len)
                .map_err(|_| Error::Budget("intermediate source"))?;
            (state.admit)(owned.capacity() as u64, 0)?;
            owned.push_str(text);
            Ok(owned)
        }))
        .unwrap_or_else(|_| Err(Error::Invalid("intermediate source callback panic")));
        let status = i32::from(result.is_err());
        state.result = Some(result);
        status
    }
    let mut state = State {
        limits,
        admit,
        result: None,
    };
    let mut message = [0 as c_char; 512];
    let status = invoke(
        sink,
        (&mut state as *mut State<'_>).cast(),
        message.as_mut_ptr(),
        message.len() as c_int,
    );
    if let Some(Err(error)) = &state.result {
        return Err(error.clone());
    }
    if status != 0 {
        message[511] = 0;
        let message = unsafe { CStr::from_ptr(message.as_ptr()) }.to_string_lossy();
        return Err(Error::Io(format!("Boolean source: {message}")));
    }
    state
        .result
        .ok_or(Error::Missing("intermediate source output"))?
}
