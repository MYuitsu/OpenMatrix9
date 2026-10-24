# OM9-L4-01-07-004-06 — Selection Window And Crossing acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Cửa sổ kéo trái→phải chỉ chọn box hoàn toàn nằm trong; phải→trái chọn giao cắt
- **Negative oracle:** Object nằm ngoài không được chọn; locked/hidden đều bỏ qua
- **Acceptance:** A window selects only fully contained objects; a crossing selects intersecting ones; outside, hidden and locked objects are not selected.
- **Evidence:** om9-scene selection_regressions: 7 passed; om9-app rubber_band: 2 passed; om9-app acceptance_viewport_selection: 8 passed; om9-app selection_overlay: 8 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
