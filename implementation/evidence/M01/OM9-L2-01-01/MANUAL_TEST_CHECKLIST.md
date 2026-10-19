# OM9-L2-01-01 (Febe Application Shell Docking) — combined manual test checklist

One pass over the five L3 items finished on 2026-10-16 (OM9-L3-01-01-004 … -008).
Machine evidence for every item is in `implementation/evidence/M01/<l3-id>/`; the
real-window screenshots are in `implementation/evidence/M01/OM9-L3-01-01-003/screenshots/`
(seven panel scenarios) and `implementation/evidence/M01/OM9-L3-01-01-008/builders-in-a-real-window.png`.

## How to run

```powershell
cd H:\OpenMatrix9\standalone
cargo run -p om9-app --bin om9          # the app for items T/R/S/V below
# machine checks (all of them must pass):
cargo test --workspace -j 2             # -j 2: the default parallel build exhausts this host's paging file
cargo test -p om9-app --test dock_layout_default --test dock_layout_persistence \
  --test document_window_state --test global_status_surface --test builder_slot
python tools\validate_m01_l3_003_panels.py     # 7 real-window panel scenarios
python tools\validate_m01_l3_002_ui_matrix.py --mode all   # 14 real-window sizes
```

## Sign-off

| L3 | Title | Artefacts | Machine checks | My test |
| --- | --- | --- | --- | --- |
| `OM9-L3-01-01-004` | Docking Layout Default Restore | 4/4 | PASS (see evidence) | ⬜ |
| `OM9-L3-01-01-005` | Docking Layout Persistence | 4/4 | PASS (see evidence) | ⬜ |
| `OM9-L3-01-01-006` | Active Document Window State | 4/4 | PASS (see evidence) | ⬜ |
| `OM9-L3-01-01-007` | Global Status And Error Surface | 5/5 | PASS (see evidence) | ⬜ |
| `OM9-L3-01-01-008` | Builder Panel Docking Slot | 5/5 | PASS (see evidence) | ⬜ |

Write `OK` or the deviation in the last column; the same answer signs the item's
`OM9-L4-01-01-0xx-05.ui-review.json` and moves it to ✅ on the dashboard.

## Real-window coverage, per item

| L3 | Real-window evidence |
| --- | --- |
| `OM9-L3-01-01-004` | shared: every real-window harness run loads the layout store and the L3-002 matrix re-checks 14 sizes on the same build |
| `OM9-L3-01-01-005` | shared: the L3-003 panel scenarios run `Set as Default`? no - the store is read by every run; the write path is covered by the frame tests in tests/dock_layout_persistence.rs |
| `OM9-L3-01-01-006` | shared: the title and the close decision are covered by frame tests on the same build; no dedicated scripted window scenario |
| `OM9-L3-01-01-007` | dedicated: om9.exe --ui-audit receipt carries status_kind/status_message/status_tail (OM9-L4-01-01-007-04.integration.log) |
| `OM9-L3-01-01-008` | dedicated: builders-in-a-real-window.png plus the left_sections/builder_slot_visible fields in the receipt (OM9-L4-01-01-008-04.integration.log) |

## OM9-L3-01-01-004 — Docking Layout Default Restore (Reset/reload)

Oracle: *Reset custom layout recreates Matrix-oriented baseline positions* / negative: *Unknown layout version does not crash, resets to default*

Evidence: `implementation/evidence/M01/OM9-L3-01-01-004/OM9-L4-01-01-004-05.ui-review.json`

### R1. Move, collapse, dock and float a few menus any way you like, then run View ▸ Matrix Menus ▸ Reset Menus.

- Machine: reset_to_default is the documented baseline byte for byte, from every customisation kind (D1, D2); an active drag is cancelled.
- You confirm: Every menu is back in the documented order and size, nothing floats, the right column is hidden again, and the status line reads 'Menus reset to the Matrix default'.
- Result: ⬜ OK / ⬜ deviation: …

### R2. Drag the left rail wider (about 420), then Reset Menus.

- Machine: the baseline size of every panel is the documented number and reset restores it (D1).
- You confirm: The rail returns to its documented width and the icons go back to their documented edge.
- Result: ⬜ OK / ⬜ deviation: …

### R3. Arrange the menus, run Set as Default, change the arrangement again, then Reload Menus. Repeat in a session that has never run Set as Default.

- Machine: reload without a store is the factory default (D4) with the status 'Menus reloaded from the factory default'; a kept configuration reloads without a schema diagnostic.
- You confirm: Reload brings back the configuration you kept, and a session without one says it used the factory default.
- Result: ⬜ OK / ⬜ deviation: …

### R4. Put a layout file with schema_version 1 at the layout path and start the app (see the OM9-L3-01-01-005 item for the exact path once it is recorded).

- Machine: N1/N2/N3/N6: any other version returns the baseline, names the version in the diagnostic, and never half-applies the document.
- You confirm: The app starts with the Matrix default, the status/history names the refused version, and nothing from the old file leaks in.
- Result: ⬜ OK / ⬜ deviation: …

### R5. Put a layout file with broken bytes, and one with an impossible stored home, at the layout path and start the app.

- Machine: N4/N5/N7: unreadable, empty or oversized bytes are refused; N9: an impossible home is repaired to the documented position with a diagnostic.
- You confirm: The app starts with the baseline, reports the ignored layout or the repaired home, and never crashes or hangs.
- Result: ⬜ OK / ⬜ deviation: …

Status of this item's checklist: PENDING HUMAN VERIFICATION

## OM9-L3-01-01-005 — Docking Layout Persistence (Remember the arrangement)

Oracle: *Persist dock positions and reopen same named panel IDs* / negative: *Removed/unrecognized panel ID ignored with warning not crash*

Evidence: `implementation/evidence/M01/OM9-L3-01-01-005/OM9-L4-01-01-005-05.ui-review.json`

### P1. Arrange the menus (dock one right, roll one up, reorder, float one, hide one, resize the rail), run View ▸ Matrix Menus ▸ Set as Default, close the app, start it again.

- Machine: S1/S2/S3: the written file reopens byte for byte, with the same named ids, orders, roll-up states, floating rectangle, height, rail width and right column.
- You confirm: The window opens exactly as you left it: same order, same sizes, the floating menu back in the same place, the hidden menu still hidden.
- Result: ⬜ OK / ⬜ deviation: …

### P2. Arrange the menus, change them again without running Set as Default, then Reload Menus. Repeat with Set as Default in between.

- Machine: the store is the only writer; reload without a kept file is the documented baseline (S4).
- You confirm: Reload brings back the kept configuration; changes you did not keep are discarded; the status line says which of the two happened.
- Result: ⬜ OK / ⬜ deviation: …

### P3. Put a layout file that names a panel the program no longer has (for example key right-dock.project-manager) at the layout path and start the app.

- Machine: M1/M2: an unknown panel or menu id is removed with the warning "... is not recognized; removed", and the rest of the file still loads.
- You confirm: The app starts, the finished arrangement is kept for every name it still knows, and the status/history names the id it dropped.
- Result: ⬜ OK / ⬜ deviation: …

### P4. Put a layout file with an absurd value in it (rail width 5000, a zero-sized floating rectangle, two menus with the same order) at the layout path and start the app.

- Machine: M4/M5/M8: each repair is clamped or renumbered and reported with a warning; nothing is silently changed.
- You confirm: The app starts with a usable arrangement and the status/history lists what it repaired.
- Result: ⬜ OK / ⬜ deviation: …

### P5. Make the layout file unreadable (garbage bytes, or delete it) and start the app.

- Machine: M10/M11/M12: unreadable, empty and oversized files are refused; a missing file is a clean baseline with no warning (S4).
- You confirm: The app always starts: garbage is reported and ignored, a missing file is silent, and no menu is lost.
- Result: ⬜ OK / ⬜ deviation: …

Status of this item's checklist: PENDING HUMAN VERIFICATION

## OM9-L3-01-01-006 — Active Document Window State (Title and unsaved work)

Oracle: *Switch active document changes title and unsaved indicator without stale UI refs* / negative: *Close document with unsaved changes requires explicit decision*

Evidence: `implementation/evidence/M01/OM9-L3-01-01-006/OM9-L4-01-01-006-05.ui-review.json`

### T1. Start the app without a document and read the title bar.

- Machine: W1: the title is 'OpenMatrix9 — Untitled' and the document is clean.
- You confirm: The title bar really shows that text (never empty) and no ' *' is present.
- Result: ⬜ OK / ⬜ deviation: …

### T2. Draw something, watch the title, then Save (or Save As) and watch it again.

- Machine: W2/W3/W4: the marker appears while the document differs from its file, disappears after the save, and the file name replaces the document name.
- You confirm: The title bar updates at once, names the file and drops the marker after saving.
- Result: ⬜ OK / ⬜ deviation: …

### T3. Open another document while the current one has unsaved changes, choose Discard, then look at the title, the layers list and the undo state.

- Machine: W5: the title, the document name, the entity count and the undo journal all belong to the newly opened document; nothing of the old one is left.
- You confirm: The window really shows the opened file everywhere (title, layer list, objects) and Undo has nothing from the previous document.
- Result: ⬜ OK / ⬜ deviation: …

### T4. Move, roll up, dock and float menus, then look at the title.

- Machine: W6: panel placement never makes the document dirty.
- You confirm: The title keeps no ' *' after rearranging menus, and running a file command afterwards does not ask to save.
- Result: ⬜ OK / ⬜ deviation: …

### T5. With unsaved changes, close the window and answer Cancel, then Discard, then (with unsaved changes again) Save.

- Machine: C1..C9: a dirty close asks first; Cancel leaves everything alone; Discard closes without writing; Save writes and then closes; a clean close never asks; a stale answer cannot close or save anything.
- You confirm: The prompt is clear, Cancel really keeps the app open with the work intact, Discard does not write, and Save writes before closing.
- Result: ⬜ OK / ⬜ deviation: …

Status of this item's checklist: PENDING HUMAN VERIFICATION

## OM9-L3-01-01-007 — Global Status And Error Surface (Status and errors)

Oracle: *Invalid command/OCCT diagnostic visible in status, not lost in console* / negative: *Error popup cannot swallow escape/cancel state or dismiss silently*

Evidence: `implementation/evidence/M01/OM9-L3-01-01-007/OM9-L4-01-01-007-05.ui-review.json`

### S1. Type a command the program does not have (for example NotACommand) and press Enter.

- Machine: G1: the line reads 'Error: unknown command: NotACommand', the same text is in the history, and the tail carries it.
- You confirm: The red-ish error line names the command you typed, and you can find the same line by scrolling the command history.
- Result: ⬜ OK / ⬜ deviation: …

### S2. After that failure, run a command that works (for example Line).

- Machine: G2: the failure stays; complete_success refuses while a failure is unacknowledged.
- You confirm: The error line is still there after the successful command, and Dismiss is the only thing that clears it.
- Result: ⬜ OK / ⬜ deviation: …

### S3. Hover the status line after a few messages have gone by.

- Machine: G9/G10: the tail keeps the last eight messages, oldest first, and a second failure keeps the first.
- You confirm: The hover lists the recent messages, so a warning you missed is still readable.
- Result: ⬜ OK / ⬜ deviation: …

### S4. With a failure on the line, start a command (for example Line) and press Escape.

- Machine: X1/X2: Escape still cancels the command and does not dismiss the failure.
- You confirm: Escape cancels the command, the failure stays, and no dialog appears that traps the keyboard.
- Result: ⬜ OK / ⬜ deviation: …

### S5. Repeat the invalid command with a very long line, and with a warning-producing action (for example a floating menu that has to be pulled back inside).

- Machine: G5/G7: a warning is visible without becoming a failure; a long diagnostic is cut at 512 bytes on a character boundary.
- You confirm: Nothing overflows the status bar, the text stays readable and the summary on the right is still elided sensibly.
- Result: ⬜ OK / ⬜ deviation: …

Status of this item's checklist: PENDING HUMAN VERIFICATION

## OM9-L3-01-01-008 — Builder Panel Docking Slot (Reserved Builders area)

Oracle: *Reserve Builder docking slot below Projects without creating a second Builder model* / negative: *Absent Builder implementation shows noninteractive placeholder*

Evidence: `implementation/evidence/M01/OM9-L3-01-01-008/OM9-L4-01-01-008-05.ui-review.json`

### V1. Look at the left column: the BUILDERS area is below PROJECTS.

- Machine: B1: the reserved slot is in the left column at the documented last position, visible, never floating; the real-window receipt shows it last.
- You confirm: The reserved area really sits under the Project Manager and stays there after resizing.
- Result: ⬜ OK / ⬜ deviation: …

### V2. Click the BUILDERS bar and try to drag it.

- Machine: Y1/B4: the click is refused with the documented message, the bar has no +/− box and no drag starts.
- You confirm: Nothing moves, the message explains that Builders are reserved, and the cursor gives no hint that it could be dragged.
- Result: ⬜ OK / ⬜ deviation: …

### V3. Try the placeholder's controls (New Emerald, Size, Show Diag., Reset, the slider, the icons).

- Machine: B5/Y2: every widget in builder_slot_ui is disabled or hover-only (source scan), and the gem box senses hover only.
- You confirm: None of them reacts, no popup opens, and the hover text says the operation is unavailable.
- Result: ⬜ OK / ⬜ deviation: …

### V4. Draw geometry, then look at the BUILDERS area and the document.

- Machine: B3/Y3: the placeholder is drawn from the layout only (no Document/Controller/builder type in its body); the slot creates no entities and no undo entries.
- You confirm: There is no second Builder model: the object count, the layer list and Undo show only your drawing.
- Result: ⬜ OK / ⬜ deviation: …

### V5. Edit a saved layout file so builder-slot is floating, hidden or above Projects, then start the app.

- Machine: B6/Y4: the reservation is repaired back to the documented slot with a diagnostic naming 'Builders is reserved'.
- You confirm: The app starts with the reserved area back below Projects and the status/history names the repair.
- Result: ⬜ OK / ⬜ deviation: …

Status of this item's checklist: PENDING HUMAN VERIFICATION

## Totals

- Items above: **25** across five L3.
- Machine checks run on 2026-10-16: every group of `cargo test --workspace -j 2` ok, `cargo check --workspace --all-targets` exit 0, `clippy -D warnings` exit 0, `fmt --check` exit 0, L3-002 matrix 14/14, L3-003 panels 7/7.
- Nothing in `OM9_Milestones` was modified: the plan tree keeps its own statuses.
