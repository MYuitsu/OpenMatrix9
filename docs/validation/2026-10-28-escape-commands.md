# Escape command cancellation — 2026-10-28

Scope: standalone OM9 command state and egui keyboard input. The shared cancel path now discards Builder, Head and Settings previews; active detection includes those staged commands and Extrude Surface and Heightfield. Escape no longer clears the selection merely because a command is active; ambiguous-pick dismissal can pass Escape to the active command.

Static check: every `*_pending: Option` field in `Controller` is referenced by both `is_running` and `cancel`. This is a field-coverage check, not proof of each command's rollback. `python tools/public_source_audit.py` passed with zero errors.

`cargo test -j 2 -p om9-app --test escape_commands --test ui_events --test command_shortcuts --test pinned_command --test builder_controls --test head_commands --test settings_commands` with `CARGO_TARGET_DIR=L:/om9-target` waited behind existing Cargo jobs and was stopped while waiting; results and real-window validation remain pending. The new keyboard test covers Builder, Head Builder and Prong Builder; it does not establish coverage of every command or stage. File/layer dialogs, IME composition and panel drag retain their existing Escape priority.

Next: run the tests on the final source, fix any failures, then exercise representative menu/CMD commands and preview cancellation in the actual Windows application. Validate other command families and stages before claiming universal coverage.
