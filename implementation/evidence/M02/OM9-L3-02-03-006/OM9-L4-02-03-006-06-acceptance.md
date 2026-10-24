# OM9-L4-02-03-006-06 — Smarttrack Inferred Alignment Lines acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Hai end snap khác nhau sinh SmartTrack alignment tạm thời qua điểm tham chiếu
- **Negative oracle:** Không có anchor/đã cancel thì xoá guide, không ghi guide vào document
- **Acceptance:** Two accepted end snaps yield transient alignment guides and an inferred intersection; clearing or cancelling removes every guide without a document write.
- **Evidence:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
