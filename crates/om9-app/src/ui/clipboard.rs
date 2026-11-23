// SPDX-License-Identifier: LGPL-2.1-or-later
//! Clipboard command routing. The platform adapter owns system clipboard access.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClipboardAction {
    Copy,
    Copy3dm,
    Paste,
}

impl ClipboardAction {
    pub(super) const fn menu_label(self) -> &'static str {
        match self {
            Self::Copy if cfg!(target_os = "macos") => "Copy\t⌘C",
            Self::Copy3dm if cfg!(target_os = "macos") => "Copy as 3DM\t⇧⌘C",
            Self::Paste if cfg!(target_os = "macos") => "Paste\t⌘V",
            Self::Copy => "Copy\tCtrl+C",
            Self::Copy3dm => "Copy as 3DM\tCtrl+Shift+C",
            Self::Paste => "Paste\tCtrl+V",
        }
    }
}

#[cfg(any(windows, target_os = "macos"))]
use om9_native_owner::clipboard::{self as system_clipboard, Format, Shortcut};

#[cfg(any(windows, target_os = "macos"))]
fn routes_to_model(
    event: om9_native_owner::clipboard::Shortcut,
    selected: bool,
    has_geometry: bool,
) -> bool {
    match event {
        Shortcut::Copy | Shortcut::Copy3dm => selected,
        Shortcut::Paste => has_geometry,
    }
}

#[cfg(any(windows, target_os = "macos"))]
fn apply_clipboard_action(
    controller: &mut Controller,
    action: ClipboardAction,
    publish: impl FnOnce(Format, &[u8]) -> std::result::Result<(), String>,
    read: impl FnOnce() -> std::result::Result<(Format, Vec<u8>, Option<Vec<u8>>), String>,
) -> Result<()> {
    match action {
        ClipboardAction::Copy => {
            let bytes = controller.copy_selected_om9()?;
            publish(Format::Om9, &bytes).map_err(Error::Io)?;
        }
        ClipboardAction::Copy3dm => {
            let bytes = controller.copy_selected_3dm()?;
            publish(Format::Rhino3dm, &bytes).map_err(Error::Io)?;
        }
        ClipboardAction::Paste => {
            let (format, bytes, metadata) = read().map_err(Error::Io)?;
            match format {
                Format::Om9 => {
                    controller.paste_om9(&bytes)?;
                }
                Format::Rhino3dm => {
                    controller.paste_3dm(&bytes, metadata.as_deref())?;
                }
            }
        }
    }
    Ok(())
}

impl MatrixApp {
    pub(super) fn clipboard_action(&mut self, action: ClipboardAction) {
        #[cfg(any(windows, target_os = "macos"))]
        let result = apply_clipboard_action(
            &mut self.controller,
            action,
            system_clipboard::publish,
            system_clipboard::read,
        );
        #[cfg(not(any(windows, target_os = "macos")))]
        let result: Result<()> = Err(Error::Unsupported("geometry clipboard on this platform"));
        let success = result.is_ok();
        self.feedback(result);
        if success {
            self.status.info(match action {
                ClipboardAction::Copy => "Copied OM9 objects",
                ClipboardAction::Copy3dm => "Copied Rhino 3DM objects",
                ClipboardAction::Paste => "Pasted objects",
            });
        }
    }

    pub(super) fn attach_clipboard_shortcuts(
        &mut self,
        frame: &eframe::Frame,
        ctx: &egui::Context,
    ) {
        #[cfg(any(windows, target_os = "macos"))]
        {
            use raw_window_handle::HasWindowHandle;
            if self.clipboard_hook_attempted {
                return;
            }
            self.clipboard_hook_attempted = true;
            let hook = frame
                .window_handle()
                .map_err(|error| error.to_string())
                .and_then(|handle| {
                    #[cfg(windows)]
                    match handle.as_raw() {
                        raw_window_handle::RawWindowHandle::Win32(handle) => {
                            system_clipboard::ShortcutHook::attach(handle.hwnd.get() as usize)
                        }
                        _ => Err("No Windows clipboard shortcut window".into()),
                    }
                    #[cfg(target_os = "macos")]
                    {
                        let ctx = ctx.clone();
                        system_clipboard::ShortcutHook::attach(handle, move || {
                            ctx.request_repaint()
                        })
                    }
                });
            match hook {
                Ok(hook) => self.clipboard_shortcut_hook = Some(hook),
                Err(error) => {
                    self.status.error(&error);
                }
            }
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = frame;
        #[cfg(not(target_os = "macos"))]
        let _ = ctx;
    }

    pub(super) fn clipboard_shortcuts_from_input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
    ) {
        #[cfg(any(windows, target_os = "macos"))]
        {
            let events = self
                .clipboard_shortcut_hook
                .as_mut()
                .map(|hook| hook.drain())
                .unwrap_or_default();
            if events.is_empty() {
                return;
            }
            let cmd = egui::Id::new("matrix-command-transcript");
            let focused = ctx.memory(|memory| memory.focused());
            let selected_text = focused == Some(cmd)
                && TextEdit::load_state(ctx, cmd)
                    .and_then(|state| state.cursor.char_range())
                    .is_some_and(|range| !range.is_empty());
            let text_owner = ctx.egui_wants_keyboard_input() && focused != Some(cmd)
                || focused == Some(cmd) && (!self.transcript.draft().is_empty() || selected_text);
            let modal = self.controller.is_running()
                || self.controller.file_operation().is_some()
                || self.controller.layer_dialog().is_some()
                || egui::Popup::is_any_open(ctx)
                || self.cmd_ime_composing
                || self.layer_dialog_ime_composing
                || self.file_ime_composing
                || raw
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Ime(_)));
            if !raw.focused || text_owner || modal {
                return;
            }
            let has_geometry = if events.contains(&Shortcut::Paste) {
                om9_native_owner::clipboard::has_geometry().unwrap_or(false)
            } else {
                false
            };
            let events: Vec<_> = events
                .into_iter()
                .filter(|event| {
                    routes_to_model(*event, !self.controller.selected().is_empty(), has_geometry)
                })
                .collect();
            if events.is_empty() {
                return;
            }
            raw.events.retain(|event| match event {
                egui::Event::Copy | egui::Event::Paste(_) => false,
                egui::Event::Key {
                    key: Key::C | Key::V,
                    modifiers,
                    ..
                } if modifiers.command => false,
                _ => true,
            });
            for event in events {
                self.clipboard_action(match event {
                    Shortcut::Copy => ClipboardAction::Copy,
                    Shortcut::Copy3dm => ClipboardAction::Copy3dm,
                    Shortcut::Paste => ClipboardAction::Paste,
                });
            }
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        let _ = (ctx, raw);
    }
}

/// Exercises the normal command handlers against a private AppKit pasteboard
/// from the extracted application, with the bundled native workers.
#[cfg(target_os = "macos")]
pub fn smoke_geometry_clipboard() -> Result<()> {
    use om9_core::geometry::Geometry;
    let make_document =
        |name| Controller::new(Document::with_template_tolerance(name), Default::default());
    let mut source = make_document("Clipboard source")?;
    source.submit("Box 4 5 6")?;
    source.submit("Point")?;
    let position = [2., 3., 4.];
    source.accept_world_point(position)?;
    let source_ids: Vec<_> = source.document().entities.keys().copied().collect();
    if source_ids.len() != 2 {
        return Err(Error::Invalid("clipboard smoke source geometry"));
    }
    source.select_objects(&source_ids)?;
    let source_undo = source.coordinator().undo_len();
    let board = system_clipboard::Pasteboard::isolated();
    let apply = |controller: &mut Controller, action| {
        apply_clipboard_action(
            controller,
            action,
            |format, bytes| board.publish(format, bytes),
            || board.read(),
        )
    };
    for (action, expected_format) in [
        (ClipboardAction::Copy, Format::Om9),
        (ClipboardAction::Copy3dm, Format::Rhino3dm),
    ] {
        apply(&mut source, action)
            .map_err(|error| Error::Io(format!("macOS clipboard {action:?}: {error}")))?;
        let (format, _, _) = board.read().map_err(Error::Io)?;
        if format != expected_format
            || source.coordinator().undo_len() != source_undo
            || source.document().entities.len() != source_ids.len()
        {
            return Err(Error::Invalid("clipboard smoke Copy output"));
        }
        let mut target = make_document("Clipboard destination")?;
        apply(&mut target, ClipboardAction::Paste)
            .map_err(|error| Error::Io(format!("macOS clipboard Paste after {action:?}: {error}")))?;
        let document = target.document();
        if document.entities.len() != 2
            || document.entities.keys().any(|id| source_ids.contains(id))
            || !document
                .entities
                .values()
                .any(|entity| matches!(entity.geometry, Geometry::Point(p) if p == position))
            || !document
                .entities
                .values()
                .any(|entity| matches!(entity.geometry, Geometry::Brep(_)))
            || target.coordinator().undo_len() != 1
        {
            return Err(Error::Invalid("clipboard smoke pasted geometry or Undo"));
        }
        target.undo()?;
        if !target.document().entities.is_empty() {
            return Err(Error::Invalid(
                "clipboard smoke Undo must remove the whole paste",
            ));
        }
        target.redo()?;
        if target.document().entities.len() != 2 {
            return Err(Error::Invalid(
                "clipboard smoke Redo must restore the whole paste",
            ));
        }
        println!("macOS clipboard smoke passed: {action:?}, Paste, Undo and Redo");
    }
    Ok(())
}

#[cfg(all(test, any(windows, target_os = "macos")))]
mod tests {
    use super::*;
    #[test]
    fn ordinary_text_paste_is_not_intercepted_by_model_clipboard() {
        assert!(!routes_to_model(Shortcut::Paste, false, false));
        assert!(!routes_to_model(Shortcut::Paste, true, false));
        assert!(routes_to_model(Shortcut::Paste, false, true));
        assert!(!routes_to_model(Shortcut::Copy, false, true));
        assert!(routes_to_model(Shortcut::Copy3dm, true, false));
    }
}
