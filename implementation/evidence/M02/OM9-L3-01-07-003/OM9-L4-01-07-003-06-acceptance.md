# OM9-L4-01-07-003-06 — Subobject Face Edge Vertex Picking acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Chọn đúng face/edge/vertex của box và lưu khóa topology bền
- **Negative oracle:** Click vùng trống hoặc face không còn sau rebuild báo miss, không đổi sang mặt gần đó
- **Acceptance:** The three subobject kinds report their own identifiers; a face that disappears after reuse/rebuild is a stale miss and is never replaced by a nearby face.
- **Evidence:** om9-scene subobject_pick: 4 passed; om9-app subobject_selection: 8 passed; om9-app subobject_occurrences: 5 passed; om9-app brep_selection (gpu-sphere): 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
