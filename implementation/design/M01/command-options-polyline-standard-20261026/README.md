# Bộ đặc tả chuẩn hóa Command theo Polyline cho GPT Luna

Ngày 2026-10-26 — **documentation-only; implementation mục tiêu chưa được thực thi bởi bộ tài liệu này**.

Người dùng yêu cầu mô tả thật chi tiết để điều chỉnh tất cả command khác dựa trên Polyline. Bộ này là tài liệu bàn giao trong repository, không tạo chat/agent mới, không đổi model, không sửa skill và không tự phát hành OM9.

## Đọc hoặc giao việc

| Tài liệu | Dùng để làm gì |
|---|---|
| [SPEC.vi.md](SPEC.vi.md) | Hợp đồng đích: owner/run, typed values, state machine, input, inline prompt/spans, units, focus, commit/cancel/Undo, nghiệm thu |
| [COMMAND-MATRIX.vi.md](COMMAND-MATRIX.vi.md) | 39 nhóm khai báo option có file/hàm/ID; 19 pending owner cần audit thêm |
| [OWNER-CARDS.vi.md](OWNER-CARDS.vi.md) | Phiếu theo từng family/mode, parameters, ràng buộc, traps và geometry checks |
| [TASKS.vi.md](TASKS.vi.md) | 13 gói P00..P12 có thứ tự, file cần sửa, tests và điều kiện kết thúc |
| [ACCEPTANCE.csv](ACCEPTANCE.csv) | Các mẫu ca kiểm thử có ID, precondition, bước chạy và expected result; tất cả là NOT_RUN_TEMPLATE |
| [source-inventory.json](source-inventory.json) | Snapshot source/hash, token payload, điều kiện availability và pending inventory để kiểm chéo |
| [LUNA-PROMPT.vi.md](LUNA-PROMPT.vi.md) | Prompt có thể copy giao Luna đọc bộ này và thực hiện |

Để giao triển khai, dùng prompt ở file cuối. Để rà nội dung, mở SPEC trước, rồi đọc owner card của family đang xét. Các API mới trong SPEC/TASKS là thiết kế đề xuất, không gọi như thể đã tồn tại trong checkout.

## Điểm phạm vi quan trọng

Hiện đã có common interface cho 37 provider + Line + Polyline = 39 nhóm. Đây chưa phải mọi command có trong ứng dụng và chưa phải chứng nhận toàn bộ parity. Inventory Controller có 57 field `_pending`, 19 field ngoài registry; còn phải reconcile các entry point không dùng pending. Số field/provider không phải số hàm, command hay tỷ lệ hoàn thành.

Các việc đích còn bao gồm owner-owned value entry thay cho bridge Name= ở từng family, typed current values, run-aware identity, một formatter/span và inline render path, classification cho positional/immediate/specialized/unsupported commands. Kernel/geometry và semantics từng owner được giữ theo local milestone; không copy options của Polyline sang lệnh khác.

## Xác minh tài liệu

`documentation-validation.json` ghi kiểm inventory, đủ owner cards, thứ tự task IDs, CSV schema, nguồn/hash và links của gói này; kiểm public-source/diff được ghi trong phần authoring khi hoàn tất. Kết quả đó chỉ chứng minh gói bàn giao nhất quán. Không thay geometry tests, executable/runtime proof hoặc human milestone gates.

Chạy trước khi bắt đầu implementation:

```powershell
rtk proxy python implementation/design/M01/command-options-polyline-standard-20261026/validate_bundle.py --write-report
```

Validator chỉ đọc source và tài liệu, có thể ghi report khi truyền `--write-report`. Nó đối chiếu **baseline nguồn ngày viết**. Sau khi migration làm source đổi, báo hash drift là điều cần được ghi trong execution checkpoint, không phải lý do sửa lại baseline cho xanh hoặc quay lui code mới. Nếu cần inventory mới, lưu một snapshot refresh riêng và giải thích delta. Các kiểm tra production vẫn phải chạy riêng theo từng gói.
