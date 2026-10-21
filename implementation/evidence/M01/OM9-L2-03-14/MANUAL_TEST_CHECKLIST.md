# OM9-L2-03-14 — Checklist kiểm thử thủ công (gộp 8 L3)

Sinh tự động từ 8 tệp `*.ui-review.json` (nguồn duy nhất). **35 mục**, 8/8 ui-review chưa ký.
Mỗi mục: *máy đã chứng minh* là phần test tự động khẳng định; *bạn xác nhận* là phần cần mắt người.

Ký bằng cách đặt `reviewer.status = "SIGNED"` + `reviewer.name`/`date` trong tệp ui-review tương ứng
(`python tools/sign_m01_ui_review.py` nếu có), rồi chạy `python tools/om9_milestone_dashboard.py`.

## Đã được máy chứng minh (chỉ cần xác nhận bằng mắt)

- **Super Select qua menu Tools** — `tests/super_select_ui.rs`: chỉ object mang `om9.gem` được chọn, status `"Super Select: Gems: 1 selected"`.
- **Property editor qua menu Edit** — `tests/property_menu_ui.rs`: mỗi toggle là **một** bước Undo.
- **Nhãn Object Info** — ảnh cửa sổ thật: `implementation/evidence/M01/OM9-L3-03-14-005/object-info-label-in-a-real-window.png`.

## OM9-L3-03-14-001 — Nhóm bảng màu và nhãn loại

`OM9-L4-03-14-001-05.ui-review.json` · **NOT_SIGNED**

- [ ] **V1** — Open the LAYERS panel and read the rows.
  - *máy đã chứng minh*: P1/P5 and the real-window receipt: the palette exposes Metals, Gems, User, Heads, Finger, Cutting, Creation in that order.
  - *bạn xác nhận*: All seven names are readable in the panel without covering the layer name, the colour swatch, the lock or the visibility control.
- [ ] **V2** — Compare the colours with Matrix8 book 1 p54-57 (reference in the milestone).
  - *máy đã chứng minh*: P3/P4: slots 1-4 Metals, 5-8 Gems, 9-12 User, 13 Heads, 14 Finger, 15 Cutting, 16 Creation; 17-32 are extra User colours.
  - *bạn xác nhận*: The metals/gems/user/builders colours match the manual closely enough to model with.
- [ ] **V3** — Click the four-row block toggles and a row's lock/visibility.
  - *máy đã chứng minh*: the existing layer tests (left_stack_layers, layer_widgets) and the recipe paths are unchanged by this item.
  - *bạn xác nhận*: The blocks still toggle together, and the new category text did not move or shrink the controls.
- [ ] **V4** — Rename a custom layer (ADVANCED, slots 17-32) to something else and look at it again.
  - *máy đã chứng minh*: N1/N2/N3: the row keeps its own name, the category stays User, and the slot number is the identity; a reserved label is refused.
  - *bạn xác nhận*: The renamed row shows its own text and still says User, and it is still the same layer after save/reopen (the persistence check is L3-008).
- [ ] **V5** — Look at the Lights row (slot 33) and the ADVANCED rows.
  - *máy đã chứng minh*: P3/N6: slot 33 is labelled Lights and is not one of the seven categories; 17-32 are User.
  - *bạn xác nhận*: Lights reads as its own row, not as a colour category, and the advanced rows read User.

## OM9-L3-03-14-002 — Gán layer, layer hiện hành và luật layer khoá

`OM9-L4-03-14-002-05.ui-review.json` · **NOT_SIGNED**

- [ ] **W1** — Click a Metals row to make it current.
  - *máy đã chứng minh*: C1/C3: the preset layer is created and made current in one call, and the panel's current id equals document.current_layer.
  - *bạn xác nhận*: The clicked row is highlighted, the status line names it, and clicking another row moves the highlight.
- [ ] **W2** — Select an object and assign it to another layer (e.g. Gems).
  - *máy đã chứng minh*: C2: one transaction moves the objects, and the current layer does not move with them.
  - *bạn xác nhận*: The object's colour/row changes to the target layer, Undo reverses it in one step, and the current layer stays where it was.
- [ ] **W3** — Lock Gems, then try to assign an object to it.
  - *máy đã chứng minh*: L1/L2: the assignment is refused before anything is applied, with 'layer is locked', and the journal does not move.
  - *bạn xác nhận*: Nothing moves, the status explains the refusal, and Undo is not needed to get back.
- [ ] **W4** — Assign with nothing selected.
  - *máy đã chứng minh*: L3: it is a no-op that succeeds and adds no journal entry.
  - *bạn xác nhận*: No error dialog, no phantom undo entry.
- [ ] **W5** — Create a layer through the CMD line (Layer 5 / Layer Gems) and look at the palette.
  - *máy đã chứng minh*: the CMD path resolves a slot number, a preset name and a quoted label to one slot, refusing an ambiguous quoted label.
  - *bạn xác nhận*: The new row appears in the right group with the manual's colour, and a repeated name is refused rather than guessed.

## OM9-L3-03-14-003 — Ẩn/hiện và khoá layer

`OM9-L4-03-14-003-05.ui-review.json` · **NOT_SIGNED**

- [ ] **X1** — Hide the Gems row.
  - *máy đã chứng minh*: S1/S2: its objects become non-visible and non-selectable at once and select_layer selects nothing.
  - *bạn xác nhận*: The objects disappear from the viewport immediately and cannot be clicked.
- [ ] **X2** — Show it again.
  - *máy đã chứng minh*: S3: visibility, selectability and the scene entry come back in the same call.
  - *bạn xác nhận*: The objects reappear in their layer colour and can be picked.
- [ ] **X3** — Lock Gems and try to move one of its objects to another layer.
  - *máy đã chứng minh*: T1/T2: refused before anything is applied, with 'locked' in the message; entities and journal unchanged.
  - *bạn xác nhận*: Nothing moves, the status names the lock, and Undo is not needed.
- [ ] **X4** — Look at a locked layer's rows.
  - *máy đã chứng minh*: T4: display attributes stay complete (name, layer, flags).
  - *bạn xác nhận*: A locked layer still shows its objects and colours; only editing is refused.

## OM9-L3-03-14-004 — Chọn theo layer

`OM9-L4-03-14-004-05.ui-review.json` · **NOT_SIGNED**

- [ ] **Y1** — Right-click the Gems swatch with two objects on it.
  - *máy đã chứng minh*: A1/A2: exactly the two Gems objects are selected and a second gesture replaces the set.
  - *bạn xác nhận*: Both objects highlight; another layer's object never joins the selection.
- [ ] **Y2** — Hide one object of the layer, then use the gesture again.
  - *máy đã chứng minh*: A3: the hidden object stays out.
  - *bạn xác nhận*: Only the visible object is selected and no hidden object is silently included.
- [ ] **Y3** — Lock the layer and use the gesture.
  - *máy đã chứng minh*: B1/B3: nothing is selected and the previous selection is cleared, not kept.
  - *bạn xác nhận*: The selection empties instead of leaving a stale highlight, and no error dialog appears.
- [ ] **Y4** — Use the group gesture (the block toggle) and Undo afterwards.
  - *máy đã chứng minh*: A4/B4: the group gesture obeys the same policy and the gesture writes no transaction.
  - *bạn xác nhận*: Group selection matches row selection, and Undo has nothing to undo from selecting.

## OM9-L3-03-14-005 — Object Info

`OM9-L4-03-14-005-05.ui-review.json` · **NOT_SIGNED**

- [ ] **Z1** — Check the fields the overlay will show for a point and a line (type, size, validity, length).
  - *máy đã chứng minh*: I1/I2: kind, bounding size and validity come from the model; a line reports its length.
  - *bạn xác nhận*: The wording and units match what you expect the label to read.
- [ ] **Z2** — Check the rule for geometry without a measurement (mesh, native B-rep without a payload).
  - *máy đã chứng minh*: I3/I4/J1: the size is Unavailable with a reason and never 0,0,0.
  - *bạn xác nhận*: You agree the overlay should say 'unavailable' rather than showing a zero.
- [ ] **Z3** — Check the colour: the label is drawn in the object's layer colour.
  - *máy đã chứng minh*: I5: the info carries the colour when the object/layer has one.
  - *bạn xác nhận*: The colour you expect is the colour you would see on the label.
- [ ] **Z4** — Select ONE object and look at the label beside it.
  - *máy đã chứng minh*: the label text is unit-tested and the receipt carries the same line; the label is painted at the projected object centre in the layer colour.
  - *bạn xác nhận*: The label sits beside the object without covering it, reads type · size · validity, and disappears when nothing (or more than one object) is selected.

## OM9-L3-03-14-006 — Super Select theo loại đối tượng

`OM9-L4-03-14-006-05.ui-review.json` · **NOT_SIGNED**

- [ ] **U1** — Open Tools → Super Select: Gems on a model that has one semantic gem and one mesh named 'Gem'.
  - *máy đã chứng minh*: tests/super_select_ui.rs: the menu item selects the gem and nothing else (real frame + click).
  - *bạn xác nhận*: The lookalike mesh is not selected, even though its name says Gem; the status line reads 'Super Select: Gems: 1 selected'.
- [ ] **U2** — Run the bad-geometry filter on a clean model.
  - *máy đã chứng minh*: G3: it matches only geometry the model's own validate rejects.
  - *bạn xác nhận*: A clean model selects nothing and the status says why.
- [ ] **U3** — Run the open-surface filter.
  - *máy đã chứng minh*: G4: every object is skipped with NoOpenVerdict — the verdict is not invented.
  - *bạn xác nhận*: You agree that showing 'no verdict yet' is better than guessing, and confirm when the kernel verdict lands (it is a later item).
- [ ] **U4** — Run the duplicate filter on repeated labels.
  - *máy đã chứng minh*: G5: the repeated labels match and the first occurrence keeps its claim.
  - *bạn xác nhận*: The duplicates you expect are the ones selected.
- [ ] **U5** — Undo after a SuperSelect gesture.
  - *máy đã chứng minh*: H5: the gesture writes no transaction.
  - *bạn xác nhận*: Undo has nothing to undo from selecting.

## OM9-L3-03-14-007 — Property editor + rename

`OM9-L4-03-14-007-05.ui-review.json` · **NOT_SIGNED**

- [ ] **N1** — Change the colour of a selected object.
  - *máy đã chứng minh*: E1: one journal entry, and Undo restores the previous colour.
  - *bạn xác nhận*: The object changes colour at once and one Undo returns the old colour.
- [ ] **N2** — Hide the object through the editor, then Undo.
  - *máy đã chứng minh*: E2: same one-command rule.
  - *bạn xác nhận*: One Undo brings the object back.
- [ ] **N3** — Look at the field list: which are editable, which are greyed out and why.
  - *máy đã chứng minh*: E3/F2: six editable fields (Name, Color, Visible, Hidden, Selectable, Locked); Geometry is disabled with its reason.
  - *bạn xác nhận*: Rename a row and confirm one Undo brings the old name back; confirm Geometry cannot be typed into.
- [ ] **N4** — Try to edit an object whose layer is locked.
  - *máy đã chứng minh*: F3/F4: refused with Permission, no journal entry, value unchanged.
  - *bạn xác nhận*: The editor explains the refusal instead of silently doing nothing.

## OM9-L3-03-14-008 — Lưu/khôi phục trạng thái layer

`OM9-L4-03-14-008-05.ui-review.json` · **NOT_SIGNED**

- [ ] **P1** — Lock Gems, hide Metals, Save, close and reopen the file.
  - *máy đã chứng minh*: K1: both groups' slots come back with locked/visible and the slot colours.
  - *bạn xác nhận*: The palette shows the same lock and visibility icons as before saving.
- [ ] **P2** — After the reopen, click the locked Gems row and the hidden Metals row.
  - *máy đã chứng minh*: K3: nothing is selected from either row, and the object ids are the same as before saving.
  - *bạn xác nhận*: No objects jump into the selection, and the ids/colours you had are still there.
- [ ] **P3** — Open a different file in the same session.
  - *máy đã chứng minh*: L1: the previous slots are absent and select nothing.
  - *bạn xác nhận*: The palette shows the new file's layers only — no leftover rows.
- [ ] **P4** — Try to open a file that does not exist.
  - *máy đã chứng minh*: L2: the current document and its flags are untouched.
  - *bạn xác nhận*: The palette keeps the current state and the error is explained.\n
## Cách ký (một lệnh)

```powershell
python tools\sign_m01_l2_03_14.py --reviewer "Tên của bạn" --dry-run   # xem trước
python tools\sign_m01_l2_03_14.py --reviewer "Tên của bạn"             # ký cả 8
python tools\om9_milestone_dashboard.py                                # 8 L3 chuyển DONE
```

Công cụ **từ chối** ký một review không có mục thủ công hoặc không có bằng chứng máy,
nên chữ ký không thể che một hồ sơ thiếu.
