# OM9-INFO-022 — Relocate Gumball

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
