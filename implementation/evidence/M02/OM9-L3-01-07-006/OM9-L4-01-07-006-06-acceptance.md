# OM9-L4-01-07-006-06 — Selection Highlighting And Hover acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Hover cạnh face rồi select box hiển thị màu highlight đồng bộ cả 4 view
- **Negative oracle:** Object hidden hoặc đã xóa không còn outline sau scene refresh
- **Acceptance:** Hover and selection draw a consistent highlight in the shared four-view workspace, and a hidden or deleted object leaves no outline after the scene refresh.
- **Evidence:** om9-app selection_overlay: 8 passed; om9-app subobject_occurrences: 5 passed; om9-app acceptance_viewport_selection: 8 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
