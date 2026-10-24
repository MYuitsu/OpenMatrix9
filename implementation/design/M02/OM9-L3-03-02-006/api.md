# OM9-L3-03-02-006 — Viewport Center Reset And Home

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/workspace.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Home phục hồi camera preset rồi lệnh center đưa tâm về world origin
- **Negative oracle:** Không reset CPlane/document layers khi chỉ reset camera
- **Verification:** om9-app navigation_commands: 6 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
