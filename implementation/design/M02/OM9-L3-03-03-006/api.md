# OM9-L3-03-03-006 — Matrix Gumball Numeric Edit

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/session/gumball_numeric.rs`, `crates/om9-app/src/session.rs`, `crates/om9-core/src/gumball.rs`
- **Positive oracle:** Nhập 12.5 mm vào handle X tạo transform chính xác và một undo entry
- **Negative oracle:** String NaN và trống không commit; parse fail giữ gizmo active
- **Verification:** om9-app gumball_numeric: 6 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
