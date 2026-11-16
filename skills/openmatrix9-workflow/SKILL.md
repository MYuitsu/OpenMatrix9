---
name: openmatrix9-workflow
description: Use when starting or resuming OpenMatrix9 work, checking implementation progress, or choosing which Matrix9 reference documents to read next.
---

# OpenMatrix9 routing

Đọc [đích milestone và giao diện Matrix9](references/milestones-target.md) trước lựa chọn kiến trúc, spec hoặc runtime. Nguồn nền tảng `OM9_Milestones/M01 - CAD Shell, Viewport`; QymCAD chỉ cho code mẫu/routine, giao diện giữ Matrix9 như mô tả hiện tại.

## Ưu tiên ngôn ngữ

Rust là ưu tiên số 1 cho code OpenMatrix9 mới và chuyển code hiện hữu. Trước khi chọn ngôn ngữ hoặc thiết kế FFI/worker, đọc [Rust trước, native bridge tối thiểu](references/rust-first.md). Safe Rust sở hữu logic và dữ liệu độc lập; C++ dùng cho API native bắt buộc, Python cho test/tool khi cần, không bootstrap FreeCAD trong target. Không coi native FFI là memory-safe hoặc cập nhật skill là đã hoàn tất migration.

## Tối ưu hàm tạo hoặc dùng mesh

Khi triển khai/sửa lệnh CAD, áp dụng [hình học nguồn và viewport trong cùng bàn giao](references/source-and-viewport-contract.md), theo yêu cầu người dùng 2026-11-13. Phải xử lý và kiểm chứng cả hai phía chịu ảnh hưởng cùng liên kết chọn nguồn; không báo hoàn tất chỉ từ kernel hoặc hình hiển thị. Tái dùng phần đã đúng và ghi rõ nếu một phía không cần sửa.

Khi tối ưu hàm tạo mesh hoặc đường dùng mesh, truy vết nguồn và nơi dùng thực tế: viewport/preview, picking/snap, phép tính hình học, Undo và xuất. Kiểm tra có cần tách mesh hiển thị dẫn xuất khỏi hình học gốc và mesh tính toán/xuất theo dung sai riêng hay không; quyết định theo code và bằng chứng, không mặc định viết lại hoặc tạo hai pipeline khi không cần.

Với B-rep/NURBS, giữ nguồn và topology/ID làm căn cứ cho phép tính chính xác; mesh viewport không được âm thầm quyết định hình học hoặc dung sai xuất. Với mesh do người dùng tạo hay nhập, các tam giác có thể chính là nguồn gốc: giữ chúng và không tự suy ra mặt cong hay ép chuyển B-rep. Khi picking/snap B-rep dùng mesh để tìm mặt ứng viên, kiểm tra khả năng hiệu chỉnh về nguồn, binding và trạng thái camera/scene; không nhận kết quả cũ hoặc không xác minh là chính xác.

Trong phạm vi thay đổi, kiểm thử hai cấu hình hiển thị với cùng nguồn và cấu hình xuất, đối chiếu hình học, picking/snap và kết quả xuất khi liên quan; phân biệt sai số được lấy mẫu với bảo đảm toàn cục. Ghi giới hạn và phần chưa kiểm, không tuyên bố đã tách toàn hệ thống chỉ từ một fixture.

Keep the user's Rust implementation goal. Guide documents describe VB6 recovery methods; they do not change that goal.

Resolve the OpenMatrix9 checkout from the user's path or repository. This machine uses `H:/OpenMatrix9`. All document paths in [stages.json](references/stages.json) are relative to that checkout.

Khi ổ workspace hoặc ổ target thiếu chỗ, chủ động dọn build cache cũ theo [quy trình build-validation](../openmatrix9-build-validation/SKILL.md#xử-lý-ổ-đầy), rồi tiếp tục công việc; không chờ bị chặn chỉ vì chưa dọn cache.

Run through Python, using this skill's actual installed or repository path:

```powershell
python <skill-directory>/scripts/reference_index.py --project-root H:/OpenMatrix9 status
```

Then run `route <stage>`:

| Stage | Required skill |
|---|---|
| inventory | openmatrix9-workflow |
| standalone-foundation | openmatrix9-feature-port |
| ui-menu / ui-viewports | openmatrix9-ui |
| native-mapping | openmatrix9-native-mapping |
| feature-port | openmatrix9-feature-port |
| build-validation | openmatrix9-build-validation |

Read `read_now` in order; read conditional files only for the stated task. Feature lookup: `search "Picking Selection" --domain M02`, then `feature OM9-L2-01-07`. Load one spec, not the complete catalog or Ghidra export.

Reconcile `docs/openmatrix9-progress.json` with code and tests. Record completed evidence, uncertainty, next document and next action using [progress.md](references/progress.md). After implementation and checks, follow its required milestone/checklist and README synchronization before reporting completion. Plan checkboxes, source presence and catalog `implementation_status` cannot establish completion.

Source conflicts or missing manuals: consult [source-policy.md](references/source-policy.md). Continue independent authorized work; report only the evidence gap affecting the selected task.

## Launcher và cập nhật OM9 trên Windows

Quy tắc người dùng duyệt ngày 2026-10-12; áp dụng khi làm launcher/cập nhật OM9:

- Chỉ kiểm tra release khi người dùng mở OM9 qua launcher/icon, một lần cho mỗi lần mở. Không thêm lịch, service hay vòng lặp kiểm tra định kỳ khi đang dùng app. Mất mạng không chặn mở OM9.
- Có bản mới, người dùng nhấn **Update** thì tự tải gói chính thức, xác minh checksum/manifest và cài; không chuyển sang trang tải để yêu cầu tải/giải nén thủ công. Nhận cả version có timestamp và revision launcher, không hạ phiên bản.
- Nếu cần thay runtime/component đang được dùng, chờ người dùng lưu và đóng OM9 rồi mới cài, sau đó mở lại cùng profile; chỉ bảo trì plugin cũ mới cần đóng FreeCAD. Không tự đóng hoặc buộc tắt phiên đang làm. **Later** để lần mở sau; **Cancel** hủy tải/chờ và giữ bản đang cài.
- Lỗi tải/xác minh giữ nguyên bản đang cài; cài lỗi phải rollback cả runtime/component và launcher/cache trong AppData, giữ cấu hình/bản vẽ của người dùng.
- `installed` chỉ chứng minh cài xong và đã yêu cầu mở lại; cần báo cáo startup để xác nhận đúng executable/UI đã mở. Phân biệt code/gói đã kiểm chứng với ZIP release đã phát hành; launcher cũ có thể cần cài mới một lần để nhận luồng này.

## MCP với cửa sổ OM9

Quy tắc người dùng duyệt 2026-11-02: Codex kết nối `om9-live` để thao tác tài liệu của cửa sổ OM9 có cầu nối đang mở, dùng Controller và Undo của cửa sổ. Nếu không có cửa sổ có cầu nối hoặc có nhiều cửa sổ như vậy, báo lỗi thay vì chọn bừa; bản cũ không có cầu nối không được nhận diện. Tạo/mở bản vẽ tại giao diện OM9; `om9_save` dùng vòng đời lưu của cửa sổ. Không coi MCP headless `--mcp` là phiên tài liệu đang mở. Trên máy này biểu tượng **OpenMatrix9 MCP** và cấu hình Codex hiện dùng bản debug; kiểm tra lại sau khi đổi bản dựng. Phạm vi kiểm thử và giới hạn tại `docs/validation/2026-11-02-mcp-live-window.md`.

## Cập nhật nghiệp vụ sau khi code

Trong mục tiêu Solid hiện tại, người dùng đã yêu cầu ngày 2026-10-28 tự động cập nhật skill sau khi triển khai và kiểm thử, không hỏi từng quy tắc. Chỉ ghi phạm vi đã kiểm thử, nêu rõ suy luận OM9 và bằng chứng gốc còn thiếu; đồng bộ bản nguồn và bản cài đặt. Chỉ áp dụng chỉ dẫn này cho mục tiêu Solid hiện tại.

Sau khi hoàn tất code và kiểm tra, nếu có đổi nghiệp vụ, hỏi người dùng có cập nhật nghiệp vụ đó vào skill không. Chỉ ghi quy tắc mới khi có đồng ý; nếu người dùng đã yêu cầu cập nhật skill cho chính thay đổi này thì không hỏi lại. Áp dụng [quy tắc xác nhận cập nhật nghiệp vụ](references/business-rule-updates.md) để phân biệt nghiệp vụ mới với sửa kỹ thuật, chuẩn bị đề xuất cụ thể và đồng bộ skill sau khi được đồng ý.
