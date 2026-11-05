# Picking RAM: unused quota, recency và đọc bounds thụ động — 2026-11-05

Phạm vi: tiếp tục R1 trên weak owner index và tài khoản RAM production đã có. Không đổi Undo/document, độ chính xác hình học, profile mesh hoặc chính sách chất lượng viewport. Không mở, đóng hoặc sửa phiên OM9 người dùng. Bàn giao trước: [pressure reclamation và read handles](2026-11-05-picking-cache-pressure-reclamation.md).

## Lỗi tái hiện và thay đổi

Hai regression mới trên code trước sửa đều fail: request chỉ thiếu một byte đã xóa arena picking vừa dùng, dù snapshot khác có quota chưa dùng; thứ tự đăng ký cũng làm arena vừa đọc lại bị xóa trước arena lâu không dùng. Run đỏ `build/standalone-validation/ui-artifacts/picking-qos-20261105-043703`: 10 pass, 2 fail, exit 101. Không gọi hai failure đó là lỗi hình học hoặc mất dữ liệu: cache có thể dựng lại, nhưng việc thu hồi sai làm tăng công việc tương tác.

Admission vẫn thử request trước và chỉ thu hồi sau lỗi ngân sách có thể xử lý. Reclaimer nay trả unused quota trước; nếu vẫn thiếu thì dùng thứ tự lần đọc picking gần nhất thay thứ tự đăng ký. Sau mỗi owner được thu hồi, request được thử lại và dừng ngay khi vừa. Pin/try-lock, old-new overlap, bounded retries và lifetime lease giữ nguyên. Touch sử dụng lại Vec weak index, không thêm sổ RAM, không clone document hoặc kiểm kê toàn lịch sử. Đọc owner ở cuối hàng là O(1); chuyển owner khác lên cuối là O(số owner cache), không O(số object). Scan thu hồi vẫn giới hạn theo watermark; cache được đăng ký/touch sau watermark có thể được hoãn sang lượt sau. Chính sách recency là quan sát đồng thời, không hứa cache vừa đọc không bao giờ bị thu hồi khi còn thiếu RAM.

`Scene::cached_object_spheres()` là đường đọc bounds thụ động: chỉ clone handle cache hợp lệ đang tồn tại, dùng try-lock; không allocation, probe RAM/commit, admission, eviction, dựng BVH hoặc nâng recency. Cold/busy/invalid trả None để render có thể vẽ bảo thủ đủ object. Reader vẫn giữ lease và dữ liệu qua preview/Scene drop. `object_spheres()`/`spatial_index()` phục vụ picking tiếp tục dựng theo admission thật và ghi nhận recency.

Hai helper đọc spheres của GPU đã chuyển sang đường thụ động. Luồng GPU/UI hiện đang khóa `close = false` để giữ đủ hình học; vì vậy không nói đây là nguyên nhân mới tìm được của giật desktop hay tối ưu một đường culling đang hoạt động. Giá trị trực tiếp của sửa này là chính sách thu hồi picking dùng chung và ngăn các helper render tự chiếm cache khi được gọi. Không bật lại close culling, LOD hoặc giảm độ phân giải.

## Oracle và cấu hình

Windows x64, default features, target riêng `E:\om9-surface-snap-validation`, Cargo `-j 2`, dev debug=0/incremental=0; OCCT 8.0.1 SDK cùng TBB/jemalloc như bàn giao trước. Không thay native C++/SDK. Headless và GPU offscreen không thay release/desktop/WPR.

Bộ managed-cache hiện có 15 tests, 16 canonical document fixtures, mỗi fixture 200 point objects; bốn render snapshots là bốn owner của cùng nguồn, không phải bốn fixture canonical mới. Năm test mới kiểm:

- Unused quota được trả trước arena active; document snapshot nguyên vẹn.
- Arena được đọc lại gần nhất sống qua việc thu hồi arena snapshot cũ hơn.
- Cold/evicted passive bounds không dựng lại; 240 lượt warm và 240 lượt cold không gọi physical sampler (sampler panic nếu bị gọi), pending/allocated đúng.
- Passive reader không lấy bounds preview cũ, nhưng handle cũ giữ đúng dữ liệu/lease đến khi thả.
- Bốn passive snapshot readers, 240 lượt trước và 240 lượt sau áp lực: polling không nâng recency, view cũ nhất bị thu hồi một lần, cache không tự dựng lại; active picker cùng các view còn lại giữ đúng charge, không sinh pending mới.

Pressure fixture giữ reservation đến sát trần 64 MiB và chừa đúng validation scratch, rồi yêu cầu thêm một byte. Đây là kiểm admission có thật trên tài khoản production, không phải cấp phát 64 MiB payload hay đo RSS. Các vòng polling là đọc CPU bounds, không phải 240 frame GPU hay số đo FPS. Các test cũ tiếp tục kiểm reader pinning/retirement, concurrent request, failed commit recheck, cold re-admission và preview overlap.

Batch `build/standalone-validation/ui-artifacts/picking-qos-20261105-043923` kết thúc exit 0: **262 passed, 1 ignored, 0 failed**. Sau đó chỉ thêm một GPU regression test, không sửa production source; batch `build/standalone-validation/ui-artifacts/picking-qos-gpu-20261105-044231` kết thúc exit 0: **29 GPU passed, không có SKIP**. Tổng riêng biệt 263 passed, 1 ignored; không cộng 28 GPU lặp lại thành 291 pass.

| Nhóm | Passed / ignored | Elapsed gồm build (giây) |
|---|---:|---:|
| Managed picking | 15 / 0 | 2,33 |
| Core retained admission | 101 / 0 | 20,90 |
| Scene unit, instance admission, shared instances, surface broad phase | 40 / 0 | 4,82 |
| Presentation | 10 / 0 | 48,79 |
| Hover | 14 / 0 | 9,76 |
| Incremental scene memory | 7 / 0 | 1,55 |
| Picking/Gumball interaction/Gumball viewport/overlay | 47 / 1 | 19,92 |
| GPU offscreen, lần cuối | 29 / 0 | 14,84 |

Một overlay responsiveness measurement vẫn ignored theo cấu hình sẵn có. GPU regression mới chạy renderer/queue completion thật: cold bounds không materialize cache picking, frame stats vẫn đủ bốn object; làm nóng picking rồi đổi camera không tăng geometry/instance upload hay scene preparations, document snapshot giữ nguyên. Đây là oracle publication/counter/object coverage, không phải đối chiếu pixel readback hoặc màn hình desktop. Các nhóm không bị thay đổi sau batch đầu không chạy lại vì chỉ thêm test GPU.

Fingerprint SHA-256 `ac3ec21687db660124b499b2331d78f91dd90b1c7a612419d20112509a39e9e5`: nối theo thứ tự UTF-8 path + NUL + raw bytes của bảy file dưới đây; không phải Git commit hay fingerprint toàn repo:

```text
crates/om9-core/src/transaction.rs
crates/om9-core/src/transaction/cache_pressure.rs
crates/om9-scene/src/scene.rs
crates/om9-scene/src/scene/picking_cache.rs
crates/om9-scene/tests/managed_picking_cache.rs
crates/om9-app/src/gpu_viewport.rs
crates/om9-app/tests/gpu_navigation_publication.rs
```

Plan/alias, README, live ledger, runtime owner M01 L2-01-02/03/06 và L3 shared graphics được đồng bộ theo phạm vi này. `tools/public_source_audit.py` exit 0: 15.181 source files, 594 icon bindings, 593 SVG, errors rỗng. JSON, fingerprint, terminal summaries và 43 local Markdown links đã kiểm tra; `git diff --check` không có lỗi whitespace trong phạm vi bàn giao (chỉ cảnh báo CRLF/LF sẵn có). Prune `tools/prune_build_artifacts.py --apply` sau kiểm tra đích trong checkout: không có run đủ điều kiện xóa, giữ 10 mới nhất mỗi root và các run được dẫn; không xóa target ngoài checkout. Không nâng toàn R1 hoặc milestone thành complete từ các kết quả trên; không chạy lại Rust sau thay đổi chỉ ở hồ sơ.

## Phần còn lại

- Đo model/scene đại diện, RSS/private commit và release/desktop dưới áp lực thật. Không suy độ mượt, 30/24 FPS hoặc tốc độ chuột từ stress admission này.
- Các cache/auxiliary/query/registry/peer/journal owners ngoài phạm vi picking vẫn cần attribution và diagnostics R1. Display cache/v5, GPU staging/fence/batching và UMA theo P1/P2 còn mở.
- Đỉnh allocation OCCT trước Rust copy callbacks, native admission envelope, phase scratch/hash capacity và STL worker UI vẫn mở theo R2/P3b.
- Không có openNURBS/3DM measurement batch mới; không đóng gói nghiệm thu exchange nào. Tổng batch tương lai chưa xác định. Không chạy lại large RAM cases ở lượt này.

Hành động tiếp theo: audit owner/diagnostics R1, sau đó native envelope/peak R2 và worker UI, đồng thời giữ các gates còn lại trong [plan](../../implementation/design/M01/viewport-export-separation-20261102/PLAN.md). Toàn mục tiêu vẫn chưa hoàn thành.
