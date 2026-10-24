# OM9-L3-02-03-006 — Smarttrack Inferred Alignment Lines

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Hai end snap khác nhau sinh SmartTrack alignment tạm thời qua điểm tham chiếu
- **Negative oracle:** Không có anchor/đã cancel thì xoá guide, không ghi guide vào document
- **Verification:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
