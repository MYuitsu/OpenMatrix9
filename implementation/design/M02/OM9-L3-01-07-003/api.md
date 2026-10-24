# OM9-L3-01-07-003 — Subobject Face Edge Vertex Picking

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-scene/src/scene.rs`, `crates/om9-scene/src/picking.rs`, `crates/om9-app/src/session/subobjects.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Chọn đúng face/edge/vertex của box và lưu khóa topology bền
- **Negative oracle:** Click vùng trống hoặc face không còn sau rebuild báo miss, không đổi sang mặt gần đó
- **Verification:** om9-scene subobject_pick: 4 passed; om9-app subobject_selection: 8 passed; om9-app subobject_occurrences: 5 passed; om9-app brep_selection (gpu-sphere): 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
