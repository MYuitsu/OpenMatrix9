# Checkpoint trước khi đổi thứ tự triển khai lõi

Đây là lịch sử từng thời điểm, không phải trạng thái hiện tại.

MatrixApp Boolean Two Objects đã dựng năm chế độ preview trên worker, giữ preview cũ khi chờ; fixture kiểm XOR, preview rỗng, stale/quota và Undo/Redo. Document/Undo và C1–C5 vẫn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-boolean-two-ui-worker.md).

MatrixApp Boolean Split qua menu/CMD đã dùng worker nhiều tập và commit một transaction; fixture headless kiểm trùng tập, placement affine, DeleteInput, Undo/Redo, stale và quota. Boolean Two Objects và C1–C5 vẫn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-boolean-split-ui-worker.md).

Split có bước chia từ lease nguồn và trích mảnh với admission; fixture hộp kiểm mảnh, thể tích và hủy. Vòng lặp nhiều tập cùng UI/transaction chưa nối worker. [Bằng chứng](../../../../docs/validation/2026-11-14-split-captured-source.md).

MatrixApp Difference và Intersection qua menu/CMD đã dùng worker nguồn cho hai nhóm; fixture hộp kiểm DeleteInput, Undo/Redo, giao rỗng, stale và quota. Boolean Split/Two Objects và C1–C5 còn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-boolean-binary-ui-worker.md).

MatrixApp Boolean Union qua menu/CMD nay chuẩn bị nhiều nguồn và placement trên worker, kiểm context trước transaction; fixture headless ba hộp, Undo/Redo và từ chối stale/quota đạt. Difference, Intersection, các route khác và C1–C5 còn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-boolean-union-ui-worker.md).

Boolean đã có API worker dùng lease nguồn và fixture xác nhận giải phóng document/display, admission và hủy; menu/CMD sản phẩm chưa nối worker. C1–C5 vẫn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-boolean-captured-source.md).

Ngày 2026-11-13. **Trạng thái: PLAN, CHƯA TRIỂN KHAI.** Người dùng yêu cầu rà đủ M01–M09 và lập kế hoạch xử lý phần core còn thiếu; yêu cầu này không tự khởi chạy implementation. Các bàn giao đã có trong goal được tái dùng, không làm lại toàn bộ Curve/Solid hoặc toàn M04–M08.

Tiến độ sau khi bắt đầu implementation: Surface Snap của Line có worker và click chờ được kiểm trong fixture headless giới hạn; các command khác, Gumball và một phần kiểm occlusion còn đồng bộ. Core C1–C5 chưa hoàn tất. [Bằng chứng và giới hạn](../../../../docs/validation/2026-11-14-surface-snap-source-worker.md#line-viewport-integration).

Fixture Line cũng kiểm hủy click đang chờ khi đổi Wireframe/Shaded; đây chưa phải kiểm desktop hay chuyển phần occlusion sang worker. [Bằng chứng](../../../../docs/validation/2026-11-14-surface-snap-source-worker.md#deferred-click-display-mode-guard).

Ở bước snapshot nền tảng, occlusion độc lập đã kiểm binding và không giữ Document trong fixture NativeBrep; lúc đó chưa nối vào worker sản phẩm hoặc chuyển fallback proof. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-occlusion-snapshot.md).

Line Surface Snap đã dùng snapshot occlusion trong worker khi không có trim fallback; fallback và trường hợp không admit snapshot vẫn chạy kiểm tra trên UI. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-occlusion-snapshot.md#line-worker-integration).

Sau đó, worker đã nhận thêm bản sao nguồn/trim fallback được admit; fixture trụ affine qua scheduler đạt, nhưng lỗi admission vẫn dùng đường UI và C1–C5 chưa hoàn tất. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-occlusion-snapshot.md#source-only-trim-fallback-on-the-worker).

Sau lượt tiếp theo, Line từ chối điểm khi không admit được snapshot thay vì chạy occlusion đồng bộ trên UI; kiểm thử áp lực managed và thử lại đạt. Những command khác và C1–C5 còn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-occlusion-snapshot.md#refuse-line-occlusion-when-admission-fails).

Probe hộp có lỗ sau lưu/mở tái lập đúng mesh nguồn ở hai deflection và phát hiện thay đổi tam giác/nhãn mặt; guard toàn mặt vẫn lỗi và chưa thay đường chấp nhận sản phẩm. [Bằng chứng](../../../../docs/validation/2026-11-14-brep-hole-chord.md#same-source-remesh-probe).

Worker xác minh mặt hiện có nhánh chứng thực mesh dựng lại khớp chặt; fixture nhiều lỗ qua ExtractIsocurve menu/CMD và Undo/Redo đạt. Đường xác minh đồng bộ vẫn lỗi và C1–C5 chưa hoàn tất. [Bằng chứng](../../../../docs/validation/2026-11-14-brep-hole-chord.md#captured-source-worker-exact-remesh-binding).

ExtractIsocurve với mặt B-rep gốc chọn trực tiếp nay xác minh bằng worker ở mọi kích thước mesh; fixture một lỗ và năm lỗ qua menu/CMD, stale và Undo/Redo đạt. Các route khác và phần dựng kết quả còn công việc đồng bộ. [Bằng chứng](../../../../docs/validation/2026-11-14-brep-hole-chord.md#selected-b-rep-isocurve-verification-always-uses-a-worker).

Worker xác minh mặt cũng tính miền UV nguồn cho route trên và receipt được kiểm lại trước khi dùng; dựng curve và các route khác vẫn còn công việc đồng bộ. [Bằng chứng](../../../../docs/validation/2026-11-14-brep-hole-chord.md#selected-b-rep-isocurve-bounds-on-the-face-worker).

Commit Isocurve B-rep không dùng ray đã chuyển sang lease nguồn, nhưng vẫn dựng curve đồng bộ trong sản phẩm; worker thử nghiệm không giữ Document/display và kiểm trim lỗ. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md).

Đã có job dựng curve nguồn với quota, cancel và guard context; chưa nối polling, phục hồi bộ nhớ hoặc commit của lệnh sản phẩm. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#source-only-construction-job).

Controller đã có API gọi/poll worker và commit nguyên tử được kiểm với quota và Undo/Redo; đường CMD/menu/UI vẫn chưa gọi API này và còn thiếu guard draft cùng phục hồi bất đồng bộ. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#explicit-controller-worker-and-commit).

MatrixApp hiện dựng Isocurve của B-rep OCCT trên worker cả với mặt chọn qua tia viewport; khởi tạo và tìm miền mặt bằng tia vẫn đồng bộ, NativeBrep và C1–C5 còn mở. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#viewport-ray-b-rep-construction-on-the-worker).

NativeBrep Isocurve đã chuyển phần chuyển đổi payload và dựng đường khi hoàn tất sang worker; chuyển đổi nguồn và xác định miền mặt ban đầu vẫn đồng bộ, payload opaque và C1–C5 chưa đóng. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#nativebrep-construction-on-the-worker).

Với mặt chọn qua tia viewport, MatrixApp hiện cũng xác định mặt nguồn và miền UV trên worker cho B-rep/NativeBrep; chọn witness hiển thị ban đầu và route không tia vẫn còn phần đồng bộ. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#viewport-ray-command-start-on-a-worker).

MatrixApp đã đưa cả xác định miền Isocurve cho B-rep/NativeBrep một mặt chọn theo root sang worker; nhánh mặc định headless, ownership C1 và các gate C2–C5 chưa đóng. [Bằng chứng](../../../../docs/validation/2026-11-14-captured-isocurve-source.md#root-only-command-start-on-a-worker).

Representation remesh đã chứng thực có thể tái dùng sau tịnh tiến/Undo/Redo nếu cùng allocation nguồn và proof; đây là cache còn sống, chưa phải Undo lạnh hoặc tách ownership C1. [Bằng chứng](../../../../docs/validation/2026-11-14-remesh-cache-source-rebind.md).

Snapshot hover phục vụ picking không giữ Document: nguồn curve được sao chép và tính quota, display dùng pin; receipt kiểm publication scene. Ở checkpoint này tạo snapshot còn đồng bộ; xem cập nhật worker bên dưới. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-hover-source.md).

Worker hover nay tự sao chép nguồn curve sau admission và nhả Document trước picking; scene metadata vẫn chuẩn bị trên UI và C1–C5 chưa đóng. [Bằng chứng](../../../../docs/validation/2026-11-14-detached-hover-source.md#source-copy-preparation-on-the-worker).

Candidate B-rep của Ring Builder đã tách reservation nguồn/topology và display; khi chỉ còn nguồn có thể nhả quota display. Document/Undo vẫn giữ display nên C1 chưa đóng. [Bằng chứng](../../../../docs/validation/2026-11-14-builder-split-candidate-owners.md).

Candidate Ring Builder nay tách ba reservation nguồn, topology chung và display; source-only hoặc display-only nhả quota không liên quan, kể cả metadata owner. Document/Undo vẫn chưa tách. [Bằng chứng](../../../../docs/validation/2026-11-14-builder-split-candidate-owners.md#independent-shared-topology-owner).

