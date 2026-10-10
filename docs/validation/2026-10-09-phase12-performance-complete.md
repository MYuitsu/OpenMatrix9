# Phase1/2 — tối ưu bounded đã kiểm chứng

Hai thay đổi product trên runtime riêng: Phase1 JSON request chỉ bỏ whitespace; Phase2 safe Rust chia sẻ candidate bất biến bằng Arc và tra cache theo object. Giữ geometry/schema/API/ABI, Undo/Redo/Cancel, mode/residual/generation, native kernels và policy floor60% logicalCPUs (24→14, cap tasks/RAM).

[Bản chạy](H:/FreeCAD-src/build/launch-om9-perf.cmd), source `H:\FreeCAD-src\build\om9-perf-dev`, runtime `H:\FreeCAD-src\build\om9-perf-sdk`. Manifest SHA256 `5d06e5d1fd9070a14afb69800aa53f99c6001ddc1dc4bc159959f3e7476eac6d`, module SHA256 `187f5c55baf2ca715409194464c76ea938c919d35ade6f3781da58cb43d3835a`. Accepted baseline source/runtime được verify lại nguyên vẹn.

## Hiệu năng đo được

| Phép đo | Baseline | Candidate | Phạm vi |
|---|---:|---:|---|
| JSON request mesh20k | 2.285.668bytes | 2.025.652bytes | nhỏ hơn11,38%; schema/geometry nguyên vẹn |
| Encode+native archive median7pairs | 0,888146s | 0,895579s | chênh nhỏ/no consistent wall gain |
| Actual Copy median5pairs | 1,109484s | 1,094175s | biến thiên~1,4%,không tuyên bố gain chắc chắn |
| Actual Paste median5pairs | 0,449506s | 0,450186s | gần như ngang nhau |
| Rust cache4096 median6pairs | 42.450µs | 0.700µs | 60.6× core-only,one nearobject |
| Rust dense8192 median6pairs | 92.050µs | 75.300µs | 18.2% core-only |

Same-process alternating A/B6pairs/workload,50warmups+1000measuredqueries/mode/pair. Canonical selectedpoint/count/completeness asserted everyquery. Actual native JSON/OLE A/B8archivepairs+6CopyPastepairs,firstpairwarmup; every archive/paste reimport verifies transformed20kmesh. Successful run used0 retries. Initial busy-clipboard attempt retained; test-driver retries5×20ms only for explicit busy error, product fail-closed unchanged. No extrapolation to whole CAD app or assertion that all Python should be converted to Rust.

RAM Windows private bytes sampled at synchronized owned-process checkpoints,3alternatingpairs/workload. Cache4096 ready median baseline9,051MiB→candidate9,676MiB (+0,625MiB lookup/Arc metadata). Dense8192 ready1,367→1,434MiB; query consumed1,820→1,406MiB. This measures process allocator footprint, not zeroallocation/leak proof. [RAM raw](phase12-performance/snap-memory.json).

Warm host timings below from sequential rerun after heavy native/regression/Rhino work finished; initial overlapped run remains raw functional evidence, excluded from timing acceptance:

- `modeling_snap_performance_smoke.FCMacro`: {'queries': 500, 'p95_ms': 0.02180004958063364, 'median_ms': 0.019599974621087313, 'last': {'candidates': [[0, 0, 0], [10, 0, 0]], 'complete': True, 'elapsed_us': 2, 'generated_candidates': 0, 'index_objects': 0, 'index_rebuilt': False, 'max_per_object': 2, 'picked': False, 'point': [0, 0, 0], 'read_cloud_points': 0, 'read_mesh_vertices': 0, 'reason': '', 'state_owner': 'rust', 'viewport_height': 382, 'viewport_width': 747, 'visited_objects': 1, 'visited_topology': 2}}
- `modeling_snap_heavy_smoke.FCMacro`: {'p95_ms': 0.02340000355616212, 'median_ms': 0.021099986042827368, 'queries': 500}

Warm timings are not a paired app-level speed comparison; cold document/camera index latency unmeasured. Existing heavy-access0 counters are literal/source-level exclusions, not independent instrumentation.

## Kiểm chứng đúng runtime

Rust114/114; tools60/60; native50/50. **38 unique FreeCAD reports/769 checks**:22 main gates (A/B1+targeted6+regression13+performance2),2SaveAs rereads and14Rhino child host reports. Actual Rhino5 Phase2:3fixtures125checks,host35,saved55; actual Rhino5 clipboard:11fixtures496checks,host218,saved120. These counts describe scoped gates, not percent full-openNURBS or new batches per assertion. Fixture/report/executable/source/runtime SHA, PID and exit0 are bound; [machine summary](phase12-performance/summary.json) names every report.

Fresh module build succeeded; fresh native suite50passed. Rust changed file rustfmt passes, Clippy exits0 with inherited11lib+4test warnings outside changed product file. Native inherited deprecated OCCT and optional missingVulkanheader warnings; no build error. Whole-diff fresh-context review:0Critical/0Important/0Minor, no declined behavior; [review](phase12-performance/final-review.md). Reviewer read existing evidence, did not independently reproduce GUI/tests; executor finished missing clipboard-reread after review.

## Trạng thái và quyết định

Đợt tối ưu bounded hoàn tất;0batch đã chuẩn bị còn chờ.7gói fullopenNURBS vẫn mở; tổng batch tương lai chưa biết. Phase3–5/full openNURBS không được nâng mức hoàn thành từ các phép đo này. Bản cũ được giữ để đối chứng; bản mới chạy bằng launcher riêng, chưa integrate vào main/commit/push.

Rulings theo thứ tự trong ledger: (1)brief bounded đã duyệt thay formal architectural plan—không đổi interfaces; nếu sai phải thiết kế lại phạm vi. (2)Git pointer hỏng dùng source/runtime copies+hash, không sửaGit—nếu thiếufile sẽ làm test hoặcbind fail, không có commit snapshot. (3)Python/PowerShell chỉ giữ host/test glue cần thiết, product cacheownership dùng safeRust—nếu xác định nhầmhotpath còn chi phíPython. (4)busyclipboard retry chỉ ở testdriver,reason/counter/timing được ghi—nếu reasonmatch sai có thể che lỗi, nên chỉ exactbusy. (5)giữ workspace/evidence do không có Gitcommit lưu thay thế—nếu xóa mất khả năng truy xuất. Không có minor mới từ review; UI explanation localCV minor của Phase2 cũ vẫn ngoài scope đợt performance.
