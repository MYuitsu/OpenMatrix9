# Kế hoạch hoàn thiện menu SNAPS

Kế hoạch bắt đầu ngày 2026-11-17 cho panel **SNAPS** của OM9 standalone trên Windows.

**Đã chốt SNAPS trên Windows ngày 2026-11-21 theo yêu cầu người dùng.** P00–P06 đã triển khai. P07 ghi kết quả kiểm thử hồi quy, phép đo Matrix9 và 4/4 bài M02-UI bằng Computer Use. Các ca phối hợp Grid Snap + Ortho và Planar + Project đã kiểm trên OM9. Phép đối chiếu trực tiếp hai tổ hợp trên Matrix9 và kiểm thử macOS thuộc phạm vi riêng. [Biên bản chốt P07](../validation/2026-11-17-snaps-p07.md#windows-snaps-plan-closeout)

OM9 đã có engine Osnap, marker/tooltip, one-shot, SmartTrack cơ bản, Ortho/Planar/Project và nhiều sửa lỗi Surface Snap gần đây. Phần thiếu đáng ưu tiên là O-Snap tổng và mặc định End/Mid, Grid Snap cùng bốn bước, Between, luồng chọn Surface/Polysurface và các constraint đúng theo Matrix. Không cần viết lại engine hoặc dựng một picking engine thứ hai.

## Nguồn và milestone

- Owner UI: **M02 `OM9-L2-03-04`**, năm L3 `001`–`005`. Owner tính snap: **M02 `OM9-L2-02-03`**, bảy L3 `001`–`007`. Constraint dùng chung thuộc `OM9-L3-02-02-006`, cross-view thuộc `02-02-007`.
- Foundation: M01 L1; `OM9-L2-03-01` quản lý viewport/CPlane/grid, `01-03` quản lý document/source identity, `01-02` quản lý command/transaction/Undo. Panel chỉ gọi các owner này.
- Tài liệu đúng cho panel là **Matrix 8 Book 1, PDF 67–72 (trang in 57–62), chương Using the Snaps Menu**; SmartTrack có thêm Book 1 PDF 40 và Rhino 5 PDF 48–49. Hai L2 Snaps hiện đều dẫn Book 1. Tìm trong tên các extract Book 2 đã chuẩn hóa chỉ thấy Retopology Snap Options PDF 90, thuộc T-Splines, không phải panel này. Chưa xác định được section Book 2 riêng cho menu SNAPS; không gán nội dung Book 1 thành Book 2.
- Registry cục bộ có đường dẫn PDF Book 1/Book 2 trong archive cũ nhưng đường dẫn đó hiện không tồn tại. Audit dùng các extract milestone đang có, không tuyên bố đã đọc lại PDF gốc hoặc loại trừ mọi chỗ nhắc tới snap trong Book 2.
- Native: `frmMenuSnaps.frm` và các definition Ghidra đúng địa chỉ đã đối chiếu. Managed `SnapsCommand.RunCommand`, token `0x06000F47`, RVA `0x00026B7E`, chỉ chứng minh `gvSnaps` mở cửa sổ; không chứa thuật toán Rhino snap.
- Receipt: [2026-11-17-snaps-menu-audit.json](../validation/2026-11-17-snaps-menu-audit.json). Mapping chi tiết, hash private input giữ tại `docs/analysis/matrix90/2026-11-17-snaps-menu.md` và file sources JSON cục bộ. Không sao chép manual, bitmap hoặc decompiled source vào public assets.

## Cách dùng theo manual và decompile

Đây là hành vi tham chiếu để triển khai; trạng thái OM9 thực tế nằm trong bảng gap bên dưới.

1. Bật công tắc **O-Snaps tổng** ở bên phải hàng trên, rồi chọn loại snap. Mặc định End và Mid bật. Khi công tắc tổng tắt, giữ lựa chọn từng loại nhưng không áp dụng chúng; màu dim/bright phải thể hiện được hai trạng thái này.
2. Chạy lệnh cần nhập điểm, đưa chuột gần vị trí phù hợp; chờ cue End/Mid/Center… rồi click để nhận điểm. Các nút snap điều chỉnh cách nhập điểm trước hoặc trong lệnh, không tự tạo hình học.
3. Hàng trên, trái sang phải: **Perpendicular, Tangent, Intersection, Point, Near, Quad, Center, Mid, End**, rồi O-Snaps tổng. End/Mid không chỉ dành cho line; manual còn mô tả curve seam/vertices và surface edges. Quad dùng cực trị theo CPlane, có cả ellipse/arc.
4. **On Surface / On Polysurface**: dùng trong lệnh, chọn surface/polysurface làm đối tượng ràng buộc điểm. Native phân biệt chuột trái `_OnSurface`/`_OnPolysurface` và chuột phải `_PersistentOnSurface`/`_PersistentOnPolysurface`. Cần giữ rõ mục tiêu đã chọn và vòng đời one-shot/persistent.
5. **Between**: trong lúc lệnh chờ điểm, chọn Between rồi lấy hai điểm phụ; trả trung điểm cho đúng một điểm của lệnh ngoài. Muốn dùng lần sau phải gọi lại. Hai điểm phụ không được trở thành hai điểm hình học của lệnh ngoài.
6. **Ortho**: manual mô tả bước 45°, F8 bật/tắt và Shift đảo tạm thời. **Planar** giữ mặt phẳng song song qua điểm đã đặt, kể cả bật sau điểm thứ hai. **Project** chiếu điểm snap về CPlane; xem viewport khác để kiểm cao độ.
7. Chọn **0.1 / 0.25 / 0.5 / 1.0 mm**, bật công tắc **Grid Snap** bên phải hàng dưới để khóa vị trí theo bước đó; tắt thì trả lại nhập tự do. Đây là snap step, khác spacing của lưới hiển thị.
8. Manual còn mô tả **Tab Lock** giữ hướng và **Ctrl+Click Elevator** lấy cao độ theo trục vuông góc CPlane. Chúng phải hợp tác với prompt/editor/menu đang giữ bàn phím.
9. **SmartTrack** ở Info & Settings dùng chung engine, không cần thêm nút vào ảnh SNAPS. Reference lines/points tồn tại trong lệnh; Rhino guide còn mô tả snap trực tiếp smart point, vuông góc và giao tracking line với curve thật.

Manual có những câu Intersection/Tangent mang tính workflow và mô tả apparent intersection trong view. Các trang hai cột còn chứa nội dung section bên cạnh. Cần chốt chính sách giao 3D/giao chiếu và điều kiện điểm đầu ở P00; không coi giao chiếu là giao hình học 3D hoặc sao chép một câu OCR thành kết quả hình học chính xác.

## Crosswalk 21 control nhìn thấy

Ảnh có 9 loại O-Snap + 1 công tắc tổng, 6 control ở trái hàng dưới, 4 giá trị bước và 1 Grid Snap. Con số 21 là số control, không phải tỷ lệ hoàn thành.

| Control tham chiếu | OM9 hiện có | Kết luận |
|---|---|---|
| Perp / Tan / Int / Point / Near / Quad / Center / Mid / End | `SNAP_TOP`, `snap_kind_for_icon`, engine Rust | Có UI và một phần geometry; phạm vi loại đối tượng còn thiếu |
| O-Snaps tổng | Không có slot hoặc cờ tổng trong `SnapSettings` | Thiếu; all-kinds mask không thay thế master switch |
| On Surface / On Polysurface | Hai icon cùng `SnapKind::Surface`, click bật/tắt cùng bit | Có exact surface pipeline; thiếu workflow chọn target, phân biệt mode và chuột phải |
| Between | Có icon `SnapBetween`, không nằm trong nhánh enabled/toggle | Hiển thị nhưng bị vô hiệu hóa |
| Ortho / Planar / Project | Có toggle trên panel và thanh thu gọn | Có trạng thái; constraint và shortcut chưa đủ semantics |
| 0.1 / 0.25 / 0.5 / 1.0 | Không render trong `SectionId::Snaps` | Thiếu cả control và giá trị snap step |
| Grid Snap | Có icon, disabled trong panel và thanh thu gọn | Chưa có quantization dùng cho nhập điểm |

## Gap theo code đang có

Đây là phát hiện đọc source, chưa là lỗi được tái hiện runtime trong lượt audit này. Các dòng trỏ tới working tree tại thời điểm receipt; checkout còn có thay đổi khác.

| ID / ưu tiên | Bằng chứng source và phần đã có | Phần cần làm / owner |
|---|---|---|
| G01 / P1 — master và defaults | `osnap.rs:96–125` chỉ có mask, radius, one-shot; default bật toàn bộ 11 kinds. `ui.rs:997` dùng default trực tiếp. `sidebar_palettes.rs:19–37,236–244` thiếu master | Thêm trạng thái tổng độc lập mask; default UI End/Mid; giữ mask khi off/on, cue và dim state đúng. `03-04-001/003`, `02-03-005` |
| G02 / P1 — Grid Snap | `sidebar_palettes.rs:134–147,243`; `chrome.rs:526–537` disabled; không có bốn giá trị hoặc snap step ở `constrain_point` | State enable/step mm, bốn nút và quantization trong input CPlane, dùng cùng preview/click; giữ grid display độc lập. `03-04-002`, `02-02-006` |
| G03 / P1 — Between | `SnapBetween` có slot nhưng không được kích hoạt; `SnapKind` không có trạng thái lấy hai điểm | Subprompt hai điểm, midpoint, Esc hủy subprompt, lỗi không mất lệnh ngoài; một commit/Undo của lệnh ngoài. `02-03-005`, `03-04-001/003` |
| G04 / P1 — Surface/Polysurface | `sidebar_palettes.rs:361` nhập hai nút làm một; chỉ xử lý `clicked()` thành persistent bit | Giữ selected target/face/source stamp; one-shot chuột trái, persistent chuột phải; loại target phù hợp; invalidation khi edit/delete/reopen. Tái dùng exact refinement hiện có. `02-03-004/005`, `03-04-001` |
| G05 / P1 — Ortho/Planar | `session.rs:7075–7099` Ortho chỉ chọn X hoặc Y; Planar và Project cùng đặt local Z=0. `input_plane:8789` lấy plane session/active view; chưa lưu mặt phẳng qua accepted point. Không tìm thấy F8 handler | Ortho 45°/F8/Shift; Planar qua điểm đúng thời điểm bật; Project vẫn chiếu CPlane. Xác định precedence với object snap và typed point. `02-02-006/007`, `03-04-002/005` |
| G06 / P1 — override và một input path | `set_snap_override` chỉ gán option. `osnap::candidates:644` loại kind tắt trước khi `ui.rs:4939` lọc one-shot; override một kind đang off vì vậy không cung cấp candidate. SmartTrack fallback ở `ui.rs:4980` chưa xét one-shot. `viewport_command_point:4509` constrain rồi thay bằng candidate, chỉ Project chạy sau | One-shot phải có effective mask riêng, không sửa persistent mask; quy định fallback khi miss, consume sau accept hợp lệ, cancel/lifecycle; một kết quả point dùng cho marker/preview/click/built-in/add-on. `02-03-005/007`, `03-04-003/005` |
| G07 / P1 — coverage nguồn | `curve_snaps:669–772` có Line/Circle/Polycurve, nhánh `Curve::Nurbs` không sinh candidate. BRep/NativeBrep/Surface chỉ sinh `Surface`, không End/Mid/edge snaps | Thêm NURBS/arc/ellipse, seam, vertices/edges và midpoint đúng contract; Center các loại manual hỗ trợ, Quad theo CPlane. Tái dùng geometry evaluator/source edge extraction, không lấy display sample làm bằng chứng điểm chính xác. `02-03-001/002/003` |
| G08 / P1 — Near/Tan/Perp trong view bất kỳ | Near line/circle dùng `screen_ray_point:929`, giao ray với World XY hoặc fallback một đơn vị trên ray. Circle solver dùng transformed axes trực tiếp | Giải gần ray/curve phù hợp camera, frame/placement và nonuniform scale; refine trên nguồn trong dung sai. Hỗ trợ constraint Tan/Perp ngoài line/circle và báo không hỗ trợ rõ. `02-03-002/003` |
| G09 / P1 — Intersection | `osnap.rs:620–642` chỉ cặp segment Line, bỏ cùng root; circle không đưa vào segments, NURBS bỏ trống; dùng ngưỡng `1e-9 * scale` | Cặp curve/edge đủ loại, self/polycurve khi phù hợp; tolerance từ document; giao thật và apparent intersection được phân biệt theo quyết định P00. `02-03-002` |
| G10 / P2 — Tab/Elevator | Tab hiện dùng cho command suggestions và viewport menu; không tìm thấy direction-lock/elevator state trong input path. Ctrl còn dùng cho thao tác selection | Thêm khóa hướng và lấy cao độ trong point prompt; không chiếm phím editor/menu hoặc modifier selection. `03-04-005`, `02-02-006` |
| G11 / P2 — SmartTrack cơ bản | `SmartTrack::push:206` giữ hai accepted anchors; guide dài cố định ±1000; `infer:230` chỉ giao hai guide khác axis. UI push sau accept | Bổ sung smart-point/guide projection, vuông góc và giao guide/curve theo reference, clear đúng vòng đời. Cơ chế acquisition mới phải được căn cứ nguồn; chưa đủ chứng cứ để bắt buộc một dwell timer cụ thể. `02-03-006`, `03-04-004` |
| G12 / P2 — CMD/SDK | Toggle hiện gọi state trực tiếp từ panel/chrome; E/M là UI shortcut. Không tìm thấy các route tên snap trong SDK public hoặc một snap query DTO; add-on ABI đã nhận điểm host resolve | Dùng một state/input service cho panel, thanh thu gọn và CMD; giữ identifiers icon và tên gốc, ghi rõ alias mới. Retrofit query/constrain API theo SDK owner plan; host giữ pointer/subprompt/transaction. `02-03`, `03-04`, `02-02` |
| G13 / P1 — evidence chưa đủ để đóng owner | L2 ngày 2026-10-22 ghi `technical_scope_complete: true` nhưng còn Grid Snap disabled, human M02-UI pending; các receipt mới chỉ chứng minh fixture của chúng | Giữ receipt cũ, bổ sung audit hiện tại, không suy ra hoàn tất toàn menu hay macOS. Kết thúc bằng một đợt validation gộp và ghi rõ giới hạn. Cả ba owner |

## Phần tái dùng và bằng chứng lịch sử

- `crates/om9-scene/src/osnap.rs`: candidates, priority deterministic, source-aware surface candidate, marker labels, one-shot storage và SmartTrack cơ bản. `crates/om9-app/src/ui.rs`: point prompt, preview/click và exact-source refinement; `session.rs`: coordinate/CPlane và transaction.
- Surface Snap đã có sửa về sphere/NURBS nguồn, trim holes, clipping, instance/source identity, worker stale-result và display witness. [Reopened-face regression ngày 11-11](../validation/2026-11-11-surface-snap-reopened-face-regression.md) ghi 6 lib + 20 picking + 13 scene tests passed; đây là bằng chứng lịch sử được báo cáo, không chạy lại trong lượt này.
- [Parallel-view ngày 11-12](../validation/2026-11-12-parallel-view-osnap.md) ghi bốn nhóm Gumball 7, BRep 1, snap UI 25 và Rectangle 5 passed. Phạm vi là fixture trong report; không xác nhận mọi command, camera, trim hoặc desktop input.
- `osnap.rs` test hiện có line/circle/point, segment intersection, tangent/perp, priority/one-shot, basic tracking và surface. `snap_panel_modes.rs` có Project, toggle, E/M và editor focus. Chưa thấy những test này kiểm master switch, bốn grid step, Between hoặc native left/right surface subprompts.
- Tái dùng public add-on point events và native source extraction/refinement đang có. Rust sở hữu state, solver độc lập, orchestration và UI. Chỉ mở rộng adapter OCCT/openNURBS nếu một phép tính nguồn cụ thể thực sự cần API native; ghi ngoại lệ tại package tương ứng. Không đưa runtime Rhino hay code thương mại vào OM9.

## Thứ tự triển khai

**Theo yêu cầu người dùng: làm xong P00–P06 rồi mới test một đợt P07.** Mỗi package mô tả kết quả cần đạt, không kèm một batch test riêng. Không thêm harness chẩn đoán hoặc benchmark chỉ để lặp lại implementation. Bảng dưới ghi thứ tự triển khai. [Kết quả xác nhận](#kết-quả-xác-nhận)

| Package | Công việc và đầu ra review được | Phụ thuộc / gap |
|---|---|---|
| P00 — chốt input contract | Ghi crosswalk 21 control; defaults, đơn vị mm, Ortho 45°, Planar plane capture, precedence grid/object/Ortho/Project/typed point; phân biệt giao thật/giao chiếu và điều kiện Intersection/Tangent điểm đầu. Đọc thêm đúng API/manual khi nguồn hiện tại chưa quyết định được; không biến câu OCR thành policy ngầm | Bắt đầu; G13 và quyết định cho G05/G09 |
| P01 — state và panel | Master O-Snap giữ mask, End/Mid startup, Grid enable/step, bốn nút, active/dim/tooltips, expanded/collapsed parity. Một controller/service chung; tooltip rõ các mode và bước đang chọn | P00; G01/G02, phần UI G12 |
| P02 — resolve point chung | Grid quantization theo CPlane/units, Ortho/F8/Shift, Planar qua accepted point, Project; one-shot effective mask và vòng đời; đồng bộ cue/preview/click/typed route. Giữ point ownership của built-in và add-on | P01; G05/G06, integration G02 |
| P03 — inline snap modes | Between nested point prompt; OnSurface/OnPolysurface left/right, target filter/selection và stamp; persistent lifecycle, cancel/retry và unsupported feedback. Gọi exact surface pipeline sẵn có | P02; G03/G04 |
| P04 — geometry nguồn | End/Mid/Center/Quad trên các loại đủ contract; NURBS/arc/ellipse/edge source queries; Near/Tan/Perp theo camera và frame; intersections thật/chiếu theo P00. Tái dùng source geometry và native adapter đang có, budget/cancel/cache theo revision | P02; G07/G08/G09 |
| P05 — tracking và shortcut | Tab Lock, Elevator Ctrl+Click, SmartTrack smart points/lines/curve relationships; editor/menu focus, cross-view, invalidation/clear khi command kết thúc. Không thêm control ngoài vị trí Matrix đã có | P02–P04; G10/G11 |
| P06 — CMD/SDK và hoàn thiện tích hợp | Panel/CMD/shortcut gọi cùng service. Public query/constrain DTO dùng context/tolerance/source identity, không lộ UI pointer; external add-on dùng host point path. Cập nhật API inventory/example và toàn bộ trạng thái unsupported; hoàn thiện source/preview/commit parity cho command consumers | P01–P05; G12 |
| **P07 — một đợt kiểm thử cuối** | Chạy các test hiện có liên quan và bổ sung ít regression theo workflow ở mục dưới, build/audit, kiểm panel/geometry thật; ghi receipt, runtime_implementation và ledger theo kết quả thực | Chỉ sau P00–P06; G13 |

P00–P06 là bảy package triển khai, P07 là một đợt validation cuối, không phải tám đợt test. Nếu sửa lỗi ở P07 chỉ chạy lại phần chịu ảnh hưởng rồi xác nhận gate cuối; không khởi động lại chuỗi test sau từng helper. Không có phép đo openNURBS mới được chạy hoặc một số batch tương lai được suy diễn trong audit này.

## P07: kiểm gộp đủ để nghiệm thu

Tái dùng `om9-scene/tests/osnap.rs`, `om9-app/tests/snap_panel_modes.rs`, `osnap_ui.rs`, `osnap_brep.rs`, coordinate/CPlane tests và `brep_picking_precision.rs`; bổ sung case vào những nơi này khi phù hợp. Không lập suite mới cho từng nút/helper.

- **State và input**: master off/on giữ mask; End/Mid defaults; bốn grid step với CPlane dịch/xoay và units; override kind đang off, miss/retry/Esc; F8/Shift, Ortho 45°, Planar ở cao độ khác 0, Project on/off; editor giữ phím.
- **Một workflow point hoàn chỉnh**: Line/Polyline hoặc command point consumer đại diện qua menu và CMD, cộng external add-on; Between không rò điểm phụ; surface target one-shot/persistent; preview và điểm commit trùng nhau, một Undo/Redo, đổi view không lấy hover cũ. Builder controls và Gumball chỉ áp dụng snap theo contract riêng, không tự lan tất cả panel constraints sang chúng.
- **Geometry nguồn**: NURBS/ellipse/arc/edge và intersection đủ loại theo P00; rotated/nonuniform instance, tolerance; reopened trimmed BRep và native source; stale worker không nhận. Kiểm điểm trên nguồn trong dung sai, không chỉ marker hoặc ảnh đẹp.
- **Một lượt UI Windows**: so panel 21 control với ảnh, màu active/dim, tooltip, layout thu gọn/mở rộng; SmartTrack, Tab/Elevator và cả bốn viewport. Headless/offscreen không thay cho desktop mouse/GPU hoặc human M02-UI.

Build dùng target hiện có `L:/om9-target`, `-j 2`, không tạo target thứ ba hoặc thay executable đang được người dùng mở. Nếu file đang khóa, ghi blocker và dùng đúng artifact kiểm được; không tự dừng app. Chạy `tools/public_source_audit.py`; sau validation batch dùng công cụ prune theo AGENTS. Native adapter thay đổi thì thêm SDK/runtime check đúng adapter, không mở rộng thành full unrelated workspace suite. macOS chỉ được ghi đạt khi có execution riêng.

P07 ghi kết quả kiểm nguồn hình học, tính nhất quán giữa CMD, SDK, preview và điểm commit, cùng phạm vi của từng fixture. Người dùng yêu cầu chốt phần SNAPS của kế hoạch sau lượt kiểm Windows. Các cổng nghiệm thu chung của milestone có phạm vi riêng. [Biên bản chốt P07](../validation/2026-11-17-snaps-p07.md#windows-snaps-plan-closeout)

<a id="bước-kế-tiếp"></a>

## Kết quả xác nhận

| Mục | Bằng chứng | Kết quả |
|---|---|---|
| Matrix9 `Int` và Project | Người dùng đo trên Matrix9 tại tâm hình chiếu bằng `_EvaluatePt`. CMD trả World `(0,0,8)` khi Project tắt và `(0,0,0)` khi bật. Hai ảnh đều hiện cue `Int`. Cả hai tệp đối chứng ba Line cũng đã có lượt đúng pixel. | Phép đo World và mặc định bật đã xác nhận trong Matrix9. OM9 dùng cùng mặc định và đã kiểm lúc bật riêng Int. [Tọa độ Matrix9](../validation/2026-11-17-snaps-p07.md#matrix9-world-coordinates-with-project-off-and-on) · [Mặc định OM9](../validation/2026-11-17-snaps-p07.md#om9-starts-with-apparent-intersections-enabled) |
| M02-UI | Ngày 2026-11-21, Computer Use kiểm 4/4 mục trên OM9 Windows gồm panel, one-shot, SmartTrack với Tab và Project. | Đạt trong lượt Computer Use. [Kết quả desktop](../validation/2026-11-17-snaps-p07.md#four-m02-ui-snaps-checks-using-computer-use) |
| Grid Snap đứng riêng tại nửa bước âm | Với bước 0,5 mm và X thô `-2.25`, Matrix9 đo được `-2.0`. OM9 trước bản sửa lưu `-2.5`, sau bản sửa preview và lưu `-2.0`. | Đã áp dụng và kiểm lại trong viewport OM9. [Kiểm hồi quy](../validation/2026-11-17-snaps-p07.md#om9-matches-the-matrix9-standalone-grid-snap-tie) |
| Int tại các cặp cách đều sau nhập 3DM | Ở cùng một tâm chiếu, OM9 so khoảng cách từ con trỏ đến Line nguồn. Khi cách đều, giao thật được ưu tiên. Hai mẫu khác thứ tự Line đều có kết quả Matrix9 `(0,0,2)` tại đúng tâm. | Đã áp dụng cho các cặp đoạn thẳng thiếu thứ tự tạo của OM9. [Kiểm lại bằng Import](../validation/2026-11-17-snaps-p07.md#om9-int-after-repeated-import-of-the-two-rhino-5-archives) · [Kết quả Matrix9](../validation/2026-11-17-snaps-p07.md#both-ordered-3dm-samples-at-the-exact-crossing-in-matrix9) |

Ngày 2026-11-20, lượt kiểm P07 trên commit `103261a86` đạt **288 ca** sau khi cập nhật kỳ vọng thứ tự menu viewport và chạy lại nhóm đó. Phạm vi gồm hình học nguồn, viewport/CMD, SDK chạy với gói SnapDemo, add-on Matrix Polyline và lưu–mở lại BRep rồi bắt điểm trên mặt. Nhóm kiểm thử bổ sung về cắt đường đạt 33 ca. Ngày 2026-11-21, lượt chốt chạy lại 35 ca tập trung trên phần SNAPS hiện tại. [Lượt P07](../validation/2026-11-17-snaps-p07.md#p07-on-commit-103261a86) · [Lượt chốt Windows](../validation/2026-11-17-snaps-p07.md#windows-snaps-plan-closeout)

Bản Windows OM9 kiểm thử ngày 2026-11-19 xác minh lại bằng chuột thật trong viewport. Cùng một pixel ở Looking Down cho `(-8,6,8)` khi Project tắt và `(-8,6,0)` khi bật. Phím E khi CMD rảnh hiện `End ONE-SHOT`, Escape hủy ngay. Bản vẽ đã lưu còn hai Line nguồn và sẵn sàng để kiểm trực tiếp M02-UI. [Bản dựng và kết quả Windows](../validation/2026-11-17-snaps-p07.md#windows-snaps-review-build)

Đối chiếu hai cửa sổ đang mở thấy cùng bố cục 10 điều khiển hàng trên và 11 hàng dưới. Cả hai hiển thị bước `0.5` đã chọn khi Grid Snap tắt. Trên bản OM9 kiểm thử, chọn `0.25` mm và bật Grid Snap làm cùng một pixel nhận World `(4.75,3,0)`. Undo khôi phục hai Line nguồn. [Panel và điểm World](../validation/2026-11-17-snaps-p07.md#snaps-panel-comparison-and-025-mm-desktop-pick)

Computer Use đã kiểm trực tiếp màu active/dim và tooltip, vòng đời one-shot, SmartTrack với Tab và Project trên bản OM9 Windows ngày 2026-11-21. Các xác nhận của người dùng về Elevator và On Surface được ghi riêng. [Kết quả desktop](../validation/2026-11-17-snaps-p07.md#four-m02-ui-snaps-checks-using-computer-use) · [Nghiệm thu theo package](../validation/2026-11-17-snaps-p07.md#p07-current-worktree-acceptance-audit)

**Trạng thái nghiệm thu ngày 2026-11-18.** Lượt P07 gộp đạt 261 ca. Người dùng đã kiểm xong Ctrl+Click Elevator với Line bắt đầu ở Z=2 và kết thúc tại `(9,10,8)`. Bản sửa lịch sử Elevator qua kiểm thử tự động và đã build trên Windows. On Polysurface đã qua ca chuột trái, chuột phải và click ngoài Box. Người dùng đã thử On Surface chuột trái và xác nhận viền cầu mịn sau cập nhật. Lượt kiểm tự động trên Windows xác minh On Surface chuột phải và điểm cuối sau một click ngoài Sphere. Bản Windows hiện tại đã hiển thị đủ 21 tooltip, giữ trạng thái Project giữa hai layout và bắt đúng End/Mid qua bốn viewport. Còn đối chiếu Matrix9 cho giao điểm biểu kiến và các ràng buộc kết hợp, rồi xác nhận M02-UI. [Kết quả theo package](../validation/2026-11-17-snaps-p07.md#p00p07-acceptance-audit-2026-11-18) · [On Surface chuột phải](../validation/2026-11-17-snaps-p07.md#windows-right-button-on-surface-on-the-standard-build) · [Panel và bốn viewport](../validation/2026-11-17-snaps-p07.md#standard-windows-snaps-panel-and-cross-view-line-replay)

Rhino 5 gốc bật giao điểm biểu kiến và dùng đường SmartTrack xám liền. Hai scheme Matrix9 đi kèm kế thừa các giá trị này. Đã đối chiếu lại mã khởi tạo Rhino và khóa cấu hình khi kiểm mặc định `Use apparent intersections`. [Bằng chứng cấu hình gốc](../validation/2026-11-17-snaps-p07.md#matrix9-factory-apparent-intersection-setting-recheck)

Đã build bản Windows có phần lịch sử CMD mới. Ca Between xác minh tọa độ midpoint trong log trùng điểm đầu Line. Ca On Surface giữ dòng chọn mặt, click trượt và hủy ràng buộc sau khi hoàn tất Line. Ca On Polysurface giữ lỗi chọn Sphere và dòng nhận Box sau khi vẽ Line với cả hai đầu ở Z=10. [Bằng chứng lịch sử CMD](../validation/2026-11-17-snaps-p07.md#between-and-surface-targets-in-command-history)

Ca egui xác minh `F2`, icon Command History và `CommandHistory` mở cùng lịch sử CMD và giữ nội dung nhập dở trong Line. Bản thử Windows mới xác nhận lịch sử giữ `Start of line`, tọa độ đầu đã gõ và `End of line` sau khi click điểm cuối. Replay egui đã kiểm cả hai cách mở Line và hai cách đặt điểm đầu. [Bản sửa CMD và lượt Windows](../validation/2026-11-17-snaps-p07.md#line-typed-start-and-viewport-end-in-command-history)

Lượt Windows ngày 2026-11-19 xác nhận nhấn Escape một lần xóa End khi CMD rảnh và khi Line đang chờ điểm cuối, kể cả lúc OM9 chỉ nhận sự kiện nhả phím. Lệnh Line vẫn nhận click điểm cuối và lưu đúng một lượt Undo. [Trace và đọc lại hình học](../validation/2026-11-17-snaps-p07.md#escape-release-with-a-viewport-one-shot-armed)

Đã đồng bộ tài liệu Workspace với quy tắc bàn phím đã duyệt trong skill UI. Tám ca kiểm lại xác nhận `E` bật End one-shot khi viewport nhận phím, CMD nhận chữ khi đang có focus, và các prompt nhập bán kính Pipe hay tọa độ Polysurface lấy focus đúng bước. [Quy tắc và kết quả kiểm lại](../validation/2026-11-17-snaps-p07.md#workspace-skill-aligned-with-the-approved-keyboard-routing)

Ca Tab Lock trên bản OM9 kiểm thử giữ preview theo phương ngang khi con trỏ rời hàng đã khóa. Cùng pixel chốt điểm cuối cho chênh lệch World Y bằng 0 khi khóa và 5,107266 mm ở lượt đối chứng. Mỗi Line có một mục Undo. [Đối chứng Windows](../validation/2026-11-17-snaps-p07.md#windows-tab-lock-and-free-click-comparison)

Gói SnapDemo đã qua kiểm tra trên Windows khi host không có OCCT. Hai Line lệch cao độ cho giao biểu kiến ở `Z=8`, sau Project lưu đúng một Point ở `Z=0`. Surface, SmartTrack và truy vấn giao biểu kiến khi có thêm Circle trả `unsupported` và giữ nguyên bản vẽ. [Ca SDK không OCCT](../validation/2026-11-17-snaps-p07.md#loaded-sdk-snaps-on-the-windows-host-without-occt)

Ba lần click cùng vị trí trong OM9 trên Windows xác nhận Grid Snap 0,5 mm chạy trước Ortho 45° khi bật đồng thời. Điểm kết hợp là `(2.5,2.5,0)`. Thứ tự trong Matrix9 vẫn chờ đo trực tiếp. [Tọa độ World đọc từ Line](../validation/2026-11-17-snaps-p07.md#windows-grid-snap-and-ortho-precedence)

Mã Rhino 5 dùng điểm đã bắt lưới để chọn hướng Ortho. Ở ca 45° dưới đây, nó chiếu vị trí con trỏ gốc lên hướng đã chọn rồi tìm giao với vạch lưới gần nhất. [Truy vết Grid Snap và Ortho](../validation/2026-11-17-snaps-p07.md#rhino-5-native-grid-snap-and-ortho-composition)

Ở đúng nửa bước lưới âm, chỉ bật Grid Snap với bước 0,5 mm và X thô `-2.25` cho X `-2.0` trên cả Matrix9 lẫn OM9 sau bản sửa. Replay xác nhận preview, điểm cuối Line đã lưu và Undo. [Tọa độ World trong hai ứng dụng](../validation/2026-11-17-snaps-p07.md#om9-matches-the-matrix9-standalone-grid-snap-tie)

Ở phần giao 3D, OM9 giữ được giao điểm của hai Line khi một cặp Line/NURBS chồng nhau ở cao độ khác nhưng cùng nằm dưới con trỏ. Ca replay đối chiếu hai đối tượng nguồn và tọa độ điểm đầu Line. [Bằng chứng P04/P07](../validation/2026-11-17-snaps-p07.md#isolated-intersection-beside-a-line-and-nurbs-overlap)

Decompile Matrix90 và disassembly Rhino 5 xác định thứ tự ở nhánh nhận điểm thông thường. Rhino chọn điểm `Int` trên đường nguồn trước, rồi chiếu điểm nếu bật Project. Với Planar tắt, Project dùng CPlane. Với Planar bật và Line đã có điểm đầu, Project dùng mặt phẳng song song CPlane đi qua điểm đầu. Người dùng đã duyệt áp dụng hai quy tắc kết hợp vào OM9. Bản thử Windows đã lưu đúng điểm cuối của hai ca Grid Snap + Ortho và Planar + Project. [Bằng chứng native](../validation/2026-11-17-snaps-p07.md#rhino-5-native-int-and-project-point-path) · [Kết quả OM9](../validation/2026-11-17-snaps-p07.md#approved-grid-snap-plus-ortho-and-planar-plus-project)

### Kiểm trực tiếp trên Matrix9

Ngày 2026-11-19, cửa sổ Matrix9 hiển thị `apparent-crossing.3dm` và CMD báo đã đọc thành công. Lượt tự động quan sát được viewport nhưng click CMD và Int chưa làm đổi trạng thái giao diện. Lượt khởi động ngày 2026-11-20 chưa mở được Matrix9 sau hộp thoại quyền Windows. Ba tệp mẫu dùng cho hai phép đối chiếu Int vẫn còn sẵn. [Trạng thái bản vẽ tham chiếu](../validation/2026-11-17-snaps-p07.md#matrix9-fixture-opens-in-the-native-viewport)

Ngày 2026-11-21, tệp cũ hiện thông báo RDK mới hơn trên Matrix9. Để đo, dùng các bản `-rhino5.3dm` đã ghi lại ở định dạng v5 với bảng dữ liệu RDK bị loại bỏ. Bộ đọc native xác nhận tệp hai Line giữ đúng hai cao độ và hai tệp ba Line giữ đúng thứ tự ban đầu. [Tệp hai Line](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-matrix9-fixtures/apparent-crossing-rhino5.3dm) · [Kiểm tra tệp mới](../validation/2026-11-17-snaps-p07.md#rhino-5-fixtures-without-the-rdk-user-table)

Người dùng mở thành công tệp hai Line mới và đo bằng `_EvaluatePt` tại tâm trong Top. Project tắt trả World `(0,0,8)` mm. Project bật trả World `(0,0,0)` mm. Cue `Int` hiện ở cả hai trạng thái. [Kết quả đọc từ hai ảnh CMD](../validation/2026-11-17-snaps-p07.md#matrix9-world-coordinates-with-project-off-and-on)

**Int.** Chọn Looking Down với CPlane World Top. Vẽ hai đoạn bằng `_Line`. Nhập từng đầu mút ở prompt tương ứng.

| Đoạn | Điểm đầu | Điểm cuối |
|---|---|---|
| Ngang | `-10,0,2` | `10,0,2` |
| Dọc | `0,-10,8` | `0,10,8` |
| Đường chéo, thêm sau phép đo hai đường | `-8,-8,2` | `8,8,2` |

Chỉ bật Int trong O-Snap. Dùng `_EvaluatePt` với `Label=Off` và ghi dòng tọa độ World trong lịch sử CMD (`F2`). Ghi cue và tọa độ khi tắt rồi bật giao điểm biểu kiến. Khi bật, rê sát đường ngang, sát đường dọc và tại tâm hình chiếu. Lặp lại tại cùng các vị trí với nút Project trong SNAPS bật. Ghi rõ lượt nào hiện cue Int trước khi click. Sau đó thêm đường chéo và lặp lại quanh tâm để đối chiếu giao điểm thật với giao điểm biểu kiến. [Rhino 5 EvaluatePt](https://docs.mcneel.com/rhino/5/help/en-us/commands/evaluatept.htm)

Book 1 mô tả Int ở điểm cuối khi đang vẽ. Nếu `_EvaluatePt` chưa hiện cue Int, thử lại tại prompt điểm cuối của `_Line`, với điểm đầu gõ `20,20,0`. Khi cue Int hiện, click, chọn riêng Line vừa tạo và đọc điểm cuối bằng `_What`. Undo Line đo trước mỗi lượt tiếp theo. Nếu ca giao biểu kiến vẫn thiếu cue, dùng đường chéo ở Z=2 làm đối chứng giao thật tại cùng vị trí. [Rhino 5 What](https://docs.mcneel.com/rhino/5/help/en-us/commands/what.htm)

Với hai ứng viên có cùng mức ưu tiên và khoảng cách màn hình, nhánh so độ sâu ưu tiên đoạn gần camera hơn. Trong Looking Down, ca đúng tâm dự kiến là `(0,0,8)` khi Project tắt và `(0,0,0)` khi Project bật, với Planar tắt ở cả hai lượt. [Truy vết thứ tự chọn và chiếu](../validation/2026-11-17-snaps-p07.md#rhino-5-native-int-and-project-point-path)

Ca Perspective bổ sung dùng giao điểm lệch trục nhìn để phân biệt điểm nguồn ngay cả khi bật Project. Hai điểm nguồn cùng vị trí trên màn hình nhưng có World X lần lượt bằng 9 và 6. Khi các tiêu chí trước độ sâu bằng nhau, mã dự đoán `(6,0,8)` lúc tắt Project và `(6,0,0)` lúc bật, với Planar tắt. Kết quả này dựa trên binary và tính toán fixture. [Tọa độ, camera và phép kiểm](../validation/2026-11-17-snaps-p07.md#apparent-intersection-in-perspective)

OM9 đã kiểm được phần lấy tham số nguồn Perspective bằng `Near` trên cùng hai đoạn, kể cả khi mặt phẳng cắt gần che một đầu đoạn. Công thức dùng chung cho snap và chọn cạnh. Ca này xác minh phần ánh xạ về đoạn nguồn dùng cho giao biểu kiến. [Kiểm thử OM9](../validation/2026-11-17-snaps-p07.md#source-parameters-for-projected-segments)

Bản thử OM9 có **Snap to apparent intersections** trong menu chuột phải của O-Snap và lệnh `ApparentIntersections On/Off`. Tùy chọn đang tắt lúc khởi động. Ca Perspective đã xác minh điểm World khi bật và tắt Project, cùng thao tác dịch con trỏ để đổi nguồn. Bộ xử lý nền đã qua ca cung, NURBS và đoạn thẳng. Trên Windows OM9, cùng một pixel ở Looking Down cho World `(-8,6,8)` khi Project tắt và `(-8,6,0)` khi bật. Hai cao độ này khớp kết quả người dùng đo trên mẫu hai Line ở Matrix9. [Replay cung/NURBS](../validation/2026-11-17-snaps-p07.md#apparent-intersection-on-arcs-and-rational-nurbs) · [Readback Windows](../validation/2026-11-17-snaps-p07.md#windows-apparent-int-with-project-on-and-off) · [Tọa độ Matrix9](../validation/2026-11-17-snaps-p07.md#matrix9-world-coordinates-with-project-off-and-on)

Replay Perspective bổ sung giao điểm biểu kiến trên phần NURBS còn nhìn thấy khi một trong các điểm điều khiển ở sau camera. Line nhận World `(6,0,37/3)` khi tắt Project và `(6,0,0)` khi bật. Ca scene kiểm thêm đường đảo chiều và loại giao điểm bị mặt phẳng cắt gần che. [Kết quả P04](../validation/2026-11-17-snaps-p07.md#apparent-int-on-a-partly-visible-perspective-curve)

Ca vành trụ BRep và Line ở cao độ khác nhau kiểm thêm giao biểu kiến cạnh nguồn. Cả Wireframe lẫn Shaded chọn điểm vành `(√2, √2, 10)` trong Perspective. Bật Project giữ X/Y và đưa đầu Line về Z=0. Mỗi lượt Undo khôi phục hai nguồn. [Replay cạnh BRep](../validation/2026-11-17-snaps-p07.md#apparent-intersection-on-a-cylinder-rim)

Đã sửa lỗi trim NURBS lệch knot làm Perpendicular từ chối vành trụ BRep. Wireframe và Shaded đều nhận được Tangent và Perpendicular trên vành trên trong dung sai bản vẽ, lưu đúng điểm marker và hoàn tác được bằng một Undo. Bốn ca cắt quanh cùng knot xác nhận giữ đúng knot nguồn và chiều chạy của đường NURBS không tuần hoàn khi cắt xuôi hoặc ngược. [Kiểm thử Tangent và Perpendicular](../validation/2026-11-17-snaps-p07.md#tangent-and-perpendicular-on-a-cylinder-rim), [Kiểm thử trim NURBS](../validation/2026-11-17-snaps-p07.md#exact-nurbs-knot-cuts-in-both-directions)

Sau khi mở lại `.om9`, Line chỉ cắt mép lỗ của mặt NURBS đã xoay và co giãn không đều. Int giữ ID của cả hai nguồn. Wireframe và Shaded đều lưu `(5,3,5)` khi Project tắt và `(5,3,0)` khi bật. Một Undo khôi phục hai nguồn sau mỗi lượt. [Kiểm thử Int trên mép lỗ](../validation/2026-11-17-snaps-p07.md#true-intersection-on-a-reopened-trim-hole-boundary)

Ca NURBS bậc ba tự cắt xác nhận giao thật giữa tham số `0.25` và `0.75` trên cùng một nguồn. Line dùng điểm World `(3,4,5)` khi Project tắt và `(3,4,0)` khi bật. Một lượt Undo giữ lại đúng NURBS gốc. [Replay NURBS tự cắt](../validation/2026-11-17-snaps-p07.md#true-intersection-on-one-cubic-nurbs)

Replay Looking Down xác minh giao điểm thật và giao điểm biểu kiến của các đoạn cùng thuộc một Polycurve lồng và đảo chiều. Tham số trên đường gốc lần lượt là `0.625` và `0.125`. Với Project bật, Line dùng điểm chiếu và tham số vẫn xác định điểm snap trên Polycurve. [Replay Polycurve](../validation/2026-11-17-snaps-p07.md#intersection-parameter-on-a-reversed-polycurve)

Bộ tính snap trực tiếp trong scene dùng cùng phép quy đổi tham số Polycurve. Ca thử giữ đúng tham số trên NURBS hữu tỉ bậc 1 có miền tham số bị cắt, kể cả giao điểm biểu kiến trong Perspective. [Kiểm tham số ở scene](../validation/2026-11-17-snaps-p07.md#scene-intersection-source-parameters)

SDK cho phép add-on yêu cầu `apparent_intersection` và lấy hai nguồn cùng điểm World trước Project. Ca add-on dùng hai đường trong một block, tạo Point tại `(9,0,0)` từ điểm trên cung `(9,0,2)` và hoàn tác bằng một lần Undo. [Bằng chứng SDK](../validation/2026-11-17-snaps-p07.md#apparent-intersection-through-the-sdk)

Tài liệu P06 liệt kê sáu hàm snap, đơn vị pixel logic và mm theo World, cùng cách xử lý khi host trả `pending`, `stale` hoặc `unsupported`. Đoạn mẫu trong README và plugin SnapDemo đều biên dịch được từ ZIP nguồn. [Danh sách API](../../sdk/API-INVENTORY.md) · [Kiểm tra gói SDK](../validation/2026-11-17-snaps-p07.md#sdk-snap-contract-and-source-bundle)

Ca Polyline nạp từ gói Matrix add-on xác minh Between xóa đoạn xem trước cũ và dùng đúng trung điểm của hai điểm End cho preview và điểm lưu. Polyline nhận một điểm từ Between và tạo một mục Undo khi hoàn tất. [Replay add-on](../validation/2026-11-17-snaps-p07.md#between-preview-in-the-packaged-polyline-add-on)

Trên cửa sổ OM9 Windows, icon Polyline và Between nhận điểm qua viewport. Một lần Enter với con trỏ vẫn ở Looking Down lưu đoạn `(-14,6,2)` đến `(-5,9,5)`. Undo đưa bản thử về hai đường nguồn. [Kiểm tra Windows](../validation/2026-11-17-snaps-p07.md#viewport-enter-completes-the-add-on-point-prompt)

Trong Polyline của add-on Matrix, Escape sau điểm tham chiếu thứ nhất ghi `Between cancelled` và trả về prompt điểm tiếp theo. Gọi lại Between, chọn hai End mới và nhấn Enter ở viewport lưu đúng đoạn từ `(-14,6,2)` đến `(-8,6,8)`. Một Undo khôi phục hai đường nguồn. [Hủy và gọi lại Between](../validation/2026-11-17-snaps-p07.md#windows-between-cancellation-and-retry-inside-polyline)

Polyline của add-on Matrix giữ điểm đầu gõ ở `Z=3` và cue End trên nguồn ở `Z=9`. Preview và hình học đã lưu cùng nhận điểm cuối ở `Z=0` khi chỉ bật Project, hoặc ở `Z=3` khi bật cả Planar. [Kiểm thử Planar và Project qua Polyline](../validation/2026-11-17-snaps-p07.md#planar-and-project-through-packaged-matrix-polyline)

Khi đối chiếu giao thật và giao biểu kiến cùng chỗ, tạo mới ba đoạn trong hai bản vẽ thử. Giữ Project và Planar tắt. Khi bật giao biểu kiến, mã dự đoán thứ tự ngang → dọc → chéo cho `Z=8`, còn dọc → ngang → chéo cho `Z=2`, với click đúng tâm và khoảng cách màn hình bằng nhau. Lặp lại khi tắt giao biểu kiến để kiểm đối chứng, khi đó cả hai thứ tự dự kiến cho `Z=2`. [Truy vết các cặp giao cạnh tranh](../validation/2026-11-17-snaps-p07.md#rhino-5-native-int-across-competing-curve-pairs)

OM9 đã trả đúng các tọa độ trên trong replay sau Undo/Redo và mở lại bản vẽ. Ca bổ sung xác nhận điểm giao thật xét sau sẽ thay kết quả trước nếu gần con trỏ hơn. Phép đo trực tiếp trên Matrix9 vẫn mở và có thể thực hiện trong bản vẽ trống, tạo Line bằng CMD. [Kết quả OM9](../validation/2026-11-17-snaps-p07.md#om9-creation-order-for-competing-intersections)

Replay viewport với block scale không đều và thành viên xoay 90° xác minh Int giữ hai đường dẫn thành viên tới cùng đường nguồn. Cả Wireframe và Shaded ghi điểm đầu Line `(6,2,8)` khi Project tắt và `(6,2,0)` khi Project bật. [Giao biểu kiến trong block](../validation/2026-11-17-snaps-p07.md#apparent-intersection-between-two-placements-of-one-block-member)

Trong Shaded, mesh ở `Z=5` vẫn để lộ điểm nguồn ở `Z=8`, nên Project nhận điểm ở `Z=0`. Mesh ở `Z=10` che cả hai nguồn, giữ Line chờ điểm đầu và giữ one-shot. Gói SnapDemo xác nhận cùng kết quả. [Kiểm tra che khuất trước Project](../validation/2026-11-17-snaps-p07.md#apparent-intersection-checks-shaded-visibility-before-project)

Hai tệp Rhino 5 đã sẵn sàng để mở riêng bằng Open. Bộ đọc 3DM xác nhận cả hai có cùng ba đoạn theo hai thứ tự trên. Nếu hai lượt Matrix9 cho cùng một Z khi đã bật giao biểu kiến, lặp ca bằng ba Line mới tạo trong CMD để đối chiếu thứ tự tạo trong phiên. [Tệp ngang trước](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-matrix9-fixtures/coincident-true-and-apparent-rhino5.3dm) · [Tệp dọc trước](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-matrix9-fixtures/coincident-vertical-first-rhino5.3dm) · [Đọc lại tệp](../validation/2026-11-17-snaps-p07.md#rhino-5-fixtures-without-the-rdk-user-table)

Để xác nhận ca cách đều sau Open, bật lại cả ba đường bằng `_Show`, chọn Top, bật riêng Int và **Use apparent intersections**, tắt Project, Planar và Grid Snap. Chạy `_-RunPythonScript` với `matrix9-exact-int-pick-anchor.py`. Click một lần gần giao điểm để chọn pixel, rồi giữ nguyên chuột khi script canh tâm chiếu vào pixel đó. Khi `_EvaluatePt` hiện `Label=Off` và cue Int hiện ở dấu xanh, click lần hai tại chỗ. Chỉ đối chiếu dòng World của lượt có `EXACT INT PIXEL: YES`. Lặp trên tệp còn lại với cùng cài đặt. [Script kiểm pixel](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-matrix9-fixtures/matrix9-exact-int-pick-anchor.py)

Tệp dọc-trước đã có lượt đúng pixel `(1121,344)` và tệp ngang-trước đúng pixel `(1137,338)`. Cả hai đều cho World `(0,0,2)` mm. Hai pixel thuộc hai khung nhìn và đều trùng tâm chiếu tương ứng. Khi kiểm lại, chạy `_Zoom _Extents` trong Top trước nếu đang zoom sát giữa các đoạn. [Kết quả click đúng pixel](../validation/2026-11-17-snaps-p07.md#both-ordered-3dm-samples-at-the-exact-crossing-in-matrix9)

Trước bản sửa, năm lần nhập tệp dọc-trước vào OM9 cho bốn lượt Z=8 và một lượt Z=2 tại cùng pixel khi bật giao biểu kiến và tắt Project. Quy tắc đã duyệt so khoảng cách màn hình từ con trỏ tới đường nguồn khi các điểm giao trùng vị trí chiếu. Kết quả khi click tâm là Z=2 ở cả hai tệp. Rê về đường dọc cho Z=8 và rê về đường ngang cho Z=2. [Lỗi trước bản sửa](../validation/2026-11-17-snaps-p07.md#repeatability-of-int-after-3dm-import) · [Kiểm lại hai tệp](../validation/2026-11-17-snaps-p07.md#om9-int-after-repeated-import-of-the-two-rhino-5-archives)

Truy vết mã Rhino 5 cho thấy Open thêm mỗi Line vào đầu danh sách mà Int duyệt. Dự đoán ban đầu cho lượt bật giao biểu kiến là tệp ngang trước nhận `Z=8` và tệp dọc trước nhận `Z=2`. Phép đo `_EvaluatePt` trực tiếp tại đúng pixel cho `Z=2` ở cả hai tệp khi tắt Project. OM9 áp dụng kết quả đo trực tiếp. [Đường Open và Int](../validation/2026-11-17-snaps-p07.md#rhino-5-ordinary-3dm-open-order-in-int) · [Phép đo Matrix9](../validation/2026-11-17-snaps-p07.md#both-ordered-3dm-samples-at-the-exact-crossing-in-matrix9)

Tệp `.om9` v5 đã giữ thứ tự tạo qua một lượt mở và lưu bằng bản OM9 Windows cũ. Bản mới mở lại đúng bảy đối tượng, hình học và thứ tự tạo của bản vẽ thử. [Kiểm thử qua hai phiên bản](../validation/2026-11-17-snaps-p07.md#v5-compatibility-for-apparent-int-creation-order)

**Grid Snap và Ortho.** Trong bản vẽ thử riêng, chọn Looking Down với CPlane World Top và đơn vị mm. Dùng `_Point` hai lần để tạo `3.22,1.29,0` và `7.31,3.27,9`. Tắt master O-Snap, Planar và Project. Với mỗi lần thử, bắt đầu `_Line`, gõ `0,0,0`, rồi click tại dấu phía dưới trái `(3.22,1.29,0)`. Giữ con trỏ tại cùng vị trí cho ba lần thử Grid Snap 0,5 mm riêng, Ortho 45° riêng và cả hai cùng bật. Phóng to đủ để xác định vị trí click. Chọn Line vừa vẽ, đọc tọa độ đầu cuối bằng `_What` và Undo trước lần tiếp theo. [Rhino 5 What](https://docs.mcneel.com/rhino/5/help/en-us/commands/what.htm)

| Chế độ | Tọa độ để phân biệt thứ tự, mm |
|---|---|
| Grid Snap 0,5 mm | `(3,1.5,0)` |
| Ortho 45° tại vị trí click đã định | Xấp xỉ `(3.22,0,0)` |
| Bật cả hai trong OM9 trước cập nhật | `(2.25,2.25,0)` |
| Bật cả hai sau cập nhật OM9 và theo nhánh Rhino 5 đã truy vết | `(2.5,2.5,0)` |

**Planar và Project.** Trong cùng bản vẽ thử hai Point, tắt Grid Snap và Ortho, mở master và chỉ bật Point. Bắt đầu `_Line` tại `0,0,3`, sau đó điều chỉnh Planar và Project theo lần thử, rồi click điểm phía trên phải `(7.31,3.27,9)` khi thấy cue Point. Ghi tọa độ thực tế khi cả hai tắt, chỉ Planar, chỉ Project và cả hai bật. Đối chiếu ba cao độ có thể phân biệt gồm nguồn `Z=9`, mặt phẳng ngang qua điểm đầu `Z=3` và CPlane `Z=0`. [Hình học thử và kết quả xác minh tọa độ](../validation/2026-11-17-snaps-p07.md#matrix9-combined-constraints-fixture)

### Kiểm trực tiếp trên OM9

Dùng tài liệu thử có Box 10 × 10 × 10 mm và Sphere bán kính 5 mm tại gốc tọa độ. Trong Looking Down, mặt trên của Box ở `Z=10` và CPlane ở `Z=0`. Giữ Project và Planar tắt khi kiểm các cao độ dưới đây. [Trạng thái tài liệu thử](../validation/2026-11-17-snaps-p07.md#windows-pointer-replay-for-surface-target-modes)

- **Elevator.** Bấm icon Line, giữ Ctrl và click trái vào vùng trống của Looking Down, xa các cạnh Box. Ghi xem CMD có hiện `Elevator: height` hay không. Nếu hiện, thả Ctrl, nhập `2` rồi Enter. Nhập `9,10,8` và Enter để hoàn tất Line. Điểm đầu dự kiến ở `Z=2` và điểm cuối là `(9,10,8)`.
- **On Polysurface.** Bấm icon Line, rồi bấm trái **On Polysurface** và click vào giữa mặt trên Box để chọn đối tượng. Click hai vị trí khác nhau ở bên trong hình chiếu mặt trên, cách xa các cạnh. Lặp lại với một Line mới nhưng bấm phải **On Polysurface**. Dự kiến ca bấm trái bắt điểm đầu ở `Z=10` rồi điểm cuối về `Z=0`. Ca bấm phải giữ cả hai điểm ở `Z=10`. Ghi prompt, hình học và một lượt Undo của mỗi Line.
- **On Surface.** Bấm icon Line và bấm trái **On Surface**. Click trong nửa trái của hình chiếu Sphere để chọn mặt, rồi click hai vị trí khác nhau ở phần đó, cách xa đường tròn biên. Điểm đầu dự kiến ở nửa trên mặt cầu, điểm cuối về CPlane. Lặp lại với chuột phải trên **On Surface**. Cả hai đầu của Line mới dự kiến nằm trên mặt cầu. Xem cao độ trong Through Finger hoặc Side View.

Ca BRep đã cắt đã qua luồng Save As v6 và Open của ứng dụng với worker đi kèm. Sau khi mở lại, Surface hiển thị đúng điểm trên mặt nguồn và Line nhận cùng tọa độ. Ca BRep v5 vẫn chịu giới hạn codec và được ghi riêng trong receipt. [Bằng chứng lưu, mở và bắt mặt](../validation/2026-11-17-snaps-p07.md#trimmed-brep-surface-snap-after-application-save-as-and-open)

Ca NativeBrep v7 kiểm Sphere đã xoay và scale không đều. Wireframe lẫn Shaded bắt đúng điểm trên mặt nguồn. Save As rồi Open giữ nguyên Sphere và Line vừa tạo. [Surface Snap trên tệp v7](../validation/2026-11-17-snaps-p07.md#surface-snap-on-a-native-sphere-reopened-from-v7)

<a id="các-mục-còn-chờ-xác-nhận-m02-ui"></a>

### Kết quả và cách kiểm lại M02-UI

Người dùng đã xác nhận chung các lệnh SNAPS đã thử hoạt động tốt. Các bước dưới đây lưu lại cách tái hiện từng bài. [Xác nhận của người dùng và kết quả kiểm bổ sung](../validation/2026-11-17-snaps-p07.md#user-review-and-remaining-snap-routes)

Dùng cửa sổ OM9 `snaps-between-reference.om9` với nút O-Snap dài và hai Line lệch cao độ trong Looking Down. Bản Windows build từ `103261a86` đã mở tệp này bằng hồ sơ thử riêng. Lượt kiểm chuột và bàn phím đã xác nhận nút tổng, lần bắt End sau click trượt, lịch sử F2, đường dẫn SmartTrack, khóa hướng bằng Tab và sự thay đổi cao độ của Line khi đổi Project. **Project** ở đây là nút chiếu điểm trong panel SNAPS. Elevator và On Surface đã có xác nhận của người dùng ở lượt trước. [Bản vẽ OM9](../../build/standalone-validation/ui-artifacts/2026-11-18-snaps-current-desktop/snaps-between-reference.om9) · [Kiểm trên Windows](../validation/2026-11-17-snaps-p07.md#windows-desktop-on-commit-103261a86) · [Xác nhận trước](../validation/2026-11-17-snaps-p07.md#windows-desktop-handoff-for-elevator-and-polysurface)

Lượt Computer Use ngày 2026-11-21 đạt cả bốn mục sau trên OM9 Windows. [Kết quả desktop](../validation/2026-11-17-snaps-p07.md#four-m02-ui-snaps-checks-using-computer-use)

- [x] [Panel và tooltip](#1-panel-và-tooltip)
- [x] [One-shot và CMD](#2-one-shot-và-cmd)
- [x] [SmartTrack và khóa hướng Tab](#3-smarttrack-và-khóa-hướng-tab)
- [x] [Project khi bắt giao biểu kiến](#4-project-khi-bắt-giao-biểu-kiến)

**Chuẩn bị.** Chọn cửa sổ có tên `OpenMatrix9 — snaps-between-reference.om9`. Trong **Looking Down** (viewport trên trái), tìm hai đường tạo thành dấu cộng phía trên trái của gốc lưới. Đường ngang nằm ở `Z=2`, đường dọc nằm ở `Z=8`, và tâm hình chiếu của chúng là `(-8,6)`. Nếu còn lệnh đang chạy, nhấn `Esc` đến khi CMD hiện `Command:`. Bấm icon **Line** trong nhóm **Curve** hoặc **Đường cong** khi bước thử yêu cầu mở Line. Sau mỗi Line thử đã vẽ, nhấn `Ctrl+Z` một lần và kiểm tra bản vẽ còn đúng hai đường nguồn. Các thao tác kiểm thử dùng bản vẽ đã mở sẵn và không cần lưu tệp.

#### 1. Panel và tooltip

1. Mở panel **SNAPS**. Trong hàng trên, **Int** là icon thứ ba từ trái, **Mid** và **End** là hai icon ngay trước **O-Snap** tổng ở ngoài cùng bên phải. Nút O-Snap nằm ngang, có một dấu `I` hoặc `O` ở giữa. Bật O-Snap tổng và bật End, Mid nếu chúng đang mờ.
2. Click trái **O-Snap** tổng một lần. **Đạt** khi nút chuyển tối, dấu `O` hiện và cả End lẫn Mid cùng mờ. Click lại. Nút chuyển xanh với dấu `I`, End và Mid tự sáng lại.
3. Rê chuột và dừng khoảng một giây lần lượt trên **End**, **Int** và **Project**. Project nằm ở hàng dưới, ngay sau **Planar** và ngay trước `0.1`. **Đạt** khi End hiện `End point`, Int hiện `Intersection`, và Project báo chiếu điểm snap lên mặt phẳng nhập.
4. Để **Grid Snap** (icon lưới cuối hàng dưới) ở trạng thái tắt. Click số `0.25`. **Đạt** khi `0.25` được tô là bước đang chọn nhưng icon Grid Snap vẫn mờ. Chọn lại `0.5` cho các bài sau.
5. Bật **Project**. Click thanh tiêu đề **SNAPS** có dấu `−` để thu gọn. Di chuột lên icon Project trong hàng rút gọn để nhận ra nó. **Đạt** khi Project vẫn sáng. Click tiêu đề có dấu `+` để mở lại, kiểm tra Project vẫn sáng, rồi tắt Project.

#### 2. One-shot và CMD

1. Tắt **Grid Snap**, **Ortho**, **Planar**, **Project** và **SmartTrack** nếu đang bật. Khi CMD hiện `Command:`, click một vùng trống của Looking Down để viewport nhận bàn phím. Nhấn `E` một lần. **Đạt** khi thấy chữ vàng `End ONE-SHOT` ở góc trên trái của viewport và ô CMD vẫn trống. Nhấn `Esc` một lần để xóa chỉ báo.
2. Bấm icon **Line**. CMD phải hiện prompt `Start of line` với tùy chọn `BothSides` và sẵn sàng nhập. Click một điểm trống trong Looking Down, chẳng hạn phía dưới bên phải của gốc lưới. CMD chuyển sang `End of line:`. Giữ chuột trong viewport và nhấn `E`.
3. Click một chỗ trống khác, xa hai đường nguồn và xa điểm đầu vừa đặt. **Đạt** khi Line vẫn chờ `End of line:`, chưa tạo đường mới và `End ONE-SHOT` còn hiện.
4. Rê đến đầu bên trái của đường ngang nguồn. Đợi marker **End**, rồi click. **Đạt** khi đầu cuối của Line mới nằm ở đầu đường nguồn, CMD trở về `Command:` và one-shot biến mất.
5. Nhấn `F2` để mở **Command History**. Tìm các dòng của lượt vừa chạy, gồm `Command: Line`, prompt `Start of line`, `End of line:` và thông báo không tìm thấy End khi click trượt. Nhấn `F2` để đóng lịch sử, rồi Undo Line thử một lần.

#### 3. SmartTrack và khóa hướng Tab

1. Bật **O-Snap** tổng và **End**. Để Grid Snap, Ortho, Planar và Project tắt. Bật **SmartTrack** trong **Info & Settings**, hàng thứ hai, icon thứ sáu từ trái. Tooltip có thể ghi `RhinoSmartTrackON`. Xác nhận trạng thái `SmartTrack on`.
2. Bấm **Line**. Trong Looking Down, rê tới đầu trái của đường ngang nguồn. Khi thấy marker **End**, giữ chuột yên khoảng một giây mà chưa click. **Đạt** khi hiện đường dẫn SmartTrack **xám, nét liền** đi qua đầu mút. Rê ra ngoài đầu trái theo đường dẫn, đủ xa để marker End biến mất. Rê sát đường dẫn và kiểm tra marker **SmartTrack** hiện trên đó. Nhấn `Esc` để hủy Line. Đường dẫn và marker phải biến mất. Tắt SmartTrack trước bài Tab để chỉ kiểm khóa hướng.
3. Bấm **Line** lần nữa. Click một điểm trống ở nửa dưới Looking Down làm điểm đầu. Rê sang phải theo đúng một hàng lưới, chưa click. Nhấn `Tab` khi chuột vẫn trong viewport. **Đạt** khi thanh trạng thái hiện `Direction lock on`.
4. Rê chéo lên hoặc xuống vài ô lưới. Chuột đã rời hàng ban đầu nhưng preview Line phải giữ hướng đã khóa. Click để hoàn tất và kiểm tra đường vẽ vẫn theo hướng đó. Undo một lần. Có thể lặp cùng hai vị trí chuột mà bỏ qua Tab để đối chiếu, khi đó preview và Line đi theo đường chéo.

#### 4. Project khi bắt giao biểu kiến

1. Bật O-Snap tổng. Trong chín loại snap ở hàng trên, chỉ để **Int** sáng. Tắt **Ortho**, **Planar**, **Grid Snap** và **SmartTrack**. Click phải nút **O-Snap** tổng và bật mục `Snap to apparent intersections` nếu chưa được chọn.
2. Để **Project tắt**. Bấm icon **Line**. Trong Looking Down, rê ngay vào tâm dấu cộng của hai đường nguồn. Đợi marker `Apparent Int`, rồi click để đặt **điểm đầu**.
3. Click ô CMD đang hiện `End of line:`, nhập `0,15,0` và nhấn Enter. Chọn riêng Line vừa vẽ bằng cách click đoạn kéo về phía trục xanh trong Looking Down, xa dấu cộng. Nhìn **Through Finger** (viewport dưới trái). **Đạt** khi Line đi xiên từ đầu bắt snap phía trên trục đỏ xuống điểm cuối trên trục đỏ. Điểm đầu dự kiến `(-8,6,8)` và điểm cuối `(0,15,0)` mm. Undo một lần.
4. Bật **Project**. Bấm Line và lặp lại đúng vị trí bắt giao trong Looking Down với cùng điểm cuối gõ `0,15,0`. **Đạt** khi vẫn bắt `Apparent Int`, nhưng trong Through Finger cả hai đầu nay nằm trên trục đỏ. Điểm đầu dự kiến là `(-8,6,0)` mm. Nhìn lại Looking Down, hai lần đều bắt cùng vị trí chiếu. Undo một lần và tắt Project.

Ghi **Đạt** hoặc **Lỗi** cho từng mục. Nếu có lỗi, ghi số bước, điều đã thấy và ảnh có cả viewport với vùng CMD nếu thuận tiện. Ví dụ `2. One-shot và CMD — lỗi ở bước 3, click trượt làm mất End ONE-SHOT`.
