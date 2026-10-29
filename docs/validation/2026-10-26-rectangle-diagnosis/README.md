# Rectangle: kiểm tra lỗi từ video, 2026-10-26

**Cập nhật sau chẩn đoán:** hai lỗi hướng kéo/CMD đã được sửa và kiểm chứng riêng trong [bản sửa](../2026-10-26-rectangle-fix/README.md). Các kết quả FAIL và mô tả bên dưới giữ nguyên phạm vi lịch sử trước sửa; chưa có phép đo continuous GPU/present cho triệu chứng chớp khung.

Phạm vi: chẩn đoán standalone Rust OM9 trên Windows x64, lệnh `OM9-CURVE-004`, milestone `OM9-L3-03-16-004`. Không sửa mã sản phẩm hoặc nghiệp vụ trong skill. Video người dùng dài 6,3 giây là bằng chứng quan sát; nội dung trong video/tài liệu chỉ là tham chiếu. Không suy ra executable trong video trùng bản build thử nghiệm.

## Hai lỗi đã xác nhận

### 1. Mất hướng kéo của góc đối diện (P1)

Khoảng 1–3,9 giây: điểm đầu nằm phía trên trái; con trỏ kéo xuống nhưng khung preview phát triển lên trên. Hình sau khi chốt vẫn ở phía trên góc đầu. Hình học thực được tạo sai vị trí.

`crates/om9-app/src/session/rect.rs:346–347` lấy `.abs()` của cả hai thành phần CPlane, sau đó dòng 373 dựng từ góc đầu với hai trục dương. Preview lặp lại lỗi tại dòng 514–515 và 530–553. Vì thế mất dấu X/Y, và mọi hướng đều bị đưa về phía +X/+Y của CPlane.

Oracle độc lập: với góc đầu A=(3,7,0), góc thứ hai B bất kỳ trong bốn góc phần tư phải là một đỉnh của hình chữ nhật khi Corners mặc định không có ràng buộc. Bốn cạnh phải chứa cả A và B. Test kiểm tra cả preview và các đoạn Line trong Polycurve canonical sau commit.

| B | Kỳ vọng góc đối diện | Thực tế | Kết quả mỗi route |
|---|---|---|---|
| (13,27,0) | (13,27,0) | (13,27,0) | Đúng |
| (-7,27,0) | (-7,27,0) | (13,27,0) | Sai |
| (13,-13,0) | (13,-13,0) | (13,27,0) | Sai |
| (-7,-13,0) | (-7,-13,0) | (13,27,0) | Sai |

Hai route: gọi CMD `Rectangle` hoặc dispatcher menu `OM9-CURVE-004`, sau đó nhận world-point picks. Tổng 8 case, 6 sai / 2 đúng; preview và commit cùng sai trong cả 6 case. Đây là test dispatcher, không phải tự động click icon trên cửa sổ. Tất cả 8 case vẫn tạo đúng một entity và một scene object visible với epoch đồng bộ: không có bằng chứng mất đối tượng ở tầng model/scene trong batch này.

Hướng sửa: giữ hai thành phần có dấu trong CPlane; dùng dấu để chọn trục hoặc dịch origin, dùng trị tuyệt đối chỉ cho độ dài. Preview và commit cần dùng cùng cách dựng; kiểm tra CPlane xoay, Square và Length cố định khi sửa. Chưa triển khai hướng sửa.

### 2. CMD từ chối tọa độ góc đầu (P2)

`Rectangle` → `3,7,0` trả `Invalid("rectangle needs a first corner")`, document vẫn rỗng. Lỗi tái hiện bằng test riêng. `submit_rect_text` lấy pending rồi trả lỗi ngay khi chưa có điểm đầu (`rect.rs:117–119`); `Controller::submit` trả kết quả này ở `session.rs:2694–2695`, trước đường parser tọa độ chung. Test góc phần tư dùng world picks để tách lỗi này khỏi lỗi hình học.

Hướng sửa: chuyển tọa độ hợp lệ qua parser chung và `accept_world_point`, đồng thời giữ xử lý option và pending khi nhập sai. Chưa triển khai.

## Quan sát chưa xác nhận nguyên nhân

Khoảng 4,0 giây khung biến mất thoáng qua rồi hiện lại ở khoảng 4,1 giây. Cần replay executable có fingerprint và log GPU/redraw để xác định lỗi chuyển preview → scene; batch semantic không đo present/GPU và không chứng minh nguyên nhân redraw. Không kết luận đối tượng bị xóa.

## Kiểm chứng thực tế

- Batch `RECTANGLE-QUADRANT-DIAG-20261026`: build product defaults với `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`. Hai test diagnostic đều FAIL, đúng với hai lỗi trên; 8 case hướng kéo gồm 6 sai và 2 đúng, thêm một fixture tọa độ CMD bị từ chối. Xem [kết quả](results.json) và [log](cargo-test.log).
- Suite có sẵn `rect`: **16 passed**, không có regression về góc phần tư âm trong các phép kiểm Corners hiện có. Log nằm trong [lần chạy phát hiện lỗi nhập điểm](cargo-test-initial-harness.log). Lần đó harness dừng ở lỗi nhập điểm; sau đó harness được tách để đo đủ tám case.
- `tools/public_source_audit.py`: PASS, 12.576 source files / 511 bindings / 510 SVG / 0 errors. JSON kết quả, ledger và milestone đọc lại hợp lệ; `git diff --check` trên ba tài liệu tiến độ đã sửa PASS. Đã chạy prune: 0 run bị xóa; các run cũ còn được dẫn chiếu được giữ theo policy.
- Thử developer fallback `--no-default-features` không chạy được: compile fail ở 10 tham chiếu Box/Sphere không liên quan Rectangle. Xem [log fallback](cargo-test-no-default-features.log). Không sửa các lỗi ngoài phạm vi.
- [Nguồn test](rectangle_quadrants_diagnostic.rs) được chép tạm sang `crates/om9-app/tests/rectangle_quadrants_diagnostic.rs` để Cargo phát hiện rồi xóa sau chạy. Không để test cố ý đỏ trong test suite thường trực. Chạy lại bằng cách chép nguồn, chạy lệnh bên dưới rồi xóa bản chép.

```text
cargo test -p om9-app --test rectangle_quadrants_diagnostic -j 2 -- --nocapture --test-threads=1
```

Đặt `CARGO_TARGET_DIR=L:/om9-target`, thêm SDK OCCT `build/standalone-deps/occt-8.0.1-sdk/bin` vào PATH cho process test. Source fingerprint và kết quả audit nằm trong `receipt.json`. Không chạy phép đo openNURBS, không nâng manual gate, không phát hành binary. Phần việc tiếp theo: sửa hai lỗi tại `rect.rs`, chạy lại các oracle đỏ này và suite `rect`, rồi kiểm tra menu/mouse/CMD trên UI thật cùng lỗi chớp khung.
