# Bỏ legacy scene high-water khỏi STL admission — 2026-11-05

R1a: bỏ `scene_high_water` và phần scene bytes chụp theo peak trong Controller/STL capture; không đổi nguồn geometry, mesher/tolerance, native adapter, input hoặc phiên desktop đang mở.

## Căn cứ và sửa

Public scene construction/publication đã có buffer và metadata owners qua các bàn giao R0/R1 trước. `cache_held_usage.bytes - registered_scene_buffer_bytes` bằng 0 trên các đường đã kiểm; các owner giữ đúng bytes theo lifetime. Preview Gumball chỉ đổi transform trong SceneObject và invalidate picking, không tạo scene geometry mới. Vì vậy shared atomic peak không còn là nguồn truth cho các scene đã đăng ký.

Counter cũ vẫn được tạo/đọc ở startup, publication và query; capture sống khiến peak chỉ tăng dù buffers sau đó đã có owner. Regression mới dùng **test-only** `refresh_scene`: publish buffers chưa đăng ký, rồi đăng ký chúng trước khi thử export. Kernel reservation pressure chừa đúng initial export allowance. Trước sửa, initial admission trả Budget thay vì tới callback hủy, dù scene lúc xuất đã có owner. Red log nằm trong `build/standalone-validation/ui-artifacts/scene-peak-removal-20261105-071248/red-before-removal.log`. Đây là lỗi tái hiện ở đường fixture chuyển tiếp; chưa có bằng chứng người dùng gặp nó qua publication thông thường. Không nhận khoản tiết kiệm RAM/FPS đo được từ regression này.

Đã xóa atomic Arc/counter và mọi fetch-max/read trong runtime app. Capture kiểm scene hiện hành có đủ registered metadata/buffer bytes trước khi cấp selection IDs; thiếu owner trả `Invalid("STL scene ownership")` thay vì dùng peak dự đoán. Đường publication hợp lệ giữ nguyên; partial scene fixture phải đăng ký owner rồi mới capture. Test guard kiểm document/selection/Export reservation không thay khi từ chối và retry sau bind thành công.

Detached capture chỉ còn `caller_held_at_capture`: snapshot của auxiliary chưa có owner và các structural quantities đang dùng. Nó không giữ scene RAM/history peak. Controller export vẫn lấy caller usage hiện hành; peer nhận scene/Doc/selection owners qua account chung. **Auxiliary snapshot còn có thể cũ**: workspace/file/recent/pending owners chưa xong, nên không đóng toàn R1a. Giữ các structural/overflow limits hiện có, không bỏ non-byte quantities khi bỏ scene bytes.

## Kiểm chứng

Windows x64, default features, OCCT 8.0.1 SDK/TBB/jemalloc; target `E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0. Run cuối `build/standalone-validation/ui-artifacts/scene-peak-removal-20261105-071248`, script `build/run-scene-peak-removal-20261105.ps1 -Full`.

| Nhóm | Passed / ignored |
|---|---:|
| STL unit, gồm 2 test mới | 63 / 0 |
| Root / subselection memory | 2 + 3 / 0 |
| Scene memory / replacement | 7 + 8 / 0 |
| Document / file / ordinary RAM | 12 + 20 + 9 / 6 |
| Presentation / hover | 13 + 14 / 0 |
| GPU offscreen publication | 29 / 0, không SKIP |
| B-rep picking / Gumball / overlay | 19 + 7 + 7 + 14 / 1 |
| Native STL integration | 60 / 0 |

**287 distinct passed, 7 ignored, 0 failed**. Hai fixtures mới là triangle mesh (một có thêm 30000 vertices không tham gia face để buộc buffer lớn). Pressure reservation không tạo payload tương ứng, không là test OS OOM. 6 large cases và 1 responsiveness case chưa chạy. Không chạy lại core vì không sửa core/API đó; không đo release/desktop/RSS mới hoặc xác minh global deviation STL/3DM.

Growth regression cũ nay đối chiếu registered bytes thật qua paste/Undo và fresh capture, giữ capacity reduction/destination-preservation checks; không thay bằng assertion peak luôn 0 trên một field giả. Source scan xác nhận các tên counter cũ không còn trong runtime app. Những regression buffer/Weak/history/picking trước tiếp tục pass. GPU offscreen không thay desktop/WPR.

Elapsed gồm build: STL unit 27,71 s, root/subselection 0,80/0,80 s, scene memory 1,59 s, replacement 4,67 s, lifecycle 29,20 s, presentation 0,97 s, hover 9,66 s, GPU 14,77 s, picking/Gumball/overlay 19,36 s, native STL 115,77 s (runtime 111,64 s). Plan/alias, README, live ledger và runtime owners được cập nhật đúng phạm vi. Không đổi skill nghiệp vụ. R1a còn live auxiliary workspace/file/recent/pending; R1b input/query/peer/retired, R2 native peak, UI worker, P1/v5, GPU batching, chính xác STL/3DM, checkpoint và release/desktop vẫn mở. Không có batch đo 3DM/openNURBS mới, exchange packages chưa đóng; tổng batch tương lai chưa xác định.

Fingerprint SHA-256 nối repo path UTF-8, NUL, bytes file theo thứ tự dưới: `95ebcba2ad8cf655e653adafe57415ffb6d417190205aa12e3d2fadafa2fa0d2` (không phải Git commit/toàn checkout).

- `crates/om9-app/src/session.rs`
- `crates/om9-app/src/session/layers.rs`
- `crates/om9-app/src/session/stl.rs`

Public-source audit exit 0: 15.193 source files, 594 icon bindings, 593 SVG, errors rỗng. Fingerprint, terminal summaries, JSON ledger và 3 owner files, 41 local Markdown links đã kiểm; `git diff --check` trong scope đạt. Prune qua tool dự án sau kiểm tra resolved targets trong checkout đã bỏ 1 run không còn được dẫn, giữ 10 mới nhất mỗi root cùng các run được hồ sơ dẫn; không xóa target ngoài checkout. Log `build/standalone-validation/scene-peak-removal-prune-20261105.log`. Không chạy lại Rust sau thay đổi chỉ ở hồ sơ. [Plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md).
