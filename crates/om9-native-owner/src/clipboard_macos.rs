// SPDX-License-Identifier: LGPL-2.1-or-later
//! AppKit pasteboard and keyboard events for one OM9 window.
use crate::clipboard_types::MAX_BYTES;
pub use crate::clipboard_types::{Format, Shortcut};
use block2::RcBlock;
use objc2::{
    MainThreadMarker, msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
};
use objc2_app_kit::{
    NSEvent, NSEventMask, NSEventModifierFlags as Modifiers, NSPasteboard, NSPasteboardItem,
    NSPasteboardWriting, NSView,
};
use objc2_foundation::{NSArray, NSData, NSString};
use raw_window_handle::{RawWindowHandle, WindowHandle};
use std::{cell::RefCell, ptr::NonNull, rc::Rc};

const OM9: &str = "org.openmatrix9.clipboard.object-archive";
const THREE_DM: &str = "org.openmatrix9.clipboard.rhino-3dm";

type Contents = (Format, Vec<u8>, Option<Vec<u8>>);

pub struct Pasteboard {
    board: Retained<NSPasteboard>,
    isolated: bool,
}

impl Pasteboard {
    fn general() -> Self {
        Self {
            board: NSPasteboard::generalPasteboard(),
            isolated: false,
        }
    }

    /// A temporary native pasteboard for validation, independent of the user's clipboard.
    pub fn isolated() -> Self {
        Self {
            board: NSPasteboard::pasteboardWithUniqueName(),
            isolated: true,
        }
    }

    fn format(&self) -> Option<(Format, Retained<NSString>)> {
        let types = [NSString::from_str(OM9), NSString::from_str(THREE_DM)];
        let selected = self
            .board
            .availableTypeFromArray(&NSArray::from_retained_slice(&types))?;
        let format = if selected.isEqualToString(&types[0]) {
            Format::Om9
        } else {
            Format::Rhino3dm
        };
        Some((format, selected))
    }

    pub fn publish(&self, format: Format, bytes: &[u8]) -> Result<(), String> {
        if bytes.is_empty() || bytes.len() > MAX_BYTES {
            return Err("Invalid clipboard geometry size".into());
        }
        let name = NSString::from_str(match format {
            Format::Om9 => OM9,
            Format::Rhino3dm => THREE_DM,
        });
        let item = NSPasteboardItem::new();
        if !item.setData_forType(&NSData::with_bytes(bytes), &name) {
            return Err("Cannot prepare clipboard geometry".into());
        }
        let item = ProtocolObject::<dyn NSPasteboardWriting>::from_retained(item);
        let items = NSArray::from_retained_slice(&[item]);
        self.board.clearContents();
        self.board
            .writeObjects(&items)
            .then_some(())
            .ok_or_else(|| "Cannot publish clipboard geometry".into())
    }

    pub fn read(&self) -> Result<Contents, String> {
        let before = self.board.changeCount();
        let (format, name) = self
            .format()
            .ok_or("Clipboard does not contain OM9 or 3DM geometry")?;
        let data = self
            .board
            .dataForType(&name)
            .ok_or("Cannot read clipboard geometry")?;
        let size = data.len();
        if size == 0 || size > MAX_BYTES {
            return Err("Invalid clipboard geometry size".into());
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(size)
            .map_err(|_| "Clipboard allocation failed")?;
        bytes.resize(size, 0);
        // SAFETY: AppKit returned a retained NSData. The destination owns size writable bytes.
        unsafe { data.getBytes_length(NonNull::from(bytes.as_mut_slice()).cast(), size) };
        if self.board.changeCount() != before {
            return Err("Clipboard changed; try again".into());
        }
        Ok((format, bytes, None))
    }
}

impl Drop for Pasteboard {
    fn drop(&mut self) {
        if self.isolated {
            // SAFETY: The pasteboard was created for this owner alone. This AppKit
            // method releases the named server resource, not the retained object.
            let _: () = unsafe { msg_send![&*self.board, releaseGlobally] };
        }
    }
}

pub fn has_geometry() -> Result<bool, String> {
    Ok(Pasteboard::general().format().is_some())
}

pub fn publish(format: Format, bytes: &[u8]) -> Result<(), String> {
    Pasteboard::general().publish(format, bytes)
}

pub fn read() -> Result<Contents, String> {
    Pasteboard::general().read()
}

pub struct ShortcutHook {
    monitor: Retained<AnyObject>,
    queued: Rc<RefCell<Vec<Shortcut>>>,
}

impl ShortcutHook {
    pub fn attach(handle: WindowHandle<'_>, wake: impl Fn() + 'static) -> Result<Self, String> {
        let _main =
            MainThreadMarker::new().ok_or("Clipboard shortcuts require the AppKit main thread")?;
        let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
            return Err("No AppKit clipboard shortcut window".into());
        };
        // SAFETY: WindowHandle guarantees the NSView is live during this borrow.
        // The main-thread gate above covers AppKit access. Only the window number is saved.
        let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
        let window = view
            .window()
            .ok_or("No macOS clipboard shortcut window")?
            .windowNumber();
        let queued = Rc::new(RefCell::new(Vec::new()));
        let pending = Rc::clone(&queued);
        let callback = RcBlock::new(move |pointer: NonNull<NSEvent>| {
            // SAFETY: AppKit supplies this live key event for the duration of the callback.
            let event = unsafe { pointer.as_ref() };
            if event.windowNumber() == window {
                if let Some(shortcut) = shortcut_for_event(event) {
                    {
                        let mut events = pending.borrow_mut();
                        if events.len() < 32 {
                            events.push(shortcut);
                        }
                    }
                    wake();
                }
            }
            pointer.as_ptr()
        });
        // SAFETY: The local monitor always returns the original borrowed event. AppKit
        // calls it on the main thread and copies the block for the life of the monitor.
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &callback)
        }
        .ok_or("Cannot install macOS clipboard shortcut monitor")?;
        Ok(Self { monitor, queued })
    }

    pub fn drain(&mut self) -> Vec<Shortcut> {
        std::mem::take(&mut *self.queued.borrow_mut())
    }
}

impl Drop for ShortcutHook {
    fn drop(&mut self) {
        // SAFETY: This token came from the installed local monitor. Rc keeps the owner on its thread.
        unsafe { NSEvent::removeMonitor(&self.monitor) };
    }
}

fn shortcut_for_event(event: &NSEvent) -> Option<Shortcut> {
    let modifiers = event.modifierFlags();
    if !modifiers.contains(Modifiers::Command) || event.isARepeat() {
        return None;
    }
    let characters = event.charactersIgnoringModifiers()?.to_string();
    shortcut_for_key(&characters, event.keyCode(), modifiers)
}

fn shortcut_for_key(characters: &str, key_code: u16, modifiers: Modifiers) -> Option<Shortcut> {
    if !modifiers.contains(Modifiers::Command)
        || modifiers.intersects(Modifiers::Control | Modifiers::Option)
    {
        return None;
    }
    let mut bytes = characters.bytes();
    let key = match (bytes.next(), bytes.next()) {
        (Some(letter), None) if letter.is_ascii_alphabetic() => letter.to_ascii_lowercase(),
        // Match egui-winit's physical-key fallback on layouts without Latin letters.
        _ => match key_code {
            8 => b'c',
            9 => b'v',
            _ => return None,
        },
    };
    match (key, modifiers.contains(Modifiers::Shift)) {
        (b'c', false) => Some(Shortcut::Copy),
        (b'c', true) => Some(Shortcut::Copy3dm),
        (b'v', false) => Some(Shortcut::Paste),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_shortcuts_use_command_and_match_non_latin_layouts() {
        let cmd = Modifiers::Command;
        let shift = Modifiers::Shift;
        assert_eq!(shortcut_for_key("c", 8, cmd), Some(Shortcut::Copy));
        assert_eq!(
            shortcut_for_key("C", 8, cmd | shift),
            Some(Shortcut::Copy3dm)
        );
        assert_eq!(shortcut_for_key("v", 9, cmd), Some(Shortcut::Paste));
        assert_eq!(shortcut_for_key("м", 9, cmd), Some(Shortcut::Paste));
        assert_eq!(shortcut_for_key("v", 9, cmd | shift), None);
        assert_eq!(shortcut_for_key("c", 8, Modifiers::Control), None);
        assert_eq!(shortcut_for_key("v", 9, cmd | Modifiers::Option), None);
    }
}
