# OM9-L4-03-03-005-06 — Matrix Gumball Alignment And Relocate acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** World/CPlane/Object alignment chuyển đổi gizmo basis nhưng không sửa BRep
- **Negative oracle:** Gizmo origin relocation ngoài object không làm tự thay đổi object position
- **Acceptance:** World/CPlane/Object alignment changes the gizmo basis without moving the object; relocating the pivot leaves object placement unchanged.
- **Evidence:** om9-app gumball_placement: 4 passed; om9-app gumball_interaction: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
