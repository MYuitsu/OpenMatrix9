# Prompt giao GPT Luna triển khai

Copy toàn bộ phần dưới khi muốn giao thực thi. Bộ tài liệu hiện tại mới là đặc tả; việc viết bộ này không tự triển khai migration.

---

Bạn đang làm trong `H:/OpenMatrix9`. Hãy chuẩn hóa phần Command/tham số của các command khác theo chất lượng tương tác của Polyline, dựa trên bộ đặc tả:

`implementation/design/M01/command-options-polyline-standard-20261026/`

Mục tiêu là thực hiện đầy đủ scope được audit trong bộ này, không chỉ viết thêm kế hoạch, không chỉ chỉnh một toggle rồi nói mọi lệnh xong. Giữ đúng thuật toán, command IDs và nghiệp vụ riêng của từng owner. “Theo Polyline” là chung owner snapshot, prompt/giá trị, shortcut/click/typed route, input stage, validation/retry/cancel, focus và transaction contract; không copy option của Polyline vào lệnh khác.

Đọc theo thứ tự:

1. Root `AGENTS.md`, `C:/Users/nguye/.codex/RTK.md`, `docs/openmatrix9-progress.json` và các skill OM9 đúng stage.
2. `README.md`, `SPEC.vi.md` trong thư mục đặc tả.
3. `COMMAND-MATRIX.vi.md`, `source-inventory.json`.
4. `OWNER-CARDS.vi.md`, `TASKS.vi.md`, `ACCEPTANCE.csv`.
5. Current code của contract/controller/UI và đúng parser/test/milestone của từng owner trước khi sửa family đó.

Hiện trạng cần biết để không bắt đầu sai:

- Đã có `crates/om9-app/src/command_options.rs` và `session/command_options.rs`; đừng tạo interface độc lập thứ hai mà bỏ caller hiện hữu.
- 37 provider + Line + Polyline = 39 nhóm khai báo. Đây không phải tổng số lệnh. Controller có 57 field `_pending`, trong đó 19 ngoài registry; phải audit cả Top11, positional Solid, surface branches và command không dùng pending.
- Common interface hiện có `label`, `kind`, `shortcut`, activation, nhưng chưa typed current value/run identity đầy đủ; nhiều numeric/text owner còn `Draft("Name=")` bridge, UI còn button row ngoài inline prompt.
- Polyline là oracle nhưng có giới hạn đã ghi: Helpers tracking, continuously closed preview, Center arc cases và exact mixed-Polycurve 3DM re-export không tự được chứng nhận.
- Bug ảnh “Start of polyline + BothSides=No Length=” xuất hiện trên release cũ vì lấy repeat Line làm active command. Source common interface đã ưu tiên pending Polyline. Giữ regression Line→Polyline; không dùng `active_feature()` có repeat fallback để chọn option owner.
- Một test Box workflow cũ đã fail trên baseline; hai native fixture tests từng ignore. Đọc report trước khi quyết định một failure là mới; không lén skip, xóa test hay sửa coordinate semantics để xanh.

Làm lần lượt P00..P12 trong `TASKS.vi.md`, checkpoint từng gói rồi tiếp tục. Mỗi family cần stage table, owner value/availability, ba route, value/point/text entry rõ ràng, formatter/span chung, actual geometry/Undo và evidence. Chưa đủ thì ghi BRIDGE_ONLY/MIGRATING/BLOCKED rõ; không nâng PARITY_DONE từ việc nút mở được.

Các yêu cầu bắt buộc khi code:

1. Safe Rust sở hữu command/session/input/validation/model; không chuyển logic này sang Python/C++. C++ chỉ khi có native API constraint thực và được scope cho phép.
2. Một active owner và run/context hợp lệ. Idle không có active options dù repeat slot còn Line. Snapshot/ID từ run cũ không mutate run mới.
3. Giá trị derive từ owner, không parse label ngược và không giữ mutable parameter cache trong MatrixApp. UI không gán thẳng pending field/geometry.
4. Toggle đảo một lần; set choice chọn một value, chọn lại không tự đảo; Cycle chỉ khi owner định nghĩa. Approx/Deformable cùng flag, Direction U/V khác Direction vector.
5. Numeric/integer/text entry đích giống Polyline: owner stage và prompt đúng parameter, draft rỗng; không submit assignment thiếu value để thử parser. Giữ direct `Name=value`; bridge chưa chuyển không được gọi là parity done.
6. Validation finite/bounds/integer/unit theo owner, không cấm mọi số âm/0. Invalid giữ stage/draft/accepted points/current value cũ và không sửa document/Undo/Redo. Parse point không bỏ token sai bằng filter_map.
7. Một protected transcript/editor; live row pinned/unwrapped và history độc lập. Inline options có spans từ formatter cùng snapshot; không substring-find Close trong PersistentClose; không giữ đồng thời floating button row cũ.
8. Unique advertised shortcut theo stage; giữ numeric keys, IME, paste, nonempty draft, popup/other editor focus, key-repeat suppression. Shortcut và snap conflict giải theo owner hợp lệ; không nuốt text đang gõ.
9. Commit timing theo command. Radius cuối của Circle có thể commit ngay; Polyline finish bằng Enter/Close theo owner. Không ép mọi command thêm Enter thứ hai, không tạo transaction chỉ để đổi option.
10. Không tạo `standalone/`; build với `CARGO_TARGET_DIR=L:/om9-target` và `-j 2`; mọi shell command prefix `rtk`. Giữ dirty changes, không stage/commit/publish ngoài yêu cầu.
11. Native verification phải biết PID/path/SHA256/profile của đúng executable. Không đóng/bỏ bản vẽ chưa lưu của người dùng hoặc build đè binary đang chạy. Debug mới không đồng nghĩa release/user install đã cập nhật.
12. Giữ original IDs/names, unsupported visible-disabled và lý do. Nếu contract/kernel thiếu, ghi đúng blocker của capability; không fake geometry hoặc success.

Sau mỗi gói, ghi report/checkpoint có source hash, command/mode đã làm, test function/scenario/fixture counts tách riêng, runtime proof, known/new failures, phần chưa làm và exact next file/action. Compaction không phải hoàn tất; đọc checkpoint rồi tiếp tục phần còn lại. Các claim 466 tests/39 groups trong report cũ chỉ là evidence lịch sử, không là acceptance của code bạn sửa.

Nghiệm thu cuối phải reconcile mọi discovered entry point, không chỉ các provider. Chạy appropriate Rust tests, Clippy `-D warnings`, scoped fmt, public-source audit, diff check và build/runtime đúng target. Cập nhật milestone runtime_implementation, README và live ledger với scope thật; human UI gates vẫn pending nếu chưa có người ký. Chỉ sửa skill nghiệp vụ khi người dùng đã đồng ý đúng thay đổi đó.

Bắt đầu bằng P00: đọc nguồn, kiểm inventory/hash/status, viết execution coverage table và xác định checkpoint đầu tiên. Tiếp tục triển khai các gói theo dependency, không yêu cầu người dùng xác nhận lại các chỉnh sửa kỹ thuật thông thường đã nằm trong nhiệm vụ.
