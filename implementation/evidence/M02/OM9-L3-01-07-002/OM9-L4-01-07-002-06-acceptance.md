# OM9-L4-01-07-002-06 — Object Selection Set Core acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Click ID A rồi Ctrl click ID B trả set {A,B}, toggle A trả {B}
- **Negative oracle:** ID đã xóa hoặc layer khóa bị loại và không lưu stale ID
- **Acceptance:** ID A followed by Ctrl+ID B yields {A,B}; toggling A leaves {B}; a removed/locked/hidden id is rejected before any mutation and no stale id is stored.
- **Evidence:** om9-app acceptance_selection_set: 6 passed; om9-core selection_state: 7 passed; om9-app selection_overlay: 8 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
