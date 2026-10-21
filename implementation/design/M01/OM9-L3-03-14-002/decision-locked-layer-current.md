# Quyết định còn treo: layer đang **khoá** có được làm **layer hiện hành** không?

Ngày ghi: 2026-10-20 · trạng thái: **CHỜ NGƯỜI QUYẾT** (không chặn phần còn lại của module)

## Hiện trạng (đã có test)

- `assign_layer` **từ chối** đưa object **vào** một layer bị khoá: `Error::Permission("layer is locked")`
  (và từ chối cả khi nguồn nằm trên layer khoá: `Permission("object is on a locked layer")`) — `tests/layer_current_and_assign.rs`.
- "Layer hiện hành" là **trạng thái phiên** (ô đang chọn trong bảng màu), **không** phải thay đổi tài liệu: không sinh
  mutation, không vào journal Undo.

## Hai lựa chọn

**A. Giữ nguyên: layer khoá KHÔNG được làm hiện hành.**
- Lý do: "khoá" nghĩa là không sửa; chọn làm hiện hành là bước đầu của việc vẽ/sửa vào layer đó.
- Thay đổi cần làm: thêm guard trong hàm đặt layer hiện hành + 1 test âm; cập nhật fixture `negative.json`.
- Rủi ro: người dùng vẫn muốn **xem** layer khoá mà không muốn vô tình vẽ vào — bị chặn hơi sớm.

**B. Cho phép layer khoá làm hiện hành, nhưng mọi lệnh ghi vào nó vẫn bị từ chối.**
- Lý do: nhất quán với Matrix9/Rhino: khoá chặn *biên tập*, không chặn *chọn để xem/tham chiếu*; luật chặn đã có ở
  `assign_layer` và ở mọi mutation (kiểm `layer_permissions(..., false).editable`).
- Thay đổi cần làm: **không** thêm guard; thêm 1 test dương ("đặt layer khoá làm hiện hành rồi thử vẽ → lệnh bị từ chối").
- Rủi ro: người dùng thấy layer khoá là hiện hành nhưng lệnh không chạy → cần dòng trạng thái giải thích (đã có
  `Error::Permission("assigned layer")`).

## Đề nghị của tôi

**B**, kèm thông báo rõ khi lệnh bị từ chối, vì nó khớp với mô tả Matrix9 trong tài liệu tham chiếu (layer khoá vẫn
hiện trong bảng, chỉ mờ/khoá biểu tượng) và không nới lỏng bất kỳ luật ghi nào.

## Khi bạn quyết

Chỉ cần trả lời **A** hoặc **B**; tôi sẽ áp dụng đúng phần "thay đổi cần làm" ở trên, cập nhật fixture, ui-review L3-002,
api.md và chạy lại `python tools/verify_m01_l2_03_14.py` (kỳ vọng 15/15 + test mới).
