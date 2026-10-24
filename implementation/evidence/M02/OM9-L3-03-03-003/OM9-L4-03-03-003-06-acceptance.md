# OM9-L4-03-03-003-06 — Matrix Ambiguous Selection Popup acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Nhiều hits hiện popup type+layer swatch; chọn ID cụ thể và đóng popup
- **Negative oracle:** Esc popup không thay selection, stale ID lọc bỏ trước khi chọn
- **Acceptance:** Overlapping objects show distinct type+swatch rows; Escape keeps the prior selection and a stale candidate is refused before selection.
- **Evidence:** om9-app pick_disambiguation: 5 passed; om9-app matrix_selection_sequence: 2 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
