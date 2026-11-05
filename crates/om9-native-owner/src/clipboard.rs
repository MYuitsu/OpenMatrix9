// SPDX-License-Identifier: LGPL-2.1-or-later
//! Windows OLE clipboard adapter. It owns no document or geometry state.
use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        Graphics::Gdi::{BITMAP, GetObjectW, HBITMAP, HGDIOBJ},
        System::{
            Com::{
                CoTaskMemFree, DATADIR_GET, DVASPECT_CONTENT, FORMATETC, IDataObject,
                STATFLAG_NONAME, STATSTG, STGMEDIUM, STGMEDIUM_0, STREAM_SEEK_SET,
                StructuredStorage::CreateStreamOnHGlobal, TYMED_GDI, TYMED_HGLOBAL, TYMED_ISTREAM,
            },
            DataExchange::{GetClipboardSequenceNumber, RegisterClipboardFormatW},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
            Ole::{
                CLIPBOARD_FORMAT, OleDuplicateData, OleFlushClipboard, OleGetClipboard,
                OleInitialize, OleSetClipboard, OleUninitialize, ReleaseStgMedium,
            },
        },
        UI::Shell::SHCreateDataObject,
    },
    core::w,
};

const MAGIC: &[u8; 8] = b"OM9CLIP1";
const MAX_BYTES: usize = 512 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Om9,
    Rhino3dm,
}

struct Ole;
impl Ole {
    fn new() -> Result<Self, String> {
        // OLE owns this thread's COM apartment only for the lifetime of this guard.
        unsafe { OleInitialize(None) }.map_err(|error| format!("OLE initialization: {error}"))?;
        Ok(Self)
    }
}
impl Drop for Ole {
    fn drop(&mut self) {
        unsafe { OleUninitialize() }
    }
}

fn format_id(format: Format) -> Result<u32, String> {
    let id = unsafe {
        RegisterClipboardFormatW(match format {
            Format::Om9 => w!("OpenMatrix9.ObjectArchive.v1"),
            Format::Rhino3dm => w!("Rhino 5.0 3DM Clip global mem"),
        })
    };
    if id == 0 {
        Err("Cannot register clipboard format".into())
    } else {
        Ok(id)
    }
}

/// A non-mutating check used to leave ordinary text paste with the editor.
pub fn has_geometry() -> Result<bool, String> {
    Ok(clipboard_win::is_format_avail(format_id(Format::Om9)?)
        || clipboard_win::is_format_avail(format_id(Format::Rhino3dm)?))
}

fn encode(format: Format, bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES {
        return Err("Invalid clipboard payload size".into());
    }
    if format == Format::Rhino3dm {
        return Ok(bytes.to_vec());
    }
    let mut data = Vec::new();
    data.try_reserve_exact(bytes.len() + 16)
        .map_err(|_| "Clipboard allocation failed")?;
    data.extend_from_slice(MAGIC);
    data.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    data.extend_from_slice(bytes);
    Ok(data)
}

fn decode(format: Format, bytes: Vec<u8>) -> Result<Vec<u8>, String> {
    if format == Format::Rhino3dm {
        return Ok(bytes);
    }
    if bytes.len() < 16 || &bytes[..8] != MAGIC {
        return Err("Invalid OM9 clipboard header".into());
    }
    let size = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let size = usize::try_from(size).map_err(|_| "Invalid OM9 clipboard length")?;
    if size == 0 || size > MAX_BYTES || size > bytes.len() - 16 {
        return Err("Invalid OM9 clipboard length".into());
    }
    Ok(bytes[16..16 + size].to_vec())
}

fn data_object() -> Result<IDataObject, String> {
    unsafe { SHCreateDataObject(None, None, None::<&IDataObject>) }
        .map_err(|error| format!("Clipboard data object: {error}"))
}

fn put_data(object: &IDataObject, format: &FORMATETC, data: &[u8]) -> Result<(), String> {
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE, data.len()) }
        .map_err(|error| format!("Clipboard allocation: {error}"))?;
    let ptr = unsafe { GlobalLock(memory) };
    if ptr.is_null() {
        let _ = unsafe { GlobalFree(Some(memory)) };
        return Err("Cannot lock clipboard memory".into());
    }
    unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast::<u8>(), data.len()) };
    let _ = unsafe { GlobalUnlock(memory) };
    let medium = STGMEDIUM {
        tymed: TYMED_HGLOBAL.0 as u32,
        u: STGMEDIUM_0 { hGlobal: memory },
        pUnkForRelease: std::mem::ManuallyDrop::new(None),
    };
    if let Err(error) = unsafe { object.SetData(format, &medium, true) } {
        let _ = unsafe { GlobalFree(Some(memory)) };
        return Err(format!("Cannot prepare clipboard: {error}"));
    }
    Ok(())
}

/// A clipboard wrapper obtained from OLE may proxy the live clipboard and
/// recurse after replacement. Materialize every supported format while the old
/// owner still exists; refuse publication if any format cannot be preserved.
fn snapshot_previous(previous: &IDataObject) -> Result<IDataObject, String> {
    let backup = data_object()?;
    let formats = unsafe { previous.EnumFormatEtc(DATADIR_GET.0 as u32) }
        .map_err(|error| format!("Cannot enumerate prior clipboard: {error}"))?;
    let mut count = 0usize;
    let mut total = 0usize;
    loop {
        let mut row = [FORMATETC::default()];
        let mut fetched = 0;
        let result = unsafe { formats.Next(&mut row, Some(&mut fetched)) };
        if fetched == 0 {
            if result.0 < 0 {
                return Err(format!("Cannot enumerate prior clipboard: {result:?}"));
            }
            break;
        }
        count += 1;
        if count > 256 {
            return Err("Prior clipboard has too many formats".into());
        }
        let mut format = row[0];
        if !format.ptd.is_null() {
            unsafe { CoTaskMemFree(Some(format.ptd.cast())) };
            return Err(
                "Prior clipboard has a device-specific format; copy was not performed".into(),
            );
        }
        let stream = format.tymed & TYMED_HGLOBAL.0 as u32 == 0
            && format.tymed & TYMED_ISTREAM.0 as u32 != 0;
        let bitmap = !stream
            && format.tymed & TYMED_HGLOBAL.0 as u32 == 0
            && format.tymed & TYMED_GDI.0 as u32 != 0
            && matches!(format.cfFormat, 2 | 0x82);
        if format.tymed & TYMED_HGLOBAL.0 as u32 == 0 && !bitmap && !stream {
            return Err(format!(
                "Prior clipboard format {} requires unsupported media {:#x}; copy was not performed",
                format.cfFormat, format.tymed
            ));
        }
        format.tymed = if stream {
            TYMED_ISTREAM.0 as u32
        } else if bitmap {
            TYMED_GDI.0 as u32
        } else {
            TYMED_HGLOBAL.0 as u32
        };
        let mut medium = unsafe { previous.GetData(&format) }
            .map_err(|error| format!("Cannot preserve prior clipboard: {error}"))?;
        let result = (|| {
            if stream {
                if medium.tymed != TYMED_ISTREAM.0 as u32 {
                    return Err("Prior clipboard stream medium changed".into());
                }
                let source = unsafe { &*medium.u.pstm }
                    .as_ref()
                    .ok_or("Missing prior clipboard stream")?;
                let mut stat = STATSTG::default();
                unsafe { source.Stat(&mut stat, STATFLAG_NONAME) }
                    .map_err(|e| format!("Cannot inspect prior clipboard stream: {e}"))?;
                let size =
                    usize::try_from(stat.cbSize).map_err(|_| "Prior clipboard stream too large")?;
                total = total.checked_add(size).ok_or("Prior clipboard too large")?;
                if total > MAX_BYTES {
                    return Err("Prior clipboard stream too large".into());
                }
                let mut bytes = Vec::new();
                bytes
                    .try_reserve_exact(size)
                    .map_err(|_| "Clipboard allocation failed")?;
                bytes.resize(size, 0);
                unsafe { source.Seek(0, STREAM_SEEK_SET, None) }
                    .map_err(|e| format!("Cannot seek prior clipboard stream: {e}"))?;
                let mut read = 0;
                unsafe { source.Read(bytes.as_mut_ptr().cast(), size as u32, Some(&mut read)) }
                    .ok()
                    .map_err(|e| format!("Cannot read prior clipboard stream: {e}"))?;
                if read as usize != size {
                    return Err("Prior clipboard stream was truncated".into());
                }
                let target = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true) }
                    .map_err(|e| format!("Cannot duplicate prior clipboard stream: {e}"))?;
                let mut written = 0;
                unsafe { target.Write(bytes.as_ptr().cast(), size as u32, Some(&mut written)) }
                    .ok()
                    .map_err(|e| format!("Cannot copy prior clipboard stream: {e}"))?;
                if written as usize != size {
                    return Err("Prior clipboard stream copy was truncated".into());
                }
                unsafe { target.Seek(0, STREAM_SEEK_SET, None) }
                    .map_err(|e| format!("Cannot rewind prior clipboard stream: {e}"))?;
                let mut copy = STGMEDIUM {
                    tymed: TYMED_ISTREAM.0 as u32,
                    u: STGMEDIUM_0 {
                        pstm: std::mem::ManuallyDrop::new(Some(target)),
                    },
                    pUnkForRelease: std::mem::ManuallyDrop::new(None),
                };
                if let Err(error) = unsafe { backup.SetData(&format, &copy, true) } {
                    unsafe { ReleaseStgMedium(&mut copy) };
                    return Err(format!("Cannot preserve prior clipboard stream: {error}"));
                }
                return Ok(());
            }
            if bitmap {
                if medium.tymed != TYMED_GDI.0 as u32 {
                    return Err("Prior clipboard bitmap medium changed".into());
                }
                let handle = unsafe { medium.u.hBitmap };
                let mut info = BITMAP::default();
                let got = unsafe {
                    GetObjectW(
                        HGDIOBJ(handle.0),
                        std::mem::size_of::<BITMAP>() as i32,
                        Some((&mut info as *mut BITMAP).cast()),
                    )
                };
                if got != std::mem::size_of::<BITMAP>() as i32
                    || info.bmWidthBytes <= 0
                    || info.bmHeight == 0
                {
                    return Err("Cannot inspect prior clipboard bitmap".into());
                }
                let size = (info.bmWidthBytes as usize)
                    .checked_mul(info.bmHeight.unsigned_abs() as usize)
                    .ok_or("Prior clipboard bitmap too large")?;
                total = total.checked_add(size).ok_or("Prior clipboard too large")?;
                if size == 0 || total > MAX_BYTES {
                    return Err("Prior clipboard bitmap too large".into());
                }
                let duplicate = unsafe {
                    OleDuplicateData(
                        HANDLE(handle.0),
                        CLIPBOARD_FORMAT(format.cfFormat),
                        GMEM_MOVEABLE,
                    )
                };
                if duplicate.0.is_null() {
                    return Err("Cannot duplicate prior clipboard bitmap".into());
                }
                let mut copy = STGMEDIUM {
                    tymed: TYMED_GDI.0 as u32,
                    u: STGMEDIUM_0 {
                        hBitmap: HBITMAP(duplicate.0),
                    },
                    pUnkForRelease: std::mem::ManuallyDrop::new(None),
                };
                if let Err(error) = unsafe { backup.SetData(&format, &copy, true) } {
                    unsafe {
                        ReleaseStgMedium(&mut copy);
                    }
                    return Err(format!("Cannot preserve prior clipboard bitmap: {error}"));
                }
                return Ok(());
            }
            if medium.tymed != TYMED_HGLOBAL.0 as u32 {
                return Err("Prior clipboard has a non-memory format".to_owned());
            }
            let handle = unsafe { medium.u.hGlobal };
            let size = unsafe { GlobalSize(handle) };
            total = total.checked_add(size).ok_or("Prior clipboard too large")?;
            if size == 0 || total > MAX_BYTES {
                return Err("Prior clipboard data is empty or too large".into());
            }
            let ptr = unsafe { GlobalLock(handle) };
            if ptr.is_null() {
                return Err("Cannot lock prior clipboard data".into());
            }
            let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), size) };
            let result = put_data(&backup, &format, bytes);
            let _ = unsafe { GlobalUnlock(handle) };
            result
        })();
        unsafe { ReleaseStgMedium(&mut medium) };
        result?;
    }
    Ok(backup)
}

/// Publish only after the new OLE data object has been materialized. Keep the
/// previous OLE object available for rollback if publication/flush fails.
pub fn publish(format: Format, payload: &[u8]) -> Result<(), String> {
    publish_inner(format, payload, false)
}

fn publish_inner(
    format: Format,
    payload: &[u8],
    simulate_flush_failure: bool,
) -> Result<(), String> {
    let data = encode(format, payload)?;
    let id = format_id(format)?;
    let _ole = Ole::new()?;
    let before = unsafe { GetClipboardSequenceNumber() };
    if before == 0 {
        return Err("Clipboard sequence unavailable".into());
    }
    let previous =
        unsafe { OleGetClipboard() }.map_err(|error| format!("Clipboard unavailable: {error}"))?;
    let previous = snapshot_previous(&previous)?;
    if unsafe { GetClipboardSequenceNumber() } != before {
        return Err("Clipboard changed; try again".into());
    }
    let object = data_object()?;
    let format = FORMATETC {
        cfFormat: id as u16,
        ptd: std::ptr::null_mut(),
        dwAspect: DVASPECT_CONTENT.0,
        lindex: -1,
        tymed: TYMED_HGLOBAL.0 as u32,
    };
    put_data(&object, &format, &data)?;
    if unsafe { GetClipboardSequenceNumber() } != before {
        return Err("Clipboard changed; try again".into());
    }
    unsafe { OleSetClipboard(&object) }
        .map_err(|error| format!("Cannot publish clipboard: {error}"))?;
    let flush = if simulate_flush_failure {
        Err("injected flush failure".to_owned())
    } else {
        unsafe { OleFlushClipboard() }.map_err(|error| error.to_string())
    };
    if let Err(error) = flush {
        use windows::core::Interface;
        if unsafe { windows_sys::Win32::System::Ole::OleIsCurrentClipboard(object.as_raw()) } == 0 {
            unsafe { OleSetClipboard(&previous) }
                .and_then(|_| unsafe { OleFlushClipboard() })
                .map_err(|restore| {
                    format!("Clipboard flush failed ({error}); rollback failed: {restore}")
                })?;
        }
        return Err(format!("Clipboard flush failed: {error}"));
    }
    Ok(())
}

/// Capture one supported format while holding the system clipboard open.
pub fn read() -> Result<(Format, Vec<u8>, Option<Vec<u8>>), String> {
    let om9 = format_id(Format::Om9)?;
    let rhino = format_id(Format::Rhino3dm)?;
    let _guard =
        clipboard_win::Clipboard::new().map_err(|error| format!("Clipboard busy: {error}"))?;
    let format = if clipboard_win::is_format_avail(om9) {
        Format::Om9
    } else if clipboard_win::is_format_avail(rhino) {
        Format::Rhino3dm
    } else {
        return Err("Clipboard does not contain OM9 or Rhino geometry".into());
    };
    let id = if format == Format::Om9 { om9 } else { rhino };
    let size = clipboard_win::size(id)
        .ok_or("Cannot read clipboard geometry")?
        .get();
    if size > MAX_BYTES + 16 {
        return Err("Clipboard geometry exceeds size limit".into());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|_| "Clipboard allocation failed")?;
    bytes.resize(size, 0);
    clipboard_win::raw::get(id, &mut bytes)
        .map_err(|error| format!("Clipboard read failed: {error}"))?;
    let metadata = if format == Format::Rhino3dm {
        let id = unsafe { RegisterClipboardFormatW(w!("OM9.LayerSession.v1")) };
        if id == 0 {
            return Err("Cannot register clipboard metadata format".into());
        }
        if clipboard_win::is_format_avail(id) {
            let size = clipboard_win::size(id)
                .ok_or("Cannot read clipboard metadata")?
                .get();
            if size > 256 * 1024 * 1024 {
                return Err("Clipboard metadata exceeds size limit".into());
            }
            let mut data = Vec::new();
            data.try_reserve_exact(size)
                .map_err(|_| "Clipboard metadata allocation failed")?;
            data.resize(size, 0);
            clipboard_win::raw::get(id, &mut data)
                .map_err(|error| format!("Clipboard metadata read failed: {error}"))?;
            Some(data)
        } else {
            None
        }
    } else {
        None
    };
    Ok((format, decode(format, bytes)?, metadata))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn om9_envelope_checks_length_and_ignores_global_allocation_padding() {
        let mut data = encode(Format::Om9, b"abc").unwrap();
        data.extend_from_slice(&[0; 8]);
        assert_eq!(decode(Format::Om9, data).unwrap(), b"abc");
        assert!(decode(Format::Om9, b"invalid".to_vec()).is_err());
    }
    #[test]
    fn snapshot_preserves_detached_bitmap_without_touching_system_clipboard() {
        use windows::Win32::Graphics::Gdi::{CreateBitmap, GetBitmapBits};
        let _ole = Ole::new().unwrap();
        let source = data_object().unwrap();
        let pixels = [1u8, 2, 3, 4, 5, 6, 7, 8];
        let bitmap = unsafe { CreateBitmap(2, 1, 1, 32, Some(pixels.as_ptr().cast())) };
        assert!(!bitmap.0.is_null());
        let format = FORMATETC {
            cfFormat: 2,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_GDI.0 as u32,
        };
        let medium = STGMEDIUM {
            tymed: TYMED_GDI.0 as u32,
            u: STGMEDIUM_0 { hBitmap: bitmap },
            pUnkForRelease: std::mem::ManuallyDrop::new(None),
        };
        unsafe { source.SetData(&format, &medium, true) }.unwrap();
        let backup = snapshot_previous(&source).unwrap();
        drop(source);
        let mut captured = unsafe { backup.GetData(&format) }.unwrap();
        let mut actual = [0u8; 8];
        assert_eq!(
            unsafe { GetBitmapBits(captured.u.hBitmap, 8, actual.as_mut_ptr().cast()) },
            8
        );
        unsafe { ReleaseStgMedium(&mut captured) };
        assert_eq!(actual, pixels);
    }
    #[test]
    fn snapshot_preserves_detached_stream_without_touching_system_clipboard() {
        let _ole = Ole::new().unwrap();
        let source = data_object().unwrap();
        let stream = unsafe { CreateStreamOnHGlobal(HGLOBAL::default(), true) }.unwrap();
        let bytes = b"clipboard stream fixture";
        let mut written = 0;
        unsafe {
            stream.Write(
                bytes.as_ptr().cast(),
                bytes.len() as u32,
                Some(&mut written),
            )
        }
        .ok()
        .unwrap();
        assert_eq!(written as usize, bytes.len());
        let format = FORMATETC {
            cfFormat: format_id(Format::Om9).unwrap() as u16,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_ISTREAM.0 as u32,
        };
        let medium = STGMEDIUM {
            tymed: TYMED_ISTREAM.0 as u32,
            u: STGMEDIUM_0 {
                pstm: std::mem::ManuallyDrop::new(Some(stream)),
            },
            pUnkForRelease: std::mem::ManuallyDrop::new(None),
        };
        unsafe { source.SetData(&format, &medium, true) }.unwrap();
        let backup = snapshot_previous(&source).unwrap();
        drop(source);
        let mut captured = unsafe { backup.GetData(&format) }.unwrap();
        let stream = unsafe { &*captured.u.pstm }.as_ref().unwrap();
        let mut actual = [0u8; 24];
        let mut read = 0;
        unsafe {
            stream.Read(
                actual.as_mut_ptr().cast(),
                actual.len() as u32,
                Some(&mut read),
            )
        }
        .ok()
        .unwrap();
        assert_eq!(&actual[..read as usize], bytes);
        unsafe { ReleaseStgMedium(&mut captured) };
    }
    #[test]
    #[ignore = "temporarily publishes to the interactive Windows clipboard"]
    fn windows_clipboard_roundtrip_restores_previous_owner() {
        eprintln!("clipboard test: initialize");
        let _ole = Ole::new().unwrap();
        eprintln!("clipboard test: capture");
        let previous =
            snapshot_previous(&unsafe { OleGetClipboard() }.expect("capture prior clipboard"))
                .expect("materialize prior clipboard");
        struct Restore {
            previous: IDataObject,
            owned_sequence: u32,
        }
        impl Drop for Restore {
            fn drop(&mut self) {
                if self.owned_sequence != 0
                    && unsafe { GetClipboardSequenceNumber() } == self.owned_sequence
                {
                    unsafe { OleSetClipboard(&self.previous).and_then(|_| OleFlushClipboard()) }
                        .expect("restore previous clipboard");
                }
            }
        }
        let mut restore = Restore {
            previous,
            owned_sequence: 0,
        };
        let text = "OM9 clipboard rollback fixture\0"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        let text_object = data_object().unwrap();
        let text_format = FORMATETC {
            cfFormat: 13,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
        };
        put_data(&text_object, &text_format, &text).unwrap();
        unsafe { OleSetClipboard(&text_object).and_then(|_| OleFlushClipboard()) }
            .expect("prepare text fixture");
        restore.owned_sequence = unsafe { GetClipboardSequenceNumber() };
        assert!(publish_inner(Format::Om9, b"test-om9-archive", true).is_err());
        restore.owned_sequence = unsafe { GetClipboardSequenceNumber() };
        let restored: String = clipboard_win::get_clipboard(clipboard_win::formats::Unicode)
            .expect("read restored text");
        assert_eq!(restored, "OM9 clipboard rollback fixture");
        eprintln!("clipboard test: publish");
        publish(Format::Om9, b"test-om9-archive").expect("publish");
        eprintln!("clipboard test: read");
        restore.owned_sequence = unsafe { GetClipboardSequenceNumber() };
        let (format, bytes, metadata) = read().expect("read");
        assert_eq!(format, Format::Om9);
        assert_eq!(bytes, b"test-om9-archive");
        assert!(metadata.is_none());
        publish(Format::Rhino3dm, b"test-rhino-archive").expect("publish Rhino");
        restore.owned_sequence = unsafe { GetClipboardSequenceNumber() };
        let (format, bytes, metadata) = read().expect("read Rhino");
        assert_eq!(format, Format::Rhino3dm);
        assert_eq!(bytes, b"test-rhino-archive");
        assert!(metadata.is_none());
    }
    #[test]
    fn native_key_hook_distinguishes_copy_3dm_and_paste() {
        use windows_sys::Win32::UI::{
            Input::KeyboardAndMouse::{GetKeyboardState, SetKeyboardState, VK_CONTROL, VK_SHIFT},
            WindowsAndMessaging::{CreateWindowExW, DestroyWindow, SendMessageW, WM_KEYDOWN},
        };
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        let mut hook = ShortcutHook::attach(hwnd as usize).unwrap();
        let mut prior = [0u8; 256];
        assert_ne!(unsafe { GetKeyboardState(prior.as_mut_ptr()) }, 0);
        let mut keys = prior;
        keys[VK_CONTROL as usize] = 0x80;
        keys[VK_SHIFT as usize] = 0;
        assert_ne!(unsafe { SetKeyboardState(keys.as_ptr()) }, 0);
        unsafe {
            SendMessageW(hwnd, WM_KEYDOWN, 0x43, 0);
        }
        keys[VK_SHIFT as usize] = 0x80;
        assert_ne!(unsafe { SetKeyboardState(keys.as_ptr()) }, 0);
        unsafe {
            SendMessageW(hwnd, WM_KEYDOWN, 0x43, 0);
        }
        keys[VK_SHIFT as usize] = 0;
        assert_ne!(unsafe { SetKeyboardState(keys.as_ptr()) }, 0);
        unsafe {
            SendMessageW(hwnd, WM_KEYDOWN, 0x56, 0);
        }
        assert_ne!(unsafe { SetKeyboardState(prior.as_ptr()) }, 0);
        assert_eq!(
            hook.drain(),
            vec![Shortcut::Copy, Shortcut::Copy3dm, Shortcut::Paste]
        );
        drop(hook);
        unsafe {
            DestroyWindow(hwnd);
        }
    }
    #[test]
    #[ignore = "reads clipboard format identifiers only"]
    fn inspect_current_clipboard_media() {
        let _ole = Ole::new().unwrap();
        let previous = unsafe { OleGetClipboard() }.unwrap();
        let formats = unsafe { previous.EnumFormatEtc(DATADIR_GET.0 as u32) }.unwrap();
        loop {
            let mut row = [FORMATETC::default()];
            let mut fetched = 0;
            let result = unsafe { formats.Next(&mut row, Some(&mut fetched)) };
            if fetched == 0 {
                assert!(result.0 >= 0);
                break;
            }
            eprintln!(
                "format={} media={:#x} device={}",
                row[0].cfFormat,
                row[0].tymed,
                !row[0].ptd.is_null()
            );
            if !row[0].ptd.is_null() {
                unsafe {
                    CoTaskMemFree(Some(row[0].ptd.cast()));
                }
            }
        }
    }
    #[test]
    #[ignore = "reads and duplicates the interactive clipboard without publishing"]
    fn snapshot_current_clipboard_without_publishing() {
        let _ole = Ole::new().unwrap();
        let previous = unsafe { OleGetClipboard() }.unwrap();
        snapshot_previous(&previous).expect("snapshot current clipboard");
    }
}

/// Capture copy/paste keydowns before egui-winit converts them to text-only
/// events (or drops Ctrl+V when the clipboard has no text).
pub struct ShortcutHook {
    hwnd: windows_sys::Win32::Foundation::HWND,
    shared: std::sync::Arc<std::sync::Mutex<Vec<Shortcut>>>,
    callback_ref: *const std::sync::Mutex<Vec<Shortcut>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Shortcut {
    Copy,
    Copy3dm,
    Paste,
}

unsafe extern "system" fn shortcut_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    msg: u32,
    key: usize,
    flags: isize,
    _id: usize,
    data: usize,
) -> isize {
    use windows_sys::Win32::UI::{
        Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT},
        Shell::DefSubclassProc,
        WindowsAndMessaging::WM_KEYDOWN,
    };
    if msg == WM_KEYDOWN && flags & (1 << 30) == 0 {
        let down = |key: u16| unsafe { GetKeyState(i32::from(key)) } < 0;
        if down(VK_CONTROL) && !down(VK_MENU) {
            let shortcut = match (key, down(VK_SHIFT)) {
                (0x43, false) => Some(Shortcut::Copy),
                (0x43, true) => Some(Shortcut::Copy3dm),
                (0x56, false) => Some(Shortcut::Paste),
                _ => None,
            };
            if let Some(shortcut) = shortcut {
                let shared = unsafe { &*(data as *const std::sync::Mutex<Vec<Shortcut>>) };
                if let Ok(mut events) = shared.lock() {
                    if events.len() < 16 {
                        events.push(shortcut);
                    }
                }
            }
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, key, flags) }
}

impl ShortcutHook {
    /// Must be attached, drained and dropped on the window's UI thread.
    pub fn attach(hwnd: usize) -> Result<Self, String> {
        use windows_sys::Win32::UI::Shell::SetWindowSubclass;
        if hwnd == 0 {
            return Err("Missing clipboard shortcut window".into());
        }
        let shared = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let callback_ref = std::sync::Arc::into_raw(shared.clone());
        let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
        let ok = unsafe {
            SetWindowSubclass(hwnd, Some(shortcut_proc), 0x4f4d3943, callback_ref as usize)
        };
        if ok == 0 {
            unsafe {
                drop(std::sync::Arc::from_raw(callback_ref));
            }
            return Err("Cannot attach clipboard shortcut handler".into());
        }
        Ok(Self {
            hwnd,
            shared,
            callback_ref,
        })
    }
    pub fn drain(&mut self) -> Vec<Shortcut> {
        self.shared
            .lock()
            .map(|mut events| std::mem::take(&mut *events))
            .unwrap_or_default()
    }
}
impl Drop for ShortcutHook {
    fn drop(&mut self) {
        unsafe {
            if windows_sys::Win32::UI::Shell::RemoveWindowSubclass(
                self.hwnd,
                Some(shortcut_proc),
                0x4f4d3943,
            ) != 0
            {
                drop(std::sync::Arc::from_raw(self.callback_ref));
            }
        }
    }
}
