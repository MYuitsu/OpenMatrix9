# OM9-INFO-022 — Relocate Gumball

Current 2026-10-12: `implemented` / `validated_supported_slice`; approved OM9 scope complete; stock CI pending.
[Final native evidence](../validation/2026-10-12-gumball-core-completion.md).
The dated 2026-10-11 implementation/evidence below is the validated baseline.

Ngày 2026-10-11; `partially_implemented` / `validated_supported_slice`.
Command `RelocateGumball`, alias `GumballRelocate`, `OM9_RelocateGumball`,
legacy `OM9_InfoSettingsRelocateGumball`.
[Spec](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-022-relocate-gumball.md).

Nguồn: Matrix 8 Book 1 PDF 45–47 / trang in 35–37; PDF 48 bắt đầu History.
CMD nhận origin rồi X point; Enter tạo frame theo CPlane nếu đủ hướng.
Ctrl có từ press rồi thả Ctrl và kéo chỉnh widget, không dịch object.

OM9 dùng coordinate parser hiện hữu (document units, CPlane hoặc `w` world),
native CoreSnaps pointer pick; optional Y point giải frame khi X song song
CPlane normal. Rust Gram-Schmidt chuẩn hóa frame; zero/collinear/nonfinite
reject, giữ phase/frame trước. Optional Y là lựa chọn OM9/Rhino được duyệt.
Nếu native snap tắt hoặc không có candidate, Rust giao world ray với captured
CPlane; ray song song bị từ chối, không dùng một điểm chiếu tùy ý.
Ctrl numeric Move/Rotate/Scale chỉ đổi pivot/basis, không scale geometry.

Pivot/frame là view state, không object/property/Undo hay associative History.
Single owner còn cùng stamp có cache session 256 entries; switch selection
hoặc explicit Alignment theo quy tắc đã ghi ở [Alignment](OM9-INFO-021.md).
Document/view/workbench close và navigation dọn editor/mouse/preview.
Cancel khôi phục frame trước gesture. Không claim pivot cache bền qua cold FCStd.

Native CMD origin/X/Enter/optional Y và Ctrl-start/numeric đã kiểm qua real Qt;
matrix tests pin parallel X, invalid axes, cancel and owner cache.
Các pointer snap modes và toàn bộ original-host interaction chưa có parity proof.
[Evidence](../validation/2026-10-11-gumball-core.md).
Tiếp theo: Matrix/Rhino comparison fixtures cho mọi snap mode và ambiguous owners.

Revision spec ngày 2026-10-11 yêu cầu Relocate preview/accept/cancel dùng
chung origin/basis ở cả bốn ô, tính lại Scale anchors/leader/hit targets,
phân biệt Ctrl-start widget-only và late-Ctrl Extrude. Input thường theo
captured CPlane/document units; `w` chọn world, không gọi mọi typed point
là world point. Default/cache scope và giới hạn native cũ được giữ.
Đã triển khai origin/X preview đồng thời ở mọi view; optional Y accepts frame
chung, bounds Scale/leader/picks cập nhật theo basis. Ctrl numeric widget-only
và late Ctrl Extrude replay ở từng ô; idle focus giữ pivot, source close hủy
gesture và mọi preview/capture. Không tạo geometry/Undo cho Relocate.
[Evidence mới](../validation/2026-10-11-gumball-four-view-completion.md).
[Đối chiếu nguồn lịch sử](../validation/2026-10-11-gumball-four-view-spec-review.md).

### Approved core closure — 2026-10-12, native gates passed

The same origin/X/optional-Y and Ctrl-start view-state workflow must operate on
the expanded owner and typed document-frame selection without changing native
definitions, dimensions, dependencies, Placement or Undo. Cancel restores the
shared frame. Typed Camera/Light object FCStd restoration is distinct from
widget persistence: the bounded256 single-owner pivot cache remains deliberately
process-local. Reset, explicit Alignment and source-stamp invalidation clear the
override; groups/multiple owners do not inherit another owner's cached pivot.
Final-source native view-only and cleanup gates passed. Rhino internal
roll, complete snap parity and cold persistence of the widget are outside the
approved scope.

[Candidate fixture map, applicability and gate status](../validation/2026-10-12-gumball-core-release-candidate.md).
