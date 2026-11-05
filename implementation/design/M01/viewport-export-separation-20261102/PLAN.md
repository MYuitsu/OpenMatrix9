# OM9 — Việc còn lại: RAM, viewport và xuất STL/3DM

Cập nhật: 2026-11-05 theo yêu cầu chống loop và gom test theo mốc chức năng. Checklist chỉ giữ việc còn thiếu; phần đã kiểm chứng chuyển sang bảng hồi quy. [Kết quả FileState hiện hành](../../../../docs/validation/2026-11-05-live-file-state-memory.md) thay trạng thái còn lỗi 91 byte trong các biên bản trước. Triển khai đang tạm dừng; lượt cập nhật này chỉ sửa tài liệu, không build/test ứng dụng hoặc đo runtime. Phiên OM9 người dùng giữ nguyên.

## Cách đọc và trạng thái

- Chỉ các công việc còn thiếu hoặc chưa đủ bằng chứng nằm trong checklist bên dưới. Các chức năng đã có tiếp tục là ràng buộc hồi quy, không phải việc cần triển khai lại.
- [Hồ sơ bằng chứng](../../../../docs/openmatrix9-progress.json) và các báo cáo liên kết xác định phạm vi đã kiểm chứng. [Bản lịch sử trước khi rút gọn](EVIDENCE-HISTORY-20261105.md) giữ phân tích Rhino 5, Blender 5, CAD nguồn mở và các ghi chép cũ; trạng thái cũ trong đó không phải checklist hiện hành.
- Gumball: người dùng đã xác nhận không còn lỗi kéo đã báo. Chỉ mở lại điều tra khi có lỗi tái hiện hoặc thay đổi tác động trực tiếp đến input/picking/publication.

### Đã bỏ khỏi danh sách triển khai

| Phần đã kiểm chứng, không triển khai lại | Bằng chứng và giới hạn |
|---|---|
| Undo 1024 MiB, trần cấu hình 50% RAM, kiểm kê 100 MiB; đường tương tác >256 MiB | [Undo](../../../../docs/validation/2026-11-02-undo-ram-cap-and-inventory.md), [release Save/New/Open](../../../../docs/validation/2026-11-04-interactive-ram-transition.md). Không chứng minh toàn tiến trình đã được hạch toán. |
| Scene/viewport/hover giữ buffer và metadata owner qua publication, history và reader cuối; fixed account roots/Document Weak/presentation Weak có hạch toán | [Scene](../../../../docs/validation/2026-11-05-snapshot-master-reservation.md), [hover](../../../../docs/validation/2026-11-05-hover-shared-buffer-lease.md), [account](../../../../docs/validation/2026-11-05-account-root-and-weak-document-memory.md), [presentation](../../../../docs/validation/2026-11-05-presentation-weak-allocation-lifetime.md). Các owner khác còn R1. |
| Pending/allocated, physical/commit gate; picking arena/quota tự reclaim/re-admit, read handles và recency | [Phase/commit](../../../../docs/validation/2026-11-05-snapshot-metadata-system-commit.md), [pressure](../../../../docs/validation/2026-11-05-picking-cache-pressure-reclamation.md), [QoS](../../../../docs/validation/2026-11-05-picking-cache-recency-qos.md). Chưa bao phủ mọi cache hoặc desktop FPS. |
| Save và hai đường STL bỏ toàn-headroom, admission theo phase; STL capture/worker/permit/session guards và admission peer chung có bằng chứng | [Save](../../../../docs/validation/2026-11-05-scene-picking-save-phases.md), [STL](../../../../docs/validation/2026-11-05-stl-staged-memory.md), [session](../../../../docs/validation/2026-11-04-stl-capture-session-identity.md), [peer](../../../../docs/validation/2026-11-05-stl-shared-admission-peer.md). UI STL vẫn đồng bộ; native peak và scratch còn R2. |
| `om9_memory` đọc tracked/pending/Undo/probes theo yêu cầu; external reserve/grow ghi final refusal thật | [Diagnostics](../../../../docs/validation/2026-11-05-on-demand-memory-diagnostics.md), [refusal](../../../../docs/validation/2026-11-05-external-memory-refusal-provenance.md). Chưa phải inventory toàn RSS hoặc provenance mọi caller. |
| Nguồn/display bất biến chia sẻ; NativeBrep STL lấy canonical source | [Chia sẻ](../../../../docs/validation/2026-11-04-brep-shared-display.md), [độc lập nguồn xuất](../../../../docs/validation/2026-11-04-native-brep-display-export-separation.md). Cache thu hồi độc lập/v5 còn P1. |
| STL có profile riêng, f64 đến encoder, guard f32 T/10, topology checks từng phần và đọc lại staging | [Lượng tử hóa](../../../../docs/validation/2026-11-03-stl-f64-quantization.md), [readback](../../../../docs/validation/2026-11-03-stl-staging-readback.md). Sai số toàn miền, self-intersection, containment/seam tổng quát còn P3a. |
| Query GPU đúng adapter DX12/Vulkan, sửa fallback sai trên RTX 3060 | [Replay release](../../../../docs/validation/2026-11-04-gpu-vulkan-luid-replay.md). Inventory/fence/batching còn P2. |
| Sphere STL giảm peak commit release quan sát từ khoảng 1050 xuống 205 MiB, giữ profile và oracle đã công bố | [Delabella và phạm vi](../../../../docs/validation/2026-11-03-sphere-stl-mesher-memory.md). Một số fixture có điều kiện, không phải upper bound OCCT chung. |
| Root selection có owner CPU sống, candidate/plugin trước publication, removal/drop trả đúng phần còn giữ | [Hai luồng, low-budget và hồi quy](../../../../docs/validation/2026-11-05-live-root-selection-memory.md). Workspace/pending auxiliary vẫn còn R1a; subselection được chốt riêng bên dưới. |
| Subselection slots/path có owner, transfer allocated không đổi tổng/pending, removal/drop trả đúng phần còn giữ | [Hai luồng, capacity/pointer và pressure regressions](../../../../docs/validation/2026-11-05-live-subselection-memory.md). Input/query scratch và auxiliary khác vẫn còn R1. |
| Legacy scene high-water đã bỏ; capture STL yêu cầu scene có owner, growth/Undo dùng registered bytes | [Regression và phạm vi fixture](../../../../docs/validation/2026-11-05-remove-scene-high-water.md). Caller auxiliary snapshot vẫn cần chuyển sang owner sống. |
| FileState path/diagnostics/overwrite destination có owner sống; oracle Open đã tính đúng path được giữ | [Kết quả và log hiện có](../../../../docs/validation/2026-11-05-live-file-state-memory.md): 5 test mới và các nhóm hồi quy liên quan đạt, gồm presentation 13, GPU 29 và native STL 60. Không bao phủ mọi UI/path-copy/decoder scratch; phần đó thuộc R1b/R2. |

Các phần này trở thành ràng buộc hồi quy. Kết quả cũ không tự chứng minh code đang sửa dở đã pass; chỉ chạy lại tập bị ảnh hưởng và chạy rộng tại cổng bàn giao.

Lỗi oracle Open lệch 91 byte đã được xử lý và kiểm chứng; không còn trong checklist triển khai. Tổng các nhóm kết quả cuối của slice FileState: **285 test distinct đạt, 7 ignored**, không cộng các lần retry. Đây là bằng chứng hồi quy trong phạm vi báo cáo, không đóng toàn R1a hoặc gate desktop/độ chính xác STL toàn miền.

## Quy tắc chống loop

1. **Một hàng đợi hiện hành.** PLAN.md là nguồn công việc còn lại duy nhất; PLAN-REVISED.md chỉ trỏ đến đây. Ledger và báo cáo cũ là bằng chứng lịch sử; không nhận lại `next_action` cũ nếu plan hoặc kết quả mới đã thay thế. Lúc tiếp tục, đọc mốc đang dở và phần thay đổi mới, không rà soát lại toàn bộ plan/decompile/spec.
2. **Gom theo chức năng dùng được.** Mỗi lần triển khai chọn một mốc ở bảng dưới, xác định đầu ra, đường production, thiếu sót cụ thể và gate kết thúc. Sửa các owner liên quan trong cùng luồng Open/Save/STL; không biến mỗi Vec/String/lease thành một mốc kèm một vòng hồi quy rộng riêng.
3. **Đã đạt thì đóng trong phạm vi đã chứng minh.** Chỉ mở lại khi có lỗi tái hiện, source/dependency/ABI/profile thay đổi tác động trực tiếp hoặc bằng chứng bị vô hiệu; ghi rõ lý do và đường bị ảnh hưởng. Thiếu desktop/native evidence không là lý do chạy lại mãi các test headless đã đạt.
4. **Audit RAM có điểm dừng.** Chỉ đưa vào đường ưu tiên những allocation/caller làm vi phạm budget, sai lifetime hoặc chặn mốc đang làm và có bằng chứng cụ thể. Metadata nhỏ có bound chứng minh được chuyển R1b; việc còn đó vẫn được ghi nhận. Không yêu cầu inventory bằng toàn RSS hoặc hoàn thiện mọi metadata trước UI STL. Không bỏ charge đang sống hay nới dung sai để đạt gate.
5. **Test hẹp khi sửa, test tích hợp khi chốt mốc.** Dùng bảng gom test dưới đây. Không mặc định chạy lại native STL 60, GPU, Gumball và toàn lifecycle sau từng sửa bộ đếm/owner hoặc sau sửa tài liệu. Chọn theo phụ thuộc thực tế; gate độ chính xác và an toàn dữ liệu giữ nguyên.
6. **Tái dùng kết quả còn hiệu lực.** Mỗi nhóm ghi source fingerprint/phạm vi, dependency, fixture/profile, cấu hình và terminal result. Cùng source liên quan/cấu hình đã đạt thì dùng lại log. Khi có lỗi, chạy lại nhóm lỗi và nhóm bị tác động bởi bản sửa; chỉ chạy rộng thêm khi có lý do cụ thể. Không khởi chạy lại job chỉ vì mất phiên quan sát hoặc hết thời gian chờ; kiểm tra handle/log/process hiện có trước.
7. **Chốt trạng thái cùng đầu ra.** Cập nhật ngay mục đã đạt, phần chưa đạt và bước tiếp theo trong plan/ledger; cập nhật cùng báo cáo của mốc. Không tạo báo cáo rà soát plan mới cho từng sửa nhỏ, không coi số test hoặc số bản ghi là phần trăm tiến độ. Bản ghi lịch sử giữ nguyên bằng chứng đỏ và trỏ sang kết quả thay thế.
8. **Ngắt vòng không tiến triển.** Nếu hai lượt liên tiếp chỉ đọc lại cùng tài liệu, lặp test xanh hoặc nhắc cùng bước tiếp theo mà không giải quyết gate, không lặp thao tác đó lần nữa. Xác định nguyên nhân, chọn hành động cụ thể còn thiếu của mốc hoặc báo đúng trở ngại. Không tự mở rộng audit/refactor để thay thế đầu ra chức năng.

## Ràng buộc giữ nguyên

1. RAM được quản lý dùng trần `floor(60% × RAM vật lý tổng)`. Undo mặc định 1024 MiB, tùy chỉnh tối đa 50% RAM tổng; mức cài đặt là hạn mức giữ lịch sử, không cấp sẵn hoặc cộng toàn bộ vào RAM đang dùng. UI ghi MB với nghĩa MiB = 1.048.576 byte.
2. Kiểm kê đầy đủ dữ liệu Undo được quản lý sau mỗi 100 MiB tăng trưởng được hạch toán, hoặc sớm hơn khi có áp lực/thay đổi quyền sở hữu. Đây là kiểm kê bộ nhớ, không tính lại hình học.
3. Hình học gốc luôn hợp lệ theo dung sai tài liệu ngay lúc commit. Undo eviction và checkpoint không quyết định độ chính xác. Checkpoint mỗi 512 MiB là kiểm chứng trong phiên; không tự lưu, cứu hộ, xóa recipe hay phát lại 256 lệnh.
4. Viewport giữ 100% độ phân giải và khử răng cưa hiện tại. Mục tiêu 30 FPS, dự phòng 24 FPS; không giảm độ phân giải, tự decimate, thêm adaptive LOD hay frame generation để đạt mục tiêu.
5. GPU Auto tối đa 50% VRAM của đúng adapter đang dùng, đồng thời trong ngân sách OS sau dự phòng 10%; cho đặt mức MB thấp hơn. UMA tính vào RAM chung một lần. Chưa xác định được phần cứng thì dùng fallback GPU 256 MiB và báo chưa xác minh.
6. STL chụp nguồn tại lúc bấm Xuất; sửa thường/Undo/Redo sau đó không đổi nguồn đã chụp. Mặc định dung sai tuyến tính từ tài liệu, góc mesher ban đầu 0,3 rad; cho chỉnh riêng. Preset 5 độ và các ngưỡng DeepSeek chưa có bằng chứng không thành mặc định.
7. Hình học, IDs/topology, placement `f64`, tài nguyên và recipe là dữ liệu gốc; mesh hiển thị là dữ liệu dẫn xuất. Export không thay mesh viewport hoặc chia sẻ một OCCT shape đang bị sửa đồng thời giữa các worker.
8. Native v5 tách nguồn/topology khỏi cache. Đọc v4 bằng chuyển đổi có kiểm tra, giữ các đường chuyển đổi cũ; lần lưu đầu phải Save As và giữ file gốc. Không ghi dữ liệu v5 dưới header v4.
9. `gvMeshRepair` là chức năng riêng: chẩn đoán, preview và xác nhận thay đổi hình dạng; không tự sửa/đóng lỗ/gộp object/giảm mesh trong xuất STL thường.
10. Rust sở hữu chính sách, document, cache và worker; native adapter chỉ xử lý API nền tảng/kernel cần thiết. Giữ nguyên phiên OM9 và file đang mở. Build ngoài checkout, `-j 2`.

## Các mốc chức năng còn lại

| Mốc | Việc cần chốt | Điều kiện kết thúc mốc |
|---|---|---|
| A — STL nền: R1a trên luồng export + R2a-STL + P3b | Gom auxiliary/admission/native scratch cần thiết với tích hợp worker UI; chuẩn bị UI trong cùng mốc | Một export busy/cancel/progress, edit/Undo tiếp tục, nguồn/profile đã chụp; memory overlap và lỗi/hủy giữ history/file đích; công bố rõ giới hạn native |
| P1 | Nguồn/display và định dạng v5 | Hình học/IDs/history được giữ, cache độc lập, v4 Save As v5 mở lại đúng |
| P3a | Các oracle độ chính xác STL còn thiếu | Error budget, topology/coverage hai chiều và receipt đúng mức chứng minh; worker không thay dung sai |
| P2 | Ngân sách GPU thật và vẽ đủ theo lô | Đếm tài nguyên thật đến fence, không bỏ object hoặc hạ chất lượng khi quá ngân sách |
| P3c | Xác minh 3DM theo từng đường chuyển đổi | Đúng hình học/trim/đơn vị/IDs theo phạm vi cam kết và publication an toàn |
| P4 | Checkpoint 512 MiB | Đếm đúng phiên bản nguồn, worker ưu tiên thấp, kết quả cũ không gắn vào document mới |
| R1b/R2b — Backlog audit có phạm vi | Gom theo caller/tác vụ có tác động, không chia thành mốc cho từng allocation | Chốt inventory/peak của đường được chọn và giới hạn còn lại; không chặn mốc A bằng mọi metadata hoặc mọi kernel operation |
| P5 | Release/desktop và nghiệm thu tích hợp | Có số đo cùng cấu hình, nêu đủ giới hạn bằng chứng |

Ưu tiên sau khi người dùng tiếp tục triển khai: **mốc A — xuất STL nền dùng được**, rồi P1 và các gate chính xác P3a. R1a/R2a-STL là hạng mục bên trong mốc A, không là chuỗi audit vô hạn trước khi bắt đầu nối UI. Chỉ bàn giao khi capture/admission/cancel của chính đường này đạt gate; không buộc UI STL chờ mọi metadata, codec hoặc đo hết Boolean/fillet/loft. Các gate chính xác hiện có vẫn bắt buộc; chưa gọi STL được chứng nhận toàn miền khi P3a còn mở.

R1b/R2b được chia thành các bàn giao riêng, không thành một vòng kiểm toán không có điểm dừng. P2 và P3c có thể kiểm chứng khi đủ phụ thuộc của chính đường đó; toàn P0 vẫn mở cho tới khi kiểm kê đạt phạm vi cam kết. P3d Mesh Repair giữ AFTER riêng. Mỗi bàn giao ghi caller, lỗi/thiếu sót đã chứng minh, sửa nhỏ nhất cần thiết, test kết thúc và giới hạn; không mở thêm refactor hoặc lặp suite rộng khi không có thay đổi liên quan.

### Gom test theo mốc

Chạy nhóm hẹp sau thay đổi có logic; khi candidate hoàn chỉnh, chạy một batch tích hợp phù hợp để chốt mốc. Sau đó chỉ chạy lại phần bị vô hiệu bởi sửa mới hoặc lỗi. Không đặt chỉ tiêu tổng số test và không lấy tên suite hiện có thay các ca nghiệm thu chưa được viết.

| Mốc | Trong lúc triển khai | Batch tích hợp khi chốt mốc |
|---|---|---|
| A — STL nền/RAM export | Test trực tiếp caller admission, captured source, lifetime hoặc UI event đang sửa; UI worker kiểm busy/cancel/edit và stale completion | Gom một batch File/Document/Undo lifecycle liên quan + capture/phase/worker + STL integration hiện có + UI replay mới cho xuất nền. GPU/Gumball chỉ thêm nếu thay scene publication, picking/input hoặc ownership của chúng; native peak đo riêng cùng fixture/profile |
| P1 — Display/source/v5 | Cache key, nguồn/topology receipt, codec/migration hoặc picking bị tác động | v4 → Save As v5 → mở lại/IDs/history; display-export independence; stale picking/publication. Gumball/GPU chỉ phần phụ thuộc thay đổi |
| P2 — GPU/batching | Budget/capacity/fence và từng pass/invalidation thay đổi | Offscreen nhiều lô đủ object, full-resolution, stale frame, pane unchanged, camera-only counters; picking/Gumball liên quan. Desktop ở P5; không chạy toàn STL nếu export không bị tác động |
| P3a — Độ chính xác STL | Oracle độc lập cho sai số/topology/transform/encoding đang bổ sung | Corpus STL đầy đủ theo profile/source, đọc lại output, coverage/error budget và receipt; không chạy toàn UI/GPU khi luồng đó không đổi |
| P3c — 3DM | Đường chuyển đổi cụ thể, units/trim/seam hoặc lỗi publication | Cold-read geometry/IDs/layers và các roundtrip cần thiết của đường đã đổi; giữ chuẩn báo cáo batch openNURBS riêng |
| P4 — Checkpoint | Threshold/version/cancel/dependency subgraph | Chuỗi edit/Undo/Redo/New/Open trong lúc worker chạy, manifest/stale result và export equivalence |
| R1b/R2b | Test/đo đúng caller hoặc tác vụ đang audit | Đối chiếu ownership/peak và failure atomicity của nhóm đã gom; không tự kèm mọi GPU/Gumball/STL suite |
| P5 — Nghiệm thu cuối | Đóng thiếu sót cụ thể còn lại từ các mốc | Một batch tổng hợp trên candidate release chung, so baseline cùng cấu hình/fixtures và desktop/WPR khi có; báo đủ skips/unknown, không suy desktop từ headless |

Sửa tài liệu chỉ kiểm links/JSON/whitespace và audit nguồn khi cần; không build Rust, chạy GPU/native hoặc benchmark. Bộ STL 60 đã đạt là baseline hồi quy hiện có, không là một batch bắt buộc sau mỗi allocation và không chứng nhận toàn miền.

## P0 — RAM: phần phải sửa và kiểm chứng tiếp

Chủ sở hữu: M01 L2-01-02, L2-01-03, L2-01-05, L2-01-06. Tái dùng coordinator và reservation hiện có.

### R1 — Chốt các owner còn lại và phản ứng khi thiếu RAM

Nền tảng đã kiểm chứng nằm trong bảng hoàn tất phía trên: phase pending/allocated, scene/picking/Weak owners, pressure reclamation, recency, diagnostics và external-refusal. Không viết lại. Giả thuyết incremental buffer lease chain giữ geometry đã mất chưa tái hiện; chỉ sửa khi có bằng chứng. Picking cache tests không chứng minh FPS desktop; close culling hiện bị tắt trên luồng thường.

Hợp đồng RAM dùng ba điều kiện độc lập: (a) `unique_live + pending <= floor(0,60 × total_physical_RAM)`; (b) tổng pending sau request không vượt `Available_RAM - max(256 MiB, 10% total_RAM)`; (c) commit còn trống của Windows đủ pending và dự phòng 256 MiB. Available và commit không cộng thành một ngân sách. Allocation đã tồn tại không cộng lại như yêu cầu mới; chuyển pending → allocated không đổi tổng managed. Số đo OS là ảnh chụp, không bảo đảm residency, RSS hoặc tránh OOM.

Ví dụ máy 16 GiB: trần managed là khoảng 9,6 GiB; Undo mặc định 1 GiB nằm trong 9,6 GiB đó, không cộng thêm. Đây là 60% **RAM vật lý tổng**, không phải 60% RAM đang trống. Lệnh vẫn có thể bị từ chối sớm hơn nếu Available/commit không đủ cho đỉnh phát sinh; không được hạ dung sai hình học để vừa RAM.

### Phân biệt các con số và phần còn thiếu

| Con số/cơ chế | Trạng thái và việc còn lại |
|---|---|
| 256 MiB trong `Limits::default()` | Không còn là trần chung ở các đường tương tác đã kiểm chứng. Audit các caller còn dùng local default theo đúng mục đích; không thay toàn cục vì codec/packet/depth/overflow vẫn cần giới hạn. |
| 256 MiB dự phòng physical/commit | Là khoảng trống an toàn, không phải giới hạn kích thước model. Giữ tách khỏi legacy cap. Physical dùng `max(256 MiB, 10% RAM tổng)`, commit dùng 256 MiB. |
| 256 MiB GPU fallback | Chỉ dùng khi không xác minh được bộ nhớ adapter, không chứng minh card chỉ có 256 MiB; phần triển khai nằm P2. |
| Undo 1024 MiB, cấu hình tối đa 50% RAM | Cấu hình đã có. Code giữ minimum steps nên quota giữ lịch sử là **mềm**: một bước bắt buộc có thể vượt quota. Phải báo vượt quota và phân biệt với trần managed 60%; không gọi 50% là trần thực dùng tuyệt đối đã được thi hành. |
| Kiểm kê 100 MiB / checkpoint 512 MiB | 100 MiB kiểm kê Undo đã có; 512 MiB kiểm chứng hình học còn P4. Cả hai không là điều kiện chờ để geometry trở nên chính xác. |

Phạm vi hạch toán phải được công bố theo owner/caller đã chứng minh. Overhead allocator, driver và OCCT còn thiếu thì ghi riêng, không đặt gate không thể đạt là counter phải bằng toàn bộ RSS. Không thêm một budget toàn app cạnh tranh với coordinator hiện có.

Root và subselection đã có owner và không còn nằm trong capture-time auxiliary; [subselection](../../../../docs/validation/2026-11-05-live-subselection-memory.md), [root selection](../../../../docs/validation/2026-11-05-live-root-selection-memory.md). Không triển khai lại phần này.

- [ ] **R1a — Bộ nhớ sống trong lúc STL và edit cùng chạy, thuộc mốc A.** `StlExportCapture::caller_held_at_capture` vẫn là snapshot auxiliary chưa có owner. FileState, legacy scene high-water, peer, root/subselection đã xong. `auxiliary_usage` còn workspace, recent/repeat commands, pending solid, layer dialog và contract data có thể thay đổi sau capture. Gom kiểm kê/mutation cần thiết của đường export, chuyển phần có tác động sang owner sống trên account chung và bỏ estimate tương ứng để không cộng hai lần. Giữ nguồn export cũ đến reader cuối và tính old/new overlap khi edit/Undo/New/Open. Metadata nhỏ có bound đã chứng minh và không ảnh hưởng gate export được tách R1b; không mở một mốc/test batch riêng cho từng trường dữ liệu.
- [ ] **Gate R1a.** Barrier giữa capture/admission/native/completion: cho auxiliary tăng rồi giảm, thay scene cùng kích thước, Undo/Redo và New/Open khi worker còn giữ nguồn; assert actual live + pending đúng một lần, trả charge sau owner cuối và destination/history không đổi khi thất bại. Bao gồm capture không xuất và worker bị drop/hủy. Không sửa peak giả bằng cách bỏ charge dữ liệu còn sống.
- [ ] **R1b — Owner/caller còn lại, chia từng nhóm.** (1) query/picking scratch và cache containers; (2) coordinator/peer/journal, retired account handles, snapshot input; (3) UI/path-copy/decode/kiểm kê scratch và phần auxiliary có bound được tách khỏi R1a. Không giao làm lại FileState persistent owner đã kiểm chứng. Mỗi nhóm ghi owner, shared identity, reserve → allocated → release, actual capacity và old/new overlap. Không coi danh sách này là lỗi đã tái hiện; native thuộc R2, display thuộc P1, GPU/upload/fence thuộc P2.
- [ ] **R1b — Thu hồi các cache tùy chọn còn lại.** Tái dùng pressure admission hiện có, không thêm sổ RAM thứ ba hoặc bộ quét toàn history mỗi frame. Raw local-Limits APIs chưa tự có global admission. Chỉ trả lease khi bytes hết owner thật; worker/native phải dừng trước. Không hủy export hoặc phá Undo/Redo để cứu giao dịch thất bại.
- [ ] **Undo: kiểm chứng tương tác quota mềm và trần cứng.** Không triển khai lại default/slider/kiểm kê. Test một bước lớn hơn quota, nhiều bước có shared source và cấu hình chuyển sang máy RAM ít hơn. Giữ minimum steps hiện hành; hiển thị requested/effective quota, actual retained bytes và lý do vượt quota. Transaction không vừa trần managed cùng lịch sử bắt buộc thì từ chối trước publication; thất bại không xóa Undo/Redo. Không hứa luôn dành sẵn 1024 MiB. Nếu muốn đổi minimum/soft quota thành trần cứng, đó là thay đổi nghiệp vụ riêng cần trình bày trước.
- [ ] **Không biến các con số cấu hình thành RAM cấp sẵn.** Code account hiện giữ ceiling nhỏ nhất đã quan sát; không giả là đã tự tăng trở lại khi probe phục hồi. Tách RAM tổng (managed/Undo) khỏi Available và commit thay đổi nhanh; không hạ quota Undo mỗi lần app khác dùng RAM. Kiểm chứng mẫu mới, mẫu lỗi, effective ceiling và cấu hình lưu trên máy ít RAM; không sửa file cấu hình âm thầm. Nếu allocation sống vượt ceiling mới, giữ document/history hợp lệ, dọn cache được phép và từ chối tăng trưởng mới không vừa; không giả đã thu hồi bằng cách đổi counter. RAM probe lỗi phải hiện unknown/fallback cùng căn cứ, không dùng số cũ như mẫu mới.
- [ ] **Admission theo đỉnh phát sinh và tác vụ đồng thời.** Lập bảng cho Open/Save/edit/STL: source đang sống, candidate, scratch, output và staging; chia sẻ chỉ tính một lần. Job mới reserve phần tăng thêm trước allocation, tính old/new overlap và pending của peer; không trừ lại live bytes khỏi Available hoặc chặn vì toàn quota Undo/GPU chưa dùng. Giữ một export đang chạy, giới hạn job native nặng có căn cứ; allocation thất bại phải trả lỗi và giữ tài liệu/file đích, không dựa vào việc đã qua preflight để hứa không thể OOM.
- [ ] **Lỗi RAM và diagnostics còn thiếu.** Tái dùng `om9_memory`/external-refusal metadata đã có; mở rộng provenance đúng các caller/job, retained prepare/publication, lazy recheck và codec bounds chưa nối, attribution Document/Undo/owner còn thiếu, tuổi cached Undo inventory. Không coi forecast hay event allocation riêng là verdict của toàn command/native job. Thông báo thường ngắn, phân biệt quota Undo, managed ceiling, physical/commit pressure và giới hạn codec. Log opt-in phải bounded/nonblocking; live diagnostics storage walk hiện đồng bộ cần đo/giới hạn hoặc chuyển worker cho model lớn trước khi đóng gate UI. Giữ trạng thái probe failed/invalid/unknown và không tự coi fallback là đã xác minh phần cứng.
- [ ] **Gate R1 theo tình huống còn thiếu.** Mở rộng concurrent/cold/warm/evicted cache và nhiều snapshot sang các owner mới; New/Open/Undo khi worker còn sống, lỗi/hủy và hai loại pressure physical/commit. Đo workload model đại diện sát trần, attribution đối chiếu private commit/working set và cache churn/latency; ghi chênh lệch thay vì ép bằng nhau. Giữ các regression picking đã có, không chạy lại thay cho đo workload thật. Fixture tài nguyên 270 MiB không thay model CAD/mesh lớn; các large tests chưa chạy giữ mở. Codec/packet 3DM/JSON/pipe cần bounded/streaming theo đường cụ thể, không tăng mọi structural/depth/overflow limit.

### R2 — Reservation theo nhu cầu và native peak

Phase Save/STL đã kiểm chứng nằm trong bảng hồi quy, không triển khai lại. Phần còn thiếu là scratch/capacity chưa được bao phủ, peak native trước callback Rust và tích hợp UI; decoder estimate hiện tại không là chứng nhận peak thực tế.

- [ ] **R2a — phần còn lại của admission:** hoàn thiện hash/control/allocator capacity và phase chuyển pending/allocated cho scratch; không giữ estimate peak của phase đã hết. Đối chiếu old/new/soup/source/topology/staging overlap trên model đại diện. Giữ tests failure/cancel/destination/history đã có; không làm lại entrypoint đã chuyển.
- [ ] **R2a-STL — Native trước callback.** Đo từng phase conversion/meshing/native result/copy sang Rust/topology/encoding của cùng fixture/profile; giữ baseline sphere đã có, thêm trimmed/NURBS/nhiều object đại diện. Rust copy hooks chạy sau khi native đã tạo kết quả; bootstrap một byte không phải task envelope. Không dùng kích thước STL, B-rep input hay hệ số 2×/3× làm upper bound OCCT.
- [ ] **R2a-STL — Chọn admission trước call từ bằng chứng.** Phân biệt estimate có dự phòng với giới hạn thực thi được. Nếu native call chưa hỗ trợ cấp theo chunk/allocator quota, ghi rõ phần chưa chặn; giới hạn số tác vụ nặng đồng thời trong Rust và giữ reservation đến native return/worker join. Process isolation chỉ chọn cho đường cần và có thể thực thi trần cứng, kèm đo overhead, truyền nguồn và bảo toàn output; không mặc định triển khai lại mọi kernel call.
- [ ] Nối worker tự cấp theo phase vào UI theo P3b; entry `StlExportJob::start` nhận fixed reservation hiện có vẫn giữ hợp đồng cũ. Backend hai luồng pass không chứng minh UI hết bị chặn. Giữ một export đang chạy và ưu tiên interaction → export → checkpoint.
- [ ] **R2b — bàn giao theo tác vụ, không chặn R2a:** đo release riêng cho tessellation/STL, Boolean, fillet và loft với fixture nhẹ/nặng/lặp, ghi peak commit và working set riêng cùng thời gian. Kết quả fixture trước đây chỉ là quan sát, không thành hệ số cấp phát 2×/3× hay upper bound chung.
- [ ] Native OCCT không tự tuân theo `Usage`; `bad_alloc` không bảo đảm sống sót khi OOM toàn tiến trình. Cancel đang chờ native phải báo đúng trạng thái và không trả slot/lease sớm; ưu tiên interaction → export → checkpoint không có nghĩa preempt được OCCT đang chạy.
- [ ] GPU UMA/shared memory vào RAM chung một lần; không cộng lại cùng vùng nhớ ở cả sổ RAM và GPU. Dedicated VRAM có ngân sách riêng nhưng CPU staging vẫn tính RAM.
- [ ] Gate P0: giới hạn/quota đúng, chia sẻ đúng, transfer không hở, failure/cancel không đổi tài liệu/Undo/Redo/file đích. Có fixture thật >256 MiB và concurrent/low-RAM tests phù hợp, ghi rõ giới hạn native và RSS.

## P1 — Phần còn thiếu của display cache và v5

- [ ] Đưa display snapshot hiện vẫn nằm trong `Brep`/archive v4 ra cache dẫn xuất có thể thu hồi độc lập. Giữ chia sẻ nguồn/display bất biến đã kiểm chứng; không viết lại cơ chế Arc chỉ để đổi tên. Giữ placement, IDs/topology, tài nguyên và recipe trong document/Undo.
- [ ] Khóa mesh có source identity/generation/revision, Display/Export, profile dung sai và kernel version. Camera/selection không tạo mesh mới. Chỉ đưa thuộc tính presentation thực sự ảnh hưởng dữ liệu vào khóa.
- [ ] Mesh có receipt chứng minh nguồn và ánh xạ topology; triangle index không phải stable face ID. Từ chối mesh/pick thuộc nguồn hoặc topology cũ.
- [ ] Tách native triangulation state và snapshot sở hữu giữa display/export; chứng minh concurrency/lifetime, không dùng chung mutable OCCT shape.
- [ ] Native v5 lưu nguồn/topology riêng cache. Chuyển v4 trong bộ nhớ có kiểm tra, Save As đầu tiên giữ bytes file gốc, báo bản cũ không đọc được v5; test đọc lại và đối chiếu geometry/IDs/history, giữ migration cũ và từ chối version/corruption lạ.
- [ ] Gate: đổi chất lượng display không đổi canonical measurements hoặc STL cùng export profile; source swap và picking cũ bị từ chối.

## P2 — GPU budget, invalidation và vẽ theo lô

- [ ] Hoàn tất cấu hình Auto/mức MB thấp hơn trên renderer thường và lưu file GPU riêng, giữ bộ đọc `undo.json`. Tái dùng query đúng adapter DX12/Vulkan đã có, bổ sung refresh OS budget và phản ứng khi budget đổi; không chọn GPU theo tên.
- [ ] Admission toàn bộ GPU capacity: geometry/instance/camera/overlay buffers, staging, pipeline owners liên quan, mọi target màu/depth/MSAA, old/new overlap và tài nguyên đến queue completion. Đánh dấu nonresident không phải đã giải phóng.
- [ ] Hoàn tất các đường invalidation/upload chưa được bao phủ; giữ cơ chế tái dùng geometry/pipeline/completed texture và world lines đã có. Bổ sung counter/fixture cho selection, placement, visibility, geometry, material và overlay; selection không upload lại toàn scene chỉ để highlight. Không triển khai lại scheduler hoặc cache chỉ vì gate GPU tổng chưa đóng.
- [ ] Scene vượt ngân sách phải vẽ đủ theo lô vào target full-resolution, giữ depth và pass order, overlay sau cùng, công bố khi hoàn chỉnh. Giới hạn submissions/staging, dùng completion/fence bất đồng bộ. Target cộng lô tối thiểu không vừa thì giữ ảnh hoàn tất gần nhất và báo lỗi; không bỏ object hoặc hạ chất lượng.
- [ ] Mỗi viewport một frame captured đang dựng/submitted và một camera mới nhất chờ. Camera đổi không liên tục hủy frame đang dựng; source/document/selection/preview/CPlane/display/size/DPI đổi phải làm kết quả lỗi thời không được công bố.
- [ ] Picking dùng camera/source/topology receipt của ảnh đã công bố và kiểm tra với document hiện hành. Gesture Gumball đã bắt tiếp tục theo nguồn hợp lệ, không phụ thuộc worker GPU kịp chuẩn bị.
- [ ] Mesh chỉ tạo candidate cho exact CAD pick. Refine canonical face/edge khi hỗ trợ, tách screen tolerance và document tolerance, báo hoặc từ chối khi thiếu mapping/refinement thay vì gắn nhãn exact.
- [ ] Gate còn lại: camera-only sau ổn định không tessellate/upload geometry/tạo pipeline; chỉ pane đổi encode lại. Đối chiếu pixels + canonical geometry sau eviction/resize/Undo/Redo/New/Open, coarse mesh/tiny features/overlap và stale completion. Dùng lại regressions Gumball hiện có; desktop là cổng riêng.

## P3a — Hoàn tất kiểm chứng STL

Đã có f64 → encoder, ngân sách lượng tử hóa T/10 và ca ba trục gần 100.000 mm; không còn là việc cần triển khai. [Bằng chứng](../../../../docs/validation/2026-11-03-stl-f64-quantization.md). Đã có đọc lại staging, count/length/hash và decode finite/area/normal-winding từng tam giác; phần thiếu là kiểm chứng hình học/topology toàn mesh, không phải viết lại atomic writer. [Bằng chứng](../../../../docs/validation/2026-11-03-stl-staging-readback.md).

- [ ] Hoàn tất error budget trong tọa độ xuất: nguồn/kernel, tessellation, chuyển đổi NURBS, đơn vị, scale không đều, placement, seam remap và f32. Đo interiors/edges và coverage hai chiều, không chỉ đỉnh nằm trên nguồn hoặc số tam giác.
- [ ] Mở rộng fixture lượng tử hóa/phép biến đổi ngoài các ca đã pass: đơn vị/scale không đều/mirror, chi tiết rất nhỏ và triangle inversion; giữ guard hiện có. Kết hợp với tổng error budget thay vì coi T/10 riêng lẻ là bảo đảm tổng T.
- [ ] Bổ sung receipt phần kernel/version và mức chứng minh còn thiếu; giữ captured source/profile, giả định đơn vị, requested tolerance và cảnh báo deviation/self-intersection đã có. Sampling phải ghi phương pháp/coverage/max quan sát/gaps; bounded cần ngân sách lỗi và phương pháp đủ toàn domain, unknown không thành pass. [Receipt hiện có](../../../../docs/validation/2026-11-03-stl-receipt-scope.md).
- [ ] Mở rộng corpus kiểm chứng độc lập cho các đường canonical B-rep/NativeBrep/NURBS đã có: prong/milgrain/filigree, trims, seams, hốc, units, scale không đều và mirrors. Giữ oracle display/export independence, không viết lại tessellation từ nguồn. Delabella tiếp tục giới hạn preconditions đã kiểm chứng; mesh tác giả so với chính bề mặt mesh gốc.
- [ ] Hoàn tất phần chưa có của topology/encoding gate: self-intersection kể cả coplanar, containment/cavity orientation tổng quát, và khác biệt giữa topology trước/sau encoding. Giữ checks boundaries/incidence/edge direction/fans/duplicates/degenerates/components đã có; bổ sung fixture độc lập, không gọi incidence hai là chứng minh kín hay không tự cắt. [Giới hạn đã tái hiện](../../../../docs/validation/2026-11-04-stl-intersecting-closed-components.md).
- [ ] Weld theo source-edge/topology identity có kiểm chứng; không weld theo proximity đơn thuần hoặc gộp object. Phân biệt seam đúng với shells gần nhau trong cùng nguồn, khe hẹp, thin wall, coincident objects. Mapping không đủ thì giữ distinct hoặc báo unknown theo hợp đồng export hiện hành.
- [ ] Bổ sung oracle encoded output cho coverage, outer/cavity shells và seam/reflection/quantization tương tác, dùng lại staging decoder hiện có. Surface hở vẫn xuất kèm cảnh báo; không gọi print-ready chỉ từ closedness hoặc digest đúng.
- [ ] Giữ các oracle và checks đã có; bổ sung phần chưa bao phủ. BBox nhỏ hoặc triangle altitude không phải wall thickness; không bật mặc định 0,2 mm hay 2×T khi chưa có bằng chứng false-positive.

## P3b — Xuất STL nền dùng được trong UI

Không triển khai lại capture-at-invocation, worker, permit, session token hoặc atomic cancel/rename đã có. Phần còn lại là đưa nền tảng đó vào đường UI hiện vẫn gọi xuất đồng bộ và cấp ngân sách đúng nhu cầu.

- [ ] Nối UI vào `StlExportJob` với admission theo R1/R2; giữ source/selection/placement/units/profile đã chụp. Dùng chung single-export permit hiện có, bổ sung busy/cancel/progress UI và nhận kết quả không chặn.
- [ ] Kiểm thử đầu-cuối UI cho edit/selection/Undo/Redo trong lúc export; giữ captured source. Nối New/Open/close và nhận kết quả vào session guards đã có, kể cả mở lại cùng UUID và hủy đóng cửa sổ; không công bố completion cũ. Không tự chụp lại nguồn hoặc nhả permit/lease khi native còn chạy.
- [ ] Giữ staging file và atomic replacement hiện có. Bound/chunk tessellation, soup và topology scratch nơi hỗ trợ; chỉ stream writer chưa đủ giới hạn peak toàn export. Lỗi/hủy/stale không làm đổi file đích.
- [ ] Gate: edit/Undo khi xuất vẫn ra đúng revision đã chụp; cancellation và document-replacement races; UI không bị khóa; đỉnh RAM và reservation không chiếm toàn headroom vô căn cứ.

## P3c — Kiểm chứng 3DM còn thiếu

Truyền dung sai tài liệu, kiểm tra metadata từ staging cold-read và atomic publication đã có bằng chứng; không triển khai lại các bước này. [Bằng chứng](../../../../docs/validation/2026-11-03-3dm-conversion-tolerance.md). Phần còn thiếu là oracle hình học theo từng đường chuyển đổi.

- [ ] Giữ việc truyền dung sai tài liệu/metadata đã có; xác minh riêng native-preservation, editable B-spline và analytic-to-NURBS. Metadata tolerance hoặc `alreadyNurbs` không chứng minh độ chính xác hình học hay byte preservation.
- [ ] Bổ sung đối chiếu geometry trong đường cold-read staging hiện có với đúng captured/require-current source của đường tương ứng. Kiểm interiors, trim boundaries, periodic seams và cavity containment trong physical units, có reparameterization; không chỉ so metadata.
- [ ] Test dung sai/đơn vị khác mặc định, tọa độ lớn, mirror/nonuniform scale, IDs/UUID/layers, corrupt/unsupported payload và stale source; lỗi giữ file đích. Chỉ yêu cầu CV/trim hoặc byte equality ở đường có cam kết giữ representation đó.
- [ ] Acceptance theo captured tolerance và error budget có căn cứ; số 1e-9/1e-4 mm trong review không thành ngưỡng chung. Phân biệt sampled/bounded và phần chưa kiểm.

## P3d — Mesh Repair: bàn giao AFTER riêng

Theo [plan gvMeshRepair](../../AFTER/gvmeshrepair-20261103/PLAN.md), owner `OM9-L2-10-02` và AFTER L3 tương ứng. Không coi engine Netfabb là thuật toán đã biết từ wrapper decompile.

- [ ] Chẩn đoán bất biến, không đổi geometry/history: pass/fail/unknown cho boundary loops, non-manifold edges/vertices, winding/cavities, duplicates/degenerates, components, self-intersection kể cả coplanar. Timeout/budget exhausted là unknown.
- [ ] Tools icon và CMD dùng chung command/default/shortcut contract; report/preview gắn source revision, worker có cancel/budget. Source đổi làm preview stale.
- [ ] Repair giữ input mặc định; centroid fan, quantized weld và bounded CSG hiện có chỉ dùng đúng preconditions đã kiểm chứng. Phân biệt seam có nguồn với bề mặt dựng mới, preview và xin chấp nhận thay đổi hình dạng/union; không xóa chi tiết nhỏ âm thầm.
- [ ] Reduction ưu tiên remesh nguồn CAD nếu có; mesh tác giả so với chính mesh gốc. QEM cost hoặc target face count không chứng minh sai số. Hai chiều deviation/topology/f32 đạt mức đã tuyên bố mới chấp nhận; không ép giảm theo phần trăm/kích thước file.
- [ ] Fixture và baseline riêng: sphere/box/torus, holes/concave/nonplanar loops, thin wall/prongs, nearby shells/components, self-crossings, seams/trims và mirrors/scale. Đo triangles, `84 + 50N` bytes, commit/working set/time; so cùng dung sai. Vùng vá không có nguồn đối chứng không được chứng nhận khớp hình gốc chưa biết.
- [ ] Cancel/Undo/Redo/source-race và encoded-output checks đầy đủ theo phạm vi; không gọi verified watertight khi self-intersection/topology còn unknown. Repair chỉ commit một giao dịch thành công; không đưa repair/reduction ngầm vào STL thường.

## P4 — Checkpoint kiểm chứng 512 MiB

- [ ] Đếm payload canonical mới được commit theo phiên bản nguồn; cùng kích thước thay thế vẫn tính. Chia sẻ tính một lần theo phiên bản; camera/selection/cache không tính, Undo/Redo không tính lại nguồn đã gặp.
- [ ] Chạm ngưỡng gộp một job cho dependency subgraph bẩn mới nhất, kể cả một lệnh vượt nhiều ngưỡng. Ưu tiên thấp nhất; không chặn interaction/export và không giữ thêm bản sao geometry.
- [ ] Worker kiểm nguồn bất biến theo tolerance, manifest gồm IDs/units/source/topology/resources/history/kernel/profile. Kiểm generations trước khi gắn kết quả; stale/failure không đổi document/checkpoint cũ hoặc xóa tăng trưởng chưa kiểm chứng. Mốc chỉ tiến khi thành công.
- [ ] New/Open/reopen hủy job và reset trạng thái checkpoint trong phiên. Save/reopen giữ geometry/history, không nhận là cứu hộ. Export kiểm nguồn riêng dù chưa đạt ngưỡng và không chờ checkpoint.
- [ ] Gate: dưới/đúng/vượt ngưỡng, lệnh lớn, cancel, edit-during-worker, Undo/Redo, manifest và export equivalence. Counter mô phỏng chỉ chứng minh trigger; claim RAM runtime cần fixture thật.

## P5 — Nghiệm thu và cách thực hiện

- [ ] Một bảng acceptance còn mở theo mốc ở trên: RAM ownership/pressure, display/export independence, GPU completeness/stale publication, STL geometry/topology/encoding, captured export, 3DM, v4/v5 và checkpoint. Một test pass không đóng cả nhóm.
- [ ] Trước thay đổi hiệu năng tiếp theo, ghi baseline rồi đo sau bằng cùng release/fixture/GPU/DPI/window; nhẹ, nặng, lặp. Báo median/p95/p99, frame trễ, CPU/GPU/input-to-submit, số tessellate/upload/target/pipeline và peak bộ nhớ. Không loại frame dài khỏi thống kê chính mà không giải thích.
- [ ] WPR/PresentMon chỉ suy luận đến phạm vi đo được; Present-call interval hoặc GPU completion không phải thời điểm ảnh lên màn hình. Desktop input-to-display/chuột thật phải kiểm riêng, không kết luận mượt từ headless.
- [ ] Chốt batch test theo bảng gom mốc và quy tắc chống loop; giữ log còn hiệu lực, chỉ retry nhóm lỗi hoặc bị thay đổi tác động. Ghi build/test elapsed, fixture, skips và giới hạn bằng chứng; không suy số giờ còn lại từ số test hoặc tổng thời gian trước.
- [ ] Build target ngoài checkout, `-j 2`, đủ DLL; giữ phiên người dùng, audit public source, prune artifact theo quy định và cập nhật ledger/milestone theo bằng chứng. Không tự publish/commit dữ liệu riêng hoặc thay skill nghiệp vụ nếu chưa có đồng ý phù hợp.

Không có ước tính thời gian còn lại đủ tin cậy. Báo mỗi mốc sau khi kiểm chứng đầu-cuối; checklist này không thay đổi tiêu chí chính xác hoặc thu hẹp phạm vi dự án gốc.
