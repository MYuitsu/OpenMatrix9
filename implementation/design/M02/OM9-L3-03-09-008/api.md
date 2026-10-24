# OM9-L3-03-09-008 — F6 Disabled Unimplemented Actions

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-09` (M02)
- **Sources:** `crates/om9-app/src/f6.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Unavailable command hiển thị disabled cùng lý do; click không dispatch
- **Negative oracle:** Click nút disabled không gọi M01 command engine và không mutate doc
- **Verification:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
