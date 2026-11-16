# B0: ranh giới owner lịch sử Sphere (thiết kế có điều kiện)

Đây là thiết kế tối thiểu để đánh giá chi phí, **không phải implementation hoặc bằng chứng eviction**. Chỉ thực hiện B1–B3 nếu gate B0 của plan chính cho phép.

`UndoEntry` hiện giữ `before`/`after` là `Arc<Document>` đầy đủ. `TransactionCoordinator::restored` đối chiếu participant bằng equality của document hiện hành với snapshot mong đợi, sau đó dựng World candidate. `undo_with`/`redo_with` gọi callback dựng scene trước khi đổi world, epoch và stack. Vì `Brep` hiện bắt buộc có display, không thể chỉ bỏ mesh trong `Arc<Document>` rồi dùng lại equality hoặc `restored` như cũ.

Phương án tối thiểu cần đo: representation riêng cho **một participant lịch sử** có hai nhánh `Full` và `ColdSphere`. Nhánh lạnh phải giữ toàn bộ metadata document/đối tượng, source/topology và proof đã chứng thực, cùng mọi object không đủ điều kiện; không giữ một `Document` ẩn còn chứa display cần thu hồi. Có thể biểu diễn participant bằng khung document và record cho các Sphere đã tách, nhưng sao chép khung/record phải được admission và tính chi phí ròng; không giả định chi phí bằng không. Shared references qua nhiều entry cần được chuyển cùng hoặc tiếp tục tính display là còn sống. Nếu chưa bảo toàn metadata, proof hoặc owner thì giữ nhánh `Full`.

Các caller bắt buộc nếu gate đạt:

1. `transaction.rs`: `UndoEntry` before/after, `restored`, `undo_with`/`redo_with`, participant matching và publication/stack; `undo_memory.rs`: inventory, cắt lịch sử, chuyển/bỏ evidence và hạch toán allocation dùng chung.
2. `session.rs`: `Controller::undo`/`redo`, `prepare_history_scene`, các đường mutation/New/Open/close trong khi restore pending. World và scene cũ còn hiệu lực đến khi candidate đầy đủ được validate và publication nguyên tử.
3. `session/topology_remesh.rs` và kernel topology owner: tái dùng worker, source lease, proof, receipt và admission để dựng display/scene của participant đích; không dùng scene-only remesh làm document đã phục hồi.
4. Save v4 chỉ chụp document hiện hành đầy đủ; không serialize record lạnh vào v4. Chỉ đối chiếu participant sau khi bảo đảm source, topology, thuộc tính và mọi thay đổi tài liệu phù hợp; một hash/UUID riêng lẻ không thay thế proof hoặc kiểm tra binding.

Các số đo hiện tại và giới hạn nằm trong [báo cáo B0](../../../../docs/validation/2026-11-10-sphere-history-b0.md). Chi phí record, số owner đủ điều kiện, managed transient peak, proof theo workload và hiệu quả thực chưa được đo; không suy B1 đã được phê duyệt từ tài liệu này.
