# Đối chiếu mã nguồn FreeCAD và Blender cho plan B OM9

Ngày: 2026-11-10. Phạm vi: đọc mã nguồn, không build/chạy benchmark FreeCAD hoặc Blender; không triển khai B. Kết luận chỉ áp dụng các đường dưới đây, không chứng nhận toàn ứng dụng.

## Nguồn cố định và cách kiểm

- FreeCAD `64c74e404e0778db9c11103b332695351ea39b6c`.
- Blender `10c9b10aae4e0d16071cc0775804cdc228aceaac`.
- SHA lấy bằng `git ls-remote ... refs/heads/main`; tải 5 file mỗi dự án từ raw GitHub theo SHA, không dùng kết quả tìm kiếm làm bằng chứng duy nhất. API GitHub bị rate limit nên dùng git để lấy SHA.
- Bản tải nằm trong build gitignored; các permalink bên dưới là nguồn tái lập, báo cáo này là nơi lưu kết luận lâu dài. Không đưa mã bên thứ ba vào source sản phẩm.
- OM9 đối chiếu: `crates/om9-core/src/brep.rs` (Brep bắt buộc giữ display), `transaction.rs` (`UndoEntry` giữ before/after document, `restored` kiểm participant/validate trước callback), cùng phạm vi B0–B3 hiện hành.

## 1. FreeCAD: thuộc tính hình học, transaction và lớp hiển thị

### Ghi và khôi phục Undo

`TransactionObject::setProperty` (Transactions.cpp:421–431) dùng ID thuộc tính và chỉ `Copy()` giá trị cũ khi chưa có bản ghi. Restore áp dụng `Paste()` (406). Đây là lịch sử thay đổi thuộc tính, không phải bắt buộc snapshot toàn scene sau mỗi lệnh. Transaction cũng xử lý vòng đời thêm/xóa object; không được suy rằng mọi transaction chỉ gồm giá trị thuộc tính.

`PropertyPartShape::Copy` (PropertyTopoShape.cpp:217–230) gán `_Shape` sang property mới và giữ `_Ver`, không gọi `makeElementCopy`; comment nói deep copy bị bỏ vì tốn bộ nhớ. `Paste` dùng `setValue` (233–239). Điều này xác nhận đường property tránh yêu cầu deep copy hình học ở đây; chưa đo toàn bộ ownership native hoặc RAM mỗi Undo. [Transaction][fc-trans], [Shape property][fc-prop].

### Mesh để hiển thị

`ViewProviderPartExt::setupCoinGeometry` nhận shape cùng độ lệch tuyến tính/góc, gọi `BRepTools::Clean(shape, Standard_True)` rồi `BRepMesh_IncrementalMesh` (1051–1110), chuyển triangulation thành dữ liệu Coin. Placement được xử lý riêng (1112–1115). `updateVisual` bỏ dựng lại khi shape vẫn `IsPartner` và không `VisualTouched` (1492–1506).

Đây là bằng chứng hình học và dữ liệu vẽ có vai trò khác nhau. Không phải bằng chứng bộ thu hồi mesh lịch sử theo áp lực RAM. Hàm Clean trên native shape cũng không giải phóng các buffer CPU/GPU riêng mà ứng dụng đã sao chép. Không sao chép cách sửa cache native này vào worker OM9 nếu chưa chứng minh shape không bị reader khác dùng chung. [View provider][fc-view].

### Lưu file và cache không cần lưu

Đường SaveDocFile ASCII mặc định (`DirectAccess=true`, PropertyTopoShape.cpp:731–756) gọi `TopoShape::exportBrep(stream)`. Hàm này dùng `BRepTools_ShapeSet(Standard_False)` (TopoShape.cpp:947–961), không lưu triangulation. Nhánh qua file tạm cũng đặt false. Nhánh BinaryBrep có đường khác; không khẳng định mọi định dạng/lựa chọn FreeCAD đều bỏ mesh.

`PropertyShapeCache::Copy()` trả cache mới trống, `Paste()` xóa cache, `Save`/`Restore` không ghi dữ liệu (1069–1085). Đây là một loại cache cụ thể, không đồng nghĩa mọi mesh viewport của FreeCAD đều bị loại khỏi mọi Undo. [Shape property][fc-prop], [TopoShape][fc-shape].

### Những phần không nên dùng làm mẫu cho bảo đảm OM9

- `Transaction::getMemSize()` trả 0 (110–113). Đường commit đọc được cắt theo `UndoMaxStackSize` (Document.cpp:619–623); tên API `setUndoLimit` không chứng minh accounting unique native allocations. Không kết luận toàn FreeCAD không có kiểm soát RAM, nhưng không lấy đường này thay coordinator OM9.
- Restore property bắt exception, ghi lỗi rồi tiếp tục vòng lặp (Transactions.cpp:405–416). Đường này không cung cấp mẫu prepare/validate/publish với rollback toàn bộ như yêu cầu OM9.
- `updateVisual` bắt lỗi dựng representation; không phải giao dịch world + history + scene nguyên tử khi remesh thất bại.

[Transaction][fc-trans], [Document][fc-doc], [View provider][fc-view].

## 2. Blender: nhiều cơ chế Undo, không phải một cache chung

### Global Undo: chia sẻ phần dữ liệu không đổi

`BLO_memfile_chunk_add` so kích thước và `memcmp`; phần giống dùng lại buffer, phần khác mới cấp phát (undofile.cc:135–170). `BLO_memfile_merge` chuyển quyền sở hữu chunk còn dùng trước khi xóa bước cũ (63–91). `MemFileSharedStorage` thả user và chỉ xóa khi hết owner (55–60).

Bài học cho OM9 là chia sẻ dữ liệu bất biến, kiểm owner khi trim và không tính mỗi reference như một bản sao. OM9 đã có Arc/source lease; không cần viết lại thành memfile/chunk-diff để làm B. So byte ở đây cũng không thay chứng thực topology native. [Memfile][bl-mem].

### Edit-mode mesh Undo: representation lịch sử khác representation đang dùng

`editmesh_undo.cc` bật array-store và background task. `um_arraystore_compact` đưa các mảng vào store dựa trên bước tham chiếu (533 trở đi), có chia sẻ/khử trùng lặp; không nên mô tả đơn giản là nén ZIP. Task dùng ưu tiên thấp (999–1008).

`undomesh_to_editmesh` chờ task xong, expand record, dựng BMesh, tính lại loop triangles và normals, rồi bỏ phần expand tạm (1028–1086). Đây là tiền lệ gần nhất với “lịch sử gọn → phục hồi dữ liệu làm việc → dựng dữ liệu dẫn xuất”. Nhưng mesh tác giả của Blender chính là nguồn; không được xóa các mảng đó như mesh viewport BRep của OM9.

Hàm này giải phóng edit mesh cũ ở 1056 trước khi dựng BMesh mới. Vì thế không thể sao chép thứ tự của nó và tuyên bố OM9 vẫn giữ world cũ khi allocation/native thất bại. OM9 phải staging riêng trước publication. [Edit mesh Undo][bl-edit].

### Cache runtime và giới hạn lịch sử

`mesh_blend_write` bỏ cache legacy và đặt runtime pointer null trước khi ghi; `mesh_blend_read_data` tạo runtime mới (mesh.cc:338–484). `BKE_mesh_runtime_clear_cache` bỏ evaluated mesh/batch cache; `clear_geometry` invalidates normals, triangulation, BVH… Comment tại 319 nói dữ liệu cache chia sẻ chỉ được giải phóng nếu chỉ còn một user. Không phải mọi runtime đều bắt buộc tính lại trong mọi dạng Undo. [Mesh persistence][bl-mesh], [Mesh runtime][bl-runtime].

`BKE_undosys_stack_limit_steps_and_memory` cộng data_size từng bước rồi cắt lịch sử, giữ tối thiểu hai bước (undo_system.cc:427–480). Đây không phải admission kiểm peak old/new/pending toàn tiến trình như OM9. [Undo limits][bl-limits].

## 3. Áp dụng vào OM9: tái dùng gì, không mở rộng gì

| Bằng chứng | Quyết định cho B lịch sử |
|---|---|
| FreeCAD nguồn shape khác representation vẽ | Giữ source/topology/proof là dữ liệu phục hồi; display dẫn xuất. Không chuyển OM9 sang App/Gui/Coin của FreeCAD. |
| Blender có record Undo gọn khác mesh hiện hành | Đề xuất representation lịch sử có hai trạng thái: snapshot đầy đủ hoặc record có thể dựng lại. Current Document/Brep vẫn đầy đủ display. Đây là thiết kế đề xuất, chưa có implementation. |
| Blender chuyển owner trước khi xóa bước cũ | Tái dùng Arc/lease/deferred release của OM9; kiểm trim, bỏ Redo, close document và external readers. Không tạo kho chunk-diff mới. |
| Blender expand rồi tính lại dữ liệu dẫn xuất | Tái dùng worker remesh/receipt OM9, nhưng chuẩn bị cả candidate document + scene trước khi đổi world/stack. |
| FreeCAD bỏ triangulation trong một đường save | Có ích cho v5 sau này; không kéo codec migration vào B lịch sử. Save v4 hiện tại giữ nguyên và phải kiểm lại sau restore lạnh. |

Đề xuất thực hiện tối thiểu khi B được chọn và B0 đạt:

1. Một loại record lịch sử Sphere chứa source/topology/proof, stamp, profile dựng lại và metadata entity/document cần restore; không chỉ lưu bán kính hay seed để chạy lại lệnh tạo Sphere. Giữ ID cũ và tất cả participant/attributes.
2. Chuyển representation trong chính stack hiện có, không tạo Undo stack thứ hai. Dedup record qua các entry; nếu snapshot đầy đủ còn giữ cùng display thì báo chưa thu hồi. Record và bản cũ cùng tồn tại khi chuyển phải được admission.
3. Trước eviction, xác minh proof còn dùng không cần display cũ. Thiếu proof hoặc dữ liệu/mode không hỗ trợ giữ snapshot đầy đủ; history hỗn hợp phải hoạt động.
4. Undo/Redo: capture → admission → worker dựng candidate → kiểm source/topology/receipt/participant → validate cả document/scene → publication nguyên tử. Khi pending giữ world cũ. Cancel/stale không công bố; không trả reservation trước khi native kết thúc.
5. Chỉ report reclaimed bytes khi allocation thật hết owner. Nối pressure reclamation hiện có, không quét mỗi frame. Đóng đợt bằng đúng workload/gate B0–B3 và Save v4 hiện hành.

Điểm khó còn nguyên: `UndoEntry` đang giữ `Arc<Document>` và participant equality phụ thuộc Brep/display. Cold record phải giữ semantics tài liệu và chứng thực remesh; bỏ display khỏi một object phụ không giải phóng snapshot trong lịch sử. Không được giải quyết bằng cách bỏ qua equality hoặc chỉ so UUID/hash.

## 4. Khả năng dùng lại mã và phạm vi kết luận

- Các file FreeCAD đã đọc ghi SPDX LGPL-2.1-or-later; file Blender EditMesh Undo ghi GPL-2.0-or-later. Đợt này chỉ lưu phân tích và permalink, không port/copy code. Nếu port thật, giữ notices và kiểm điều kiện của đúng file trước khi tích hợp.
- Hướng phù hợp Rust-first: học representation/ownership/restore sequence, viết logic trong Rust và dùng adapter OCCT hiện có. Không thêm runtime FreeCAD hoặc Blender.
- Không có bằng chứng ở các đường đã đọc cho một giải pháp có sẵn đáp ứng đồng thời history-only eviction, native topology proof, managed admission và atomic low-memory restore đúng hợp đồng OM9.
- Đã đủ hai hệ thống để ra quyết định thiết kế; không mở khảo sát dự án thứ ba chỉ để kéo dài nghiên cứu. OCCT là engine được tái dùng, không phải một hệ thống Undo thay thế.
- Giữ thứ tự A trước, B0 đo rồi quyết định. Các ngưỡng 32 MiB/20%/1 giây trong plan là mục tiêu OM9 đã đặt, không rút ra từ FreeCAD/Blender và chưa được đo đạt.
- Không thay đổi plan triển khai hay nghiệp vụ trong lượt nghiên cứu này. Bước tiếp theo khi đến B: B0 đo ownership/chi phí record; nếu đạt, dựng một fixture cold-history Sphere xuyên suốt thay vì làm cache/proof phụ.

[fc-trans]: https://github.com/FreeCAD/FreeCAD/blob/64c74e404e0778db9c11103b332695351ea39b6c/src/App/Transactions.cpp#L405
[fc-prop]: https://github.com/FreeCAD/FreeCAD/blob/64c74e404e0778db9c11103b332695351ea39b6c/src/Mod/Part/App/PropertyTopoShape.cpp#L217
[fc-view]: https://github.com/FreeCAD/FreeCAD/blob/64c74e404e0778db9c11103b332695351ea39b6c/src/Mod/Part/Gui/ViewProviderExt.cpp#L1051
[fc-shape]: https://github.com/FreeCAD/FreeCAD/blob/64c74e404e0778db9c11103b332695351ea39b6c/src/Mod/Part/App/TopoShape.cpp#L947
[fc-doc]: https://github.com/FreeCAD/FreeCAD/blob/64c74e404e0778db9c11103b332695351ea39b6c/src/App/Document.cpp#L595
[bl-mem]: https://github.com/blender/blender/blob/10c9b10aae4e0d16071cc0775804cdc228aceaac/source/blender/blenloader/intern/undofile.cc#L63
[bl-edit]: https://github.com/blender/blender/blob/10c9b10aae4e0d16071cc0775804cdc228aceaac/source/blender/editors/mesh/editmesh_undo.cc#L1028
[bl-mesh]: https://github.com/blender/blender/blob/10c9b10aae4e0d16071cc0775804cdc228aceaac/source/blender/blenkernel/intern/mesh.cc#L338
[bl-runtime]: https://github.com/blender/blender/blob/10c9b10aae4e0d16071cc0775804cdc228aceaac/source/blender/blenkernel/intern/mesh_runtime.cc#L308
[bl-limits]: https://github.com/blender/blender/blob/10c9b10aae4e0d16071cc0775804cdc228aceaac/source/blender/blenkernel/intern/undo_system.cc#L427
