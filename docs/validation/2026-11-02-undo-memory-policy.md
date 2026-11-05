# Undo retention by memory

Follow-up: [50% physical-RAM ceiling, 100 MB inventory cadence and viewport/export plan](2026-11-02-undo-ram-cap-and-inventory.md). The checks and executable fingerprint below describe the initial patch.

Implemented at the user's request on 2026-11-02, following their Rhino 5 General options screenshot. Defaults: minimum 1 Undo step and maximum attempted history storage 1024 MB (1 MB = 1,048,576 bytes). This is a soft retention policy, not a reservation of system RAM.

## Behavior and ownership

- `om9-core::transaction::UndoPolicy` configures retention at the common publication boundary. Successful recorded edits prune the oldest Undo entries only while tracked memory exceeds the budget and more than the minimum remain. There is no 256-step product cap; other preexisting admission limits still apply. A minimum larger than the available history does not manufacture entries.
- An explicit policy update trims immediately without changing document geometry, associative recipes, publication epoch or the valid Redo branch. Policy changes are refused during an active editing scope/command. Failed transactions, canceled preparations and empty/no-op commits do not prune history. New/Open/Master replacements retain this application preference through the shared controller adoption path.
- Inventory estimates the journal vectors, entry metadata and distinct immutable document owners. It deduplicates document Arcs and embedded resource payloads and excludes owners/payloads already required by live documents or editing scopes. It does not measure process RSS, GPU VRAM, allocator overhead or memory held by independent external snapshots. Dropping journal references does not force an externally retained owner to free its buffers.
- Existing hard admission limits are unchanged. In particular the default aggregate byte limit is 256 MB unless explicitly configured otherwise; it can reject an allocation before a 1024 MB soft Undo budget is reached. This change does not promise a 1 GB allocation or remove application safety limits. Keeping the minimum can leave history above the soft budget.

## UI and persistence

The existing `RhinoOptions` icon in Info & Settings and the Standard toolbar opens **Options — General** with **Min number of undos** and **Max memory used (MB)**. Typed `Options`, `RhinoOptions` and `UndoSettings` open the same dialog. OK applies and persists the policy, Cancel/window close leaves the draft unapplied. Settings are application scoped in `undo.json` beside the interactive layout store; missing files use defaults and malformed/oversized files report errors without blocking startup. No document transaction is created by the settings dialog.

## Verification

Built and tested default features on Windows with `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`.

- `om9-core --test undo_memory`: 5 passed. Covers eviction/contiguous Undo-Redo, minimum-over-budget, unchanged document/epoch/Redo and failure atomicity, 260 retained steps under budget, invalid preferences, and shared embedded payload ownership.
- `om9-core --test atomic_transactions`: 17 passed; `transaction_scopes`: 9 passed; `resource_transactions`: 12 passed.
- `om9-app --lib undo_settings::tests`: 3 passed. Simulated actual icon and OK-button clicks, preference persistence/corruption, typed submission routing, unchanged document snapshot and application preference retention across replacement. These are headless egui checks, not physical mouse evidence.
- Default `cargo build -p om9-app --bin om9 -j 2` could not replace the in-use `L:/om9-target/debug/om9.exe`. The active process was left untouched.
- `cargo rustc -p om9-app --bin om9 -j 2 -- --emit=link=L:/om9-target/debug/om9-undo.exe` succeeded. Separate debug executable SHA-256: `e848d5a2c50e8bfaa30e1c1d9cc57c9e89ee206f331befba3fe97985afc125c5`. Its isolated `--mcp` initialization passed; no live window/document was controlled. No release performance or desktop appearance claim.

Business-rule text has not been added to reusable skills; the project requires separate consent for that recording step.
