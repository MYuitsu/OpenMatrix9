// SPDX-License-Identifier: LGPL-2.1-or-later
//! Clipboard command routing. The native adapter owns Windows clipboard handles.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClipboardAction {
    Copy,
    Copy3dm,
    Paste,
}

#[cfg(windows)]
fn routes_to_model(
    event: om9_native_owner::clipboard::Shortcut,
    selected: bool,
    has_geometry: bool,
) -> bool {
    use om9_native_owner::clipboard::Shortcut;
    match event {
        Shortcut::Copy | Shortcut::Copy3dm => selected,
        Shortcut::Paste => has_geometry,
    }
}

impl MatrixApp {
    pub(super) fn clipboard_action(&mut self, action: ClipboardAction) {
        #[cfg(windows)]
        let result: Result<()> = (|| {
            use om9_native_owner::clipboard::{self, Format};
            match action {
                ClipboardAction::Copy => {
                    let bytes = self.controller.copy_selected_om9()?;
                    clipboard::publish(Format::Om9, &bytes).map_err(Error::Io)?;
                }
                ClipboardAction::Copy3dm => {
                    let bytes = self.controller.copy_selected_3dm()?;
                    clipboard::publish(Format::Rhino3dm, &bytes).map_err(Error::Io)?;
                }
                ClipboardAction::Paste => {
                    let (format, bytes, metadata) = clipboard::read().map_err(Error::Io)?;
                    match format {
                        Format::Om9 => {
                            self.controller.paste_om9(&bytes)?;
                        }
                        Format::Rhino3dm => {
                            self.controller.paste_3dm(&bytes, metadata.as_deref())?;
                        }
                    }
                }
            }
            Ok(())
        })();
        #[cfg(not(windows))]
        let result: Result<()> = Err(Error::Unsupported("geometry clipboard requires Windows"));
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

    pub(super) fn attach_clipboard_shortcuts(&mut self, frame: &eframe::Frame) {
        #[cfg(windows)]
        {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};
            if self.clipboard_hook_attempted {
                return;
            }
            self.clipboard_hook_attempted = true;
            let hook =
                frame
                    .window_handle()
                    .map_err(|error| error.to_string())
                    .and_then(|handle| match handle.as_raw() {
                        RawWindowHandle::Win32(handle) => {
                            om9_native_owner::clipboard::ShortcutHook::attach(
                                handle.hwnd.get() as usize
                            )
                        }
                        _ => Err("No Windows clipboard shortcut window".into()),
                    });
            match hook {
                Ok(hook) => self.clipboard_shortcut_hook = Some(hook),
                Err(error) => {
                    self.status.error(&error);
                }
            }
        }
        #[cfg(not(windows))]
        let _ = frame;
    }

    pub(super) fn clipboard_shortcuts_from_input(
        &mut self,
        ctx: &egui::Context,
        raw: &mut egui::RawInput,
    ) {
        #[cfg(windows)]
        {
            use om9_native_owner::clipboard::Shortcut;
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
        #[cfg(not(windows))]
        let _ = (ctx, raw);
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use om9_native_owner::clipboard::Shortcut;
    #[test]
    fn ordinary_text_paste_is_not_intercepted_by_model_clipboard() {
        assert!(!routes_to_model(Shortcut::Paste, false, false));
        assert!(!routes_to_model(Shortcut::Paste, true, false));
        assert!(routes_to_model(Shortcut::Paste, false, true));
        assert!(!routes_to_model(Shortcut::Copy, false, true));
        assert!(routes_to_model(Shortcut::Copy3dm, true, false));
    }
}
