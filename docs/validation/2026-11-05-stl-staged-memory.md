# STL: admission theo giai đoạn trên tài khoản RAM chung — 2026-11-05

Phạm vi: bỏ reservation toàn headroom trên hai đường Controller/captured và detached STL; nối kiểm tra allocation hiện có vào tài khoản production. Không hoàn tất toàn R1/R2 hoặc plan viewport/export. Không mở/đóng OM9 hoặc sửa bản vẽ người dùng.

## Thay đổi

Hai entrypoint dùng chung `export_staged` và pipeline `export_with_admission`. Export vẫn có một permit chung đến khi thành công/lỗi/hủy. Admission ban đầu lấy lower bound các mảng mesh tác giả đã biết, hoặc một byte bootstrap cho nguồn kernel; đây không phải upper bound tác vụ. Các bước tiếp theo grow/shrink/materialize trên registry hiện có, không tạo ledger RAM thứ ba và không lấy toàn phần còn trống.

Known buffers được tính theo capacity: soup f64/u32 và old/new realloc, receipt strings, nguồn chuyển đổi NativeBrep/NURBS, snapshot B-rep và groups/bindings, remap, output buffers, writer 8 KiB và temporary path. Snapshot/nguồn kernel có callback trước copy sang Rust và sau copy để đối chiếu capacity. Core Error từ admission được giữ khi đi qua kernel error wrapper; panic trong callback C ABI được bắt tại Rust adapter, không unwind qua C++. C++/ABI không thay đổi. Đây là Rust ownership/policy trong adapter hiện có, không chứng nhận native memory safety.

Topology và quantized-position table dùng callback known-slot estimate hiện có. Soup/notes/path vẫn tính khi scratch sống. Scratch được bỏ khỏi allowance sau khi phase trả về; không giữ allowance mesh/topology suốt đến hết file. Cửa sổ admission 64 KiB giảm việc quét owner/probe từng scalar; khi xin dư cửa sổ bị Budget, thử đúng số cần. Dữ liệu cũ và mới được tính cùng lúc ở các ranh giới realloc đã nối. Selection IDs đã được tạo được chuyển từ pending sang allocated.

Không thay deflection tuyến tính/góc, tolerance scaling, mesher, source-edge weld, orientation, topology checks, guard f32 T/10 hoặc staging readback/publication. Raw test/legacy fixed-reservation worker wrappers vẫn dùng cùng pipeline với local Limits và guard do caller cung cấp. UI vẫn gọi xuất đồng bộ; chưa nối UI sang worker tự cấp ngân sách theo phase.

## Giới hạn còn mở

- Native OCCT/openNURBS có thể cấp phát trong conversion/meshing **trước** khi callback Rust nhận lengths. Hooks copy không chặn được peak đó. Bootstrap một byte không là native task envelope. Cần admission/concurrency/isolation có căn cứ ở R2b; không coi giới hạn `max_bytes` của adapter hay `bad_alloc` là RSS/native cap.
- Topology/hash scratch vẫn là known-slot estimate pending trong phase, chưa phản ánh mọi allocation đã sống; hash control bytes, allocator over-allocation và metadata chưa đủ. Actual capacity có phần chỉ biết sau `try_reserve`; có thể phải từ chối/dọn kết quả sau allocation. Không tuyên bố managed 60% bao phủ mọi byte trong tiến trình.
- Target/path/worker plumbing, một số validation scratch, auxiliary/high-water estimates và receipt/result giữ qua worker completion còn phải audit. Không trả lease sớm cho native call còn chạy; cancellation trong native call vẫn chỉ có hiệu lực tại ranh giới được hỗ trợ.
- Chưa có thu hồi cache tự động, pha worker UI mới, release/desktop/WPR, hoặc đo peak mới cho fixture CAD lớn. Thử nghiệm cạnh tranh giữ reservation 16 MiB, không cấp phát 16 MiB payload.
- Sai số toàn miền STL, self-intersection/containment tổng quát, 3DM fidelity, GPU batching, v5 và checkpoint vẫn theo các cổng mở của plan.

## Kiểm chứng đã chạy

Windows x64; root workspace, Cargo default features; `CARGO_TARGET_DIR=E:\om9-surface-snap-validation`, `-j 2`, dev debug=0/incremental=0; PATH gồm OCCT 8.0.1 SDK, TBB và jemalloc runtime của checkout. Không thay SDK/backend hoặc profile dung sai để chạy test.

Batch `build/standalone-validation/ui-artifacts/stl-phases-20261105-031935`, script `build/standalone-validation/run-stl-phases-20261105.ps1`: source admission 2, snapshot copy 4, STL lifecycle 46, orientation 10, STL integration 60 pass. Integration chạy 112,62 s, build + run 135,59 s. Sau đó chỉ bổ sung test hai luồng, assertion về scene growth/selection phase và typed copy refusal; không đổi production pipeline sau lượt integration này.

Kết quả cuối theo đúng nhóm, không cộng lại các lần lặp:

| Nhóm | Bằng chứng terminal | Kết quả |
|---|---|---|
| Toàn kernel unit | `build/standalone-validation/ui-artifacts/stl-phase-followup-20261105-032441/kernel-lib.log` | 17 pass, gồm native snapshot/group/remap và hai test mới |
| Toàn STL unit sau test cuối | `build/standalone-validation/ui-artifacts/stl-phase-followup-20261105-032441/stl-lib-final.log` | 58 pass: 46 lifecycle, 11 orientation, 1 quad |
| STL integration với OCCT/openNURBS thật | `build/standalone-validation/ui-artifacts/stl-phases-20261105-031935/stl-integration.log` | 60 pass |
| Document/file/ordinary RAM | `build/standalone-validation/ui-artifacts/stl-phase-followup-20261105-032441/lifecycle.log` | 12 + 20 + 9 pass, 6 large ignored |

176 test pass trong các nhóm cuối; đây là số test, không là số fixture độc lập hoặc phần trăm hoàn thành. Không chạy lại 6 large cases; kết quả resource 270 MiB trước đó không thay chúng. Ma trận tương lai chưa liệt kê hết, số batch còn lại chưa xác định.

Oracle bổ sung:

1. Luồng xuất dừng bằng barrier ngay khi có lease; luồng chính thực hiện Point, Undo/Redo và chọn lại nguồn, đồng thời giữ reservation khác 16 MiB dưới limit 32 MiB. Luồng xuất tiếp tục thành công từ nguồn đã chụp. Controller entrypoint xuất cùng nguồn/profile ra bytes giống hệt, peer capacity không bị thu hồi. Đây là kiểm tra hai luồng backend, không chứng minh UI đã chạy export nền.
2. Sau initial admission, peer chiếm phần còn trống làm growth bị từ chối. Destination bytes, document snapshot và Redo được giữ; permit và lease được trả đúng để thử lại.
3. Scene tăng trong lúc admission làm phần còn khả dụng giảm qua registry, thay vì tăng lease export cho hết headroom. Capture mới không thừa kế peak đã hết owner.
4. Native source copy bị từ chối trước khi tạo Rust buffer; callback panic được bắt. Snapshot được chấp nhận có points/triangles/shell groups giống wrapper tham chiếu. Core Budget đi qua kernel callback vẫn giữ loại lỗi và output soup chưa cấp phát.
5. Corpus integration giữ các oracle hình cầu (mọi triangle và sampled surface points), B-rep/display independence, NativeBrep canonical payload, NURBS trims/periodicity, unit/scale/mirror, tiny triangles, quantization, topology và destination preservation đã có. Sampling không thành chứng nhận global deviation.

Compile lần đầu phát hiện lifetime callback dùng chung với context; đã tách lifetime của borrow context và callback. Một lần check thiếu import `Usage`, đã sửa. `cargo check -p om9-app -j 2` và các test sau đó pass. Không coi lượt compile lỗi là runtime pass.

## Bàn giao

Plan/alias, README, ledger và runtime entries M01 L2-01-03/05/12 cùng L3-01-12-008 được đồng bộ theo phạm vi. Các yêu cầu đã chốt trong entrypoint không còn là việc viết lại; envelope native, phase/hash inventory, automatic cache pressure và UI worker vẫn mở. Bước tiếp theo: nối eviction/re-admission ở caller thực, rồi đo/admit native peak và tích hợp worker UI theo plan.

Kiểm tra cuối: fingerprint khớp bốn file dưới, terminal summaries khớp 176 pass/6 ignored, entry ledger duy nhất, ba owner JSON/L3 JSON và liên kết local hợp lệ. Kiểm tra source xác nhận các lời gọi remaining-headroom trong file STL đều nằm trong module test áp lực. Public-source audit: 15.178 source files, 594 icon bindings, 593 SVG, không có lỗi. Chạy prune theo tool dự án sau kiểm resolved targets trong validation root; giữ run được dẫn và không xóa target ngoài checkout. Không commit/publish hoặc đổi skill nghiệp vụ.


Source fingerprint SHA-256: `a5342a43fac840207497b1d1f775f636a3eb5fc7662adfff99c243f3d05eedf5`. Nối lần lượt UTF-8 repo path, NUL, bytes file gốc theo thứ tự dưới; không phải Git commit hoặc hash toàn checkout.

- `crates/om9-app/src/session/stl.rs`
- `crates/om9-kernel/src/brep.rs`
- `crates/om9-kernel/src/measurement.rs`
- `crates/om9-app/tests/stl_export.rs`
