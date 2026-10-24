# OM9-L3-03-06-003 — Reference Image Transparency And Lock

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-06` (M02)
- **Sources:** `crates/om9-app/src/pictureframe.rs`
- **Positive oracle:** Opacity 50%, locked prevents edits/selection nhưng vẫn visible
- **Negative oracle:** Unlock/hidden flags không làm hỏng dữ liệu asset hoặc lock state
- **Verification:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
