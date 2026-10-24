# OM9-L4-03-09-001-06 — F6 Selected Object Class Dispatch acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 với selected curve hiển thị nhóm curve; với box hiển thị solid actions
- **Negative oracle:** Mixed selection không ép về Gem sai; unknown semantic chọn General
- **Acceptance:** A curve selection yields a Curve group and a mesh/solid selection a Solid group; mixed selections never force Gem mode.
- **Evidence:** om9-app f6 unit: 7 passed; om9-app f6_context: 2 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
