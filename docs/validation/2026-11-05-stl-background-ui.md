# Mốc A xuất STL nền trên hieu-nang-2

Ngày 2026-11-05. Bản nền sau merge PR #3 ở `dc2d4dfd665e14d1a68aa9c30f1fd59daca6db02`. Log và fixture nằm trong [batch STL](../../build/standalone-validation/ui-artifacts/stl-background-20261105/).

## Kiểm release trước thay đổi

- `cargo build --release --locked -p om9-app -j 2` đạt trong 6 phút 34 giây. Target `L:/om9-target`, Windows x64 và SDK OCCT 8.0.1.
- Release qua MCP chạy Line bằng CMD, Circle qua route menu và điểm, Undo/Redo. Sphere bán kính 5 mm xuất với 0,01 mm và 0,3 rad. Kiểm số tam giác, kích thước binary và trạng thái document trước/sau. [Receipt](../../build/standalone-validation/ui-artifacts/stl-background-20261105/baseline.json)
- Cửa sổ release thử riêng, Perspective 1600×1000 ở DPI 100%. Kéo Gumball Snappy bằng con trỏ Windows tổng hợp. Hai đầu đường thẳng từ `[0,0,0]`, `[10,0,0]` đến `[0,5,0]`, `[10,5,0]`. Ảnh GPU và receipt được lưu, tiến trình thử đã thoát. [Desktop receipt](../../build/standalone-validation/ui-artifacts/stl-background-20261105/desktop/os-acceptance.json)
- Phạm vi Curve của lần này là route dispatch qua MCP. Test add-on, Top11 và grip theo [báo cáo tích hợp](2026-11-05-pr3-hieu-nang-2-integration.md).

## Luồng xuất nền

- `Export Selected` chụp document và selection khi gọi lệnh. Chọn profile và đường dẫn xong, UI chuyển capture vào một worker. Menu và CMD dùng chung đường này.
- Worker dùng chung tessellation, topology, lượng tử hóa và writer staging với đường đồng bộ. UI hiển thị trạng thái đang xuất hoặc đang hủy, và cho thao tác tiếp trên tài liệu. Kết quả của document session cũ bị bỏ qua.
- Worker xin reserve/grow theo pha. UI áp yêu cầu bằng `scene_cache_usage` tại thời điểm cấp RAM trên account chung. Capture, bộ nhớ đã cấp và quyền xuất sống qua native call. Receipt giữ lease tới lúc UI đọc. Đóng cửa sổ đã được duyệt đợi worker dọn staging trước khi thoát.
- Phần đầu vào đã biết của grouped B-rep được tính trước OCCT. Bản sao chuỗi vào `istringstream` được giữ trong phần tính overlap đến callback. Giới hạn nội bộ khi OCCT dựng shape và tessellate vẫn là phần mở của R2a.

## Batch kiểm chứng

- Lượt tích hợp gồm `stl_export` 60, `file_lifecycle` 20, `file_widgets` 21 và `project_manager_persistence` 5. Tất cả 106 test đạt. [Log](../../build/standalone-validation/ui-artifacts/stl-background-20261105/stl-lifecycle.log)
- Nhóm lib lọc `stl` cuối cùng đạt 77 test. Kiểm Cancel qua widget, byte parity trên sphere, edit/Undo/Redo khi job giữ nguồn, áp lực RAM sau capture, đóng cửa sổ và Open lại cùng UUID. Kiểm từ chối stream-copy trước tessellation. Lần kiểm compile đầu của test stream-copy thiếu import nội bộ, được sửa trong test. [Log](../../build/standalone-validation/ui-artifacts/stl-background-20261105/stl-unit-final.log)
- Hai nhóm kết hợp có 183 ca khác nhau. Lượt lib đầu 75 test là kiểm trung gian trước hai ca cuối và đã được thay bằng kết quả 77.

<!-- RELEASE_EVIDENCE -->
Build release cuối đạt trong 5 phút 02 giây. Binary `L:/om9-target/release/om9.exe` có SHA-256 `d2da11e1a7b3e6b35776138c4fdca21982c4a5adf67e3c71ca896f49c8a15c2a`. Release mới chạy lại phép kiểm Line, Circle, Undo/Redo và sphere qua MCP. STL sphere cùng profile có 10.148 tam giác, 507.484 byte và trùng SHA-256 `bdd5f489776e070ad95285099359ae51bb0a5add0f100ba36315c25cd20303eb` với bản nền. [Receipt release](../../build/standalone-validation/ui-artifacts/stl-background-20261105/candidate-cli/baseline.json)

| Fixture release | Dung sai (mm) | Tam giác | Thời gian đo (ms) | Peak commit (MiB) | Peak working set (MiB) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Sphere bán kính 12 mm, adapter native tới callback | 0,001 | 241.324 | 535,1 | 165,05 | — |
| Block có hốc cầu cắt trim, toàn luồng xuất | 0,01 | 6.214 | 63,9 | 79,07 | 75,01 |
| Bề mặt NURBS cong, toàn luồng xuất | 0,01 | 1.004 | 21,6 | 54,82 | 53,07 |
| Tám instance dùng chung sphere, toàn luồng xuất | 0,005 | 159.600 | 568,4 | 93,38 | 93,84 |

Mỗi dòng là một fixture trong process release riêng của `stl_memory_probe`. Sphere ghi peak commit 54,54 MiB trước call và 165,05 MiB tại callback nhóm shell. Ba dòng còn lại đo toàn luồng controller và encoder với góc 0,3 rad. Peak Windows tích lũy trong vòng đời process, gồm khởi tạo fixture. Bảng cung cấp mẫu quan sát cho R2a và để mở phần giới hạn cứng của bộ cấp phát OCCT. Log `release-memory-*.log` cùng batch.

Bản release được Cargo dựng trong batch test đã mở cửa sổ fixture STL riêng. Quyền Computer Use hết thời gian chờ khi đọc cửa sổ nên bước chọn đích trong hộp lưu Windows còn mở. Cửa sổ fixture còn mở với dấu sửa sau yêu cầu đóng. Bản release build thường đã được xác minh qua CLI và so hash ở trên.
<!-- END_RELEASE_EVIDENCE -->

## Phần còn mở

Kiểm tra desktop bổ sung: release `L:/om9-target/release/om9.exe` mở fixture sphere riêng, chọn đối tượng qua viewport, gọi File → Export Selected STL và hoàn tất qua hộp lưu Windows. Với yêu cầu 0,001 mm / 0,3 rad, UI báo 100.644 tam giác, 5.032.284 byte. File thử có SHA-256 `473831acc40f18a279f7f41641c3a4d49c6ab1b5b140558971272dfefe5787bb`. Phép thử này chỉ xác nhận đường hộp lưu và xuất hoàn tất trên sphere, chưa kiểm sửa đồng thời, Cancel hoặc model nặng. Cửa sổ fixture không còn tiến trình sau phép thử. Cần tiếp tục cổng A với fixture đủ lâu để quan sát cả hai hành vi.

Đã tạo fixture riêng `heavy-spheres.om9` gồm 24 B-rep sphere, xác nhận bằng trạng thái document của MCP và mở bằng release. Cửa sổ thử hiện được nhận diện theo tên fixture. Windows Computer Use hai lần báo có input người dùng trên cửa sổ thử và yêu cầu quan sát lại, chưa gửi được thao tác chọn đối tượng. Chưa có kết quả edit/cancel hoặc kết luận cho model nặng.

Kiểm lại trên cùng fixture, chọn cả 24 sphere bằng quét viewport và xuất tại 0,001 mm / 0,3 rad. Ở lượt hủy, UI đang hiện `Exporting STL` rồi báo `STL export cancelled` sau khi nhấn Cancel. File đích thử `heavy-cancel-target.stl` giữ SHA-256 trước và sau là `bdd5f489776e070ad95285099359ae51bb0a5add0f100ba36315c25cd20303eb`. Ở lượt xuất hoàn tất, trong khi progress còn hiện, kéo Gumball dịch nhóm và document hiện dấu sửa. STL thu được 2.415.456 tam giác, 120.772.884 byte, SHA-256 `f26cacaa26a7cc15585e4302994d96bfe9619f6f6e0f81b6e13968b83369abdf`. Đọc toàn bộ tam giác cho bbox x `[-4,999755859375; 80]`, y `[-4,999755859375; 49,999755859375]`, z `[-5; 5]`, phù hợp vị trí nguồn 24 sphere trước khi dịch. Đây là kiểm desktop trên fixture lặp, chưa đo thời gian frame, thời gian hủy hoặc chứng nhận sai số toàn miền. Cửa sổ fixture có thay đổi chưa lưu được giữ lại.

Các ca kiểm chứng so sánh pipeline và profile giống nhau. Kiểm sai số STL toàn miền, containment và self-intersection tổng quát thuộc P3a. Mức bộ nhớ quan sát thuộc fixture và cấu hình ghi trong batch. R1b tiếp tục xử lý owner nhỏ có phạm vi cụ thể. Cổng desktop nặng và WPR thuộc P5.

Audit nguồn đạt trên 15.799 file, 594 bindings và 593 SVG. `git diff --check` đạt. Prune giữ 10 batch mới nhất và các batch được dẫn, không có mục đủ điều kiện xóa. [Manifest nguồn Rust](../../build/standalone-validation/ui-artifacts/stl-background-20261105/source.sha256)
