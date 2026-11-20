# Kế hoạch hoàn thiện menu Tools

Ngày kiểm tra ban đầu: 2026-11-16. Cập nhật ưu tiên theo người dùng: **2026-11-17**. Target: standalone Rust OM9, Windows. Lượt cập nhật này chỉ đổi kế hoạch; các kết quả audit ngày 2026-11-16 bên dưới là mốc lịch sử, không phải trạng thái runtime mới nhất. Tiến độ thực thi được ghi tại `docs/openmatrix9-progress.json` và các bản ghi validation được dẫn từ đó.

Menu Tools có **31 mục**. `addons/tools` cung cấp 30 command; `BuilderBooleanBuilder` do `addons/setting_tools::BooleanTool` cung cấp và được gộp trong `addons/matrix`. Đã chạy **10 test Tools và 1 test Boolean, tất cả đạt**. Các test này kiểm những fixture hiện có, chưa chứng minh tương thích đầy đủ với Matrix. Đặc biệt, một test Join History đang xác nhận việc xóa đường cha, trái với manual và code Matrix đã đối chiếu.

**Ring Rail (`BuilderRingRail`) là lệnh triển khai đầu tiên; đường trực tiếp là chính, add-on gọi lõi Rust chung qua lớp kết nối mỏng.** Người dùng xác nhận ngày 2026-11-17: OM9 là phần mềm chuyên nhẫn, các công cụ cốt lõi liên quan đến nhẫn/trang sức được triển khai trực tiếp trong OM9. Tiếp tục duy trì cả hai đường nhưng tập trung hoàn thiện core và luồng trực tiếp, rồi nối add-on vào cùng nghiệp vụ. Sau Ring Rail mới tiếp tục Join History, Metal Weights và các nhóm còn lại. Không dùng số icon, số handler hay trạng thái kế hoạch để tính phần trăm hoàn thành.

## Hướng triển khai và thứ tự ưu tiên đã cập nhật

- Các công cụ cốt lõi về nhẫn/trang sức (Ring Rail, rail/profile, resize và các builder liên quan) triển khai trực tiếp trong OM9 làm đường chính. Duy trì đường add-on như lớp gọi mỏng vào lõi dùng chung; không đặt nghiệp vụ cốt lõi chỉ trong plugin. Với Ring Rail, hoàn thiện builder/producer Rust hiện có trong host/core và nối `addons/tools`; `addons/matrix` tiếp tục gộp Tools và đồng bộ manifest/help.
- Ring Rail ở Top11 và Tools là cùng một lệnh, chỉ có hai icon dẫn vào cùng owner/session và giá trị tham số. Giữ tên lệnh cũ làm entry tương thích; không có implementation Ring Rail riêng cho từng menu. VNM (Việt Nam) dùng mm cho kích thước trong, có đường kính/chu vi; bảng US phải có nguồn công bố và quy tắc làm tròn/nội suy rõ ràng. Nguồn và bằng chứng từng phần: [Ring Rail](../validation/2026-11-17-ring-rail-shared-input.md).
- Hai đường dùng chung logic hình học, đơn vị, validation và contract nghiệp vụ trong service Rust khi phù hợp; adapter add-on và trực tiếp nối vào service đó. Host giữ document, transaction, Undo/Redo, publication, scene và các service dùng chung; add-on sử dụng ABI/SDK để yêu cầu thao tác host. Mỗi phần chức năng bổ sung phải nối cả hai đường, tránh phát triển hai thuật toán khác nghĩa.
- Menu Tools, CMD `RingRail`/`gvRingRail`, builder và F6 khi có context phải có route rõ ràng theo cấu hình add-on. Khi add-on bật, kiểm đường add-on; khi tắt hoặc không cài, kiểm đường trực tiếp. Giữ quy tắc chọn route hiện có trừ thay đổi cần thiết được ghi rõ; nghiệm thu riêng cả hai đường cùng dữ liệu và oracle, không lấy kết quả một đường làm bằng chứng cho đường kia.
- Thứ tự thực thi: **T00/T01 chỉ phần Ring Rail → T04-A Ring Rail → T02 Join History → T03 Metal Weights → phần còn lại của T04 → T05 → T06 → T10**; bổ sung T00/T01 theo nhu cầu từng lệnh. T07–T09 tiếp tục theo lịch owner/AFTER. T11 chỉ bắt đầu khi đã triển khai nhiều phần liên quan thành một đợt đủ lớn hoặc phần đang làm đã kiểm tra được trên giao diện.
- T00 đủ 31 route, toàn bộ T01 và T02 không phải điều kiện phải hoàn tất trước khi bắt đầu Ring Rail. Giữ toàn bộ phạm vi 31 lệnh và các gap chưa đóng trong kế hoạch.
- Bảo toàn code và bằng chứng Join History, Metal Weights, Boolean Builder đã có. Lần thực thi tiếp theo ưu tiên Ring Rail; chỉ quay lại các phần đó sớm nếu phát hiện lỗi trực tiếp chặn Ring Rail hoặc làm hỏng trạng thái dùng chung.

## Căn cứ và phạm vi bằng chứng

- Danh sách và thứ tự: `Resources/menu/MainMenu.ini` mục Tools, `crates/om9-app/src/menu_catalog.rs::TOOLS`.
- Owner M01: `OM9-L2-01-02` command/transaction/Undo, `OM9-L2-03-07` menu, `OM9-L2-03-10` CMD; document và native geometry theo `OM9-L2-01-03`/`OM9-L2-01-05`. Đọc M01 L1 và owner trước quyết định triển khai.
- Book 2: các extract `reference/text/m8book2__...` trong milestone; số trang dưới đây là **trang PDF theo tên extract**, không phải trang in. Ví dụ Profile Placer bắt đầu PDF 135, milestone ghi trang in 361. Nhiều trang hai cột chứa cả lệnh kế bên; không tự gán mọi option của trang cho lệnh tiêu đề.
- Decompile: [ghi chú procedure và độ tin cậy](../analysis/matrix90/2026-11-16-tools-menu.md). Đã đối chiếu các procedure menu VB6/Ghidra và các command C# liên quan trực tiếp đến gap. Chưa giải mã toàn bộ thuật toán của cả 31 mục.
- Code đang chạy cần xét cả built-in và add-on. `Controller::submit` ưu tiên add-on được cài và bật; toolbar cũng ưu tiên `icon_addon`. File DLL/profile thực tế của cửa sổ người dùng chưa được kiểm trong lượt này.
- [Receipt và inventory](../validation/2026-11-16-tools-menu-audit.json) lưu commit, hash, owner, alias, nguồn code và test thực chạy. Build dùng target có sẵn `L:/om9-target`, `-j 2`.

## Kết quả đối chiếu quan trọng

1. **Join History đang sai nghĩa ở nhánh add-on.** `addons/tools/src/lib.rs:891` tạo polyline từ mẫu điểm và đưa các đường cha vào `reply.delete`. Test `smart_targets_blend` còn yêu cầu có hai đối tượng bị xóa. Book 2 PDF 149 và `gvJoinHistory.cs::RunCommand/ReplayHistory` yêu cầu giữ cha, tạo con có dependency và cập nhật con khi cha đổi. Undo transaction có sẵn không thay thế associative History.
2. **Dữ liệu add-on đang phụ thuộc scene hiển thị.** `addon_session.rs::session_selection` lấy `Selected.points` và `Selected.mesh` từ `RenderBuffers`; curve bị giới hạn 4096 điểm, mesh chịu giới hạn tổng tam giác. `Selected.exact` có cho một số curve/surface, chưa có BRep service tương ứng trong đường này. Nhiều handler Tools vẫn dùng points/mesh. Vì vậy đổi độ mịn display có thể đổi phép tính; đây là rủi ro từ code, chưa phải kết quả phép đo Light/High mới.
3. **Metal Weights có hai đường khác nhau.** Built-in `session/metal.rs` đo BRep và dùng density registry; add-on `metal_weights` tính thể tích mesh hiển thị rồi chỉ cảnh báo khi mesh hở, vẫn xuất cân nặng. Khi bật add-on, `MetalWeights` có thể ưu tiên đường mesh. Cần thống nhất theo nguồn, giữ riêng phép đo mesh người dùng nếu hỗ trợ.
4. **Có lệnh mang cùng tên nhưng làm thao tác khác manual.** Surface Pullback đang chiếu thiết kế phẳng lên mặt thay vì tạo vùng lõm với Drop Depth/Bevel/Insert/Cap; Mesh Mapper đang đổi sang mesh; Center Line đang lấy trung bình hai curve hoặc tạo dấu cộng mà không dùng mặt nguồn. Đây là gap nghiệp vụ đã xác định, không chỉ thiếu UI.
5. **Built-in không thể thay thế add-on một cách máy móc.** Profile và Profile Placer dùng chung `OM9-TOOLS-002`; Target On Curve End và Smart Target dùng chung `OM9-TOOLS-011`; Mesh Mapper và Mesh Repair cùng `OM9-TOOLS-020`; Object On Curve chưa có feature fallback. Add-on phân biệt được các command icon này khi cài/bật. Four Rail Profile ở menu text gọi `OM9-TOOLS-004`, có nhánh phụ thuộc số curve chọn. Cần kiểm riêng từng route và trạng thái bật/tắt add-on.
6. **Milestone chưa mô tả đủ interaction.** Nhiều `parameter_contract` M04 còn `DOMAIN_ONLY`, không có command options và type/default còn unresolved. Curve Transform, Center Line, Surface Inset, Mesh Mapper và Image Trace chưa có L3 riêng rõ ràng theo đúng tên trong cây đã tra; ref của chúng có thể nằm trong trang dùng chung của một L3 khác. Phải chốt owner trước khi thêm API hoặc đổi trạng thái.

## Bảng đối chiếu đủ 31 mục

Mỗi dòng dưới đây đều ở mức **MOT_PHAN**: có handler hoặc hạ tầng để tái dùng nhưng còn gap hoặc nghiệm thu chưa có. Hướng dẫn là hành vi Matrix theo nguồn, không phải cam kết app hiện đã làm đủ. `Chốt owner` là việc trong T00, không phải ID milestone mới đã được tạo.

| # | Command icon và tên | Book 2 và thao tác cần có | Owner milestone | Code hiện có và phần còn thiếu |
|---|---|---|---|---|
| 1 | `BuilderRingRail` Ring Rail | PDF 134–135: chọn region hoặc mm/inch, đường kính/chu vi; đặt rail; thay rail Yes/No/Cancel | M04 `04-02-001`; M03 `03-08-005` entry | **Triển khai đầu tiên, phối hợp add-on và trực tiếp, T04-A.** Mốc audit: add-on tạo circle, US/ISO/diameter; built-in có builder/history. Hoàn thiện region, inch, custom shape/library và replace workflow qua service chung; nghiệm thu cả hai đường, package Tools và Matrix. |
| 2 | `BuilderProfilePlacer` Profile Placer | PDF 135–139: chọn một rail bất kỳ, profile; Width/Height/Position/Tilt/Twist/Roll/XYZ; Add/Mirror/Second Rail/Edit | M04 `04-02-003`, `008`, `009`; `03-11` UI | Add-on tạo các profile tại các góc circle; built-in có RailProfile/handles/history. Cần arbitrary rail, interactive Add, editor/library, mirror và đủ orientation. Count hiện đặt 3/5 nhưng sinh 4/6 section; làm rõ bằng fixture. |
| 3 | `BuilderOutsideRingRail` Outside Ring Rail | PDF 139–140: chọn rail/profile, Top/Side/Bottom Thickness, Lock, Reset và F6 Edit | M04 `04-02-002`, `009` | Có rail theo thickness và builder circle. Thiếu Lock, custom shape, editor/handle và History tương ứng với nguồn rail. Alias `gvOutsideRingRail` chưa có trong manifest Tools. |
| 4 | `BuilderCustomRail` Custom Rail | PDF 140–141: ring rail và shape Top, Make Rail, guide width/size, Delete Curves; kết quả có hai curve chưa join | M04 `04-02-004` | Add-on tạo curve từ mẫu rail/shape, mirror tùy chọn; built-in chỉ validate rồi sao chép một rail kín. Cần đúng input, kết quả, guides, Delete Curves và ràng buộc frame. |
| 5 | `BuilderFourProfile` Four Rail Profile | PDF 141–142: bốn rail, hai điểm Set Plane, profile; Move/Rotate Y/Z/Add/Sweep 4 | M04 `04-02-005`, `008`, `009` | Add-on có mặt phẳng cắt và profile nội suy; built-in có metadata Sweep4. Thiếu nối profile library, handles, các lần Add và associative sweep theo cùng nguồn. |
| 6 | `BuilderMetalWeights` Metal Weights | PDF 142–143: chọn solid kín, Calculate; GV/User/Use Object Materials; grams/DWT | M04 `04-19-001`…`005`; M06 `04-15-010` consumer | Built-in có đo BRep/density/multi-material; add-on dùng mesh và vẫn tính khi hở. Tái dùng service đo nguồn, từ chối nguồn không hợp lệ, hoàn thiện panel/material binding; thêm alias gốc `gvMetalWeights`. |
| 7 | `BuilderResize` Ring Resizer | PDF 143–144: metal + rail gốc, primary radius, góc trái/phải, nhiều cỡ; Symmetrical/Add Rings/Save to File/STL | M04 `04-02-007` | Add-on có nhiều output mesh bằng radial offset; built-in có producer circle. Chưa giữ vùng head bằng góc, đọc rail gốc bắt buộc, asymmetry và file output; không nghiệm thu từ circle resize. |
| 8 | `BuilderSurfacePullback` Surface Pullback | PDF 144–145: surface + curve kín trên mặt, Drop Depth/Bevel/Extrude/Insert Only/Cap, Apply/Select/Undo | Ref tại M03 `02-04-009`; chốt capability Tools riêng | Handler hiện drop thiết kế phẳng lên mesh mặt. Cần tạo recess/cap bằng nguồn surface/BRep, đúng depth và trim; không đồng nhất với Curve Pullback. |
| 9 | `BuilderBermarkLibrary` Bermark library | Chưa tìm thấy mục riêng Book 2 trong refs Tools đã tra; đối chiếu nguồn Matrix9 trước chốt nội dung | M08 `05-03` là owner ứng viên | Add-on hiện là Settings Library gồm head tham số OM9. Không tự coi tương đương thư viện gốc; cần catalog/import/preview cho tài sản người dùng hoặc có quyền sử dụng. |
| 10 | `BuilderStullerFindingsLibrary` Stuller findings | PDF 145–146: duyệt category/size/quality, nhiều view preview, chọn finding để load | M08 `05-03-001`…`005`, chốt adapter catalog | Có phụ kiện tham số OM9, chưa phải browser/catalog Stuller. Làm provider/browser và import có quyền, không đóng gói bộ mẫu thương mại. |
| 11 | `gvSmartTarget` Smart Target | PDF 146: click blend point, click hướng handle; target dùng cho curve có History | M08 `04-04-001`, `005` | Có Point + Line rời. Thiếu semantic target ID, nhiều handle cùng điểm, selection filter, serialization/dependency và lifecycle. |
| 12 | `gvSmartTargetAdd` Add Smart Target | PDF 147: chọn blend point cũ, thêm hướng vào chính target | M08 `04-04-001`, `005` | Có tạo line tại midpoint curve chọn; thiếu xác minh đúng target và thêm handle vào cùng object/History. |
| 13 | `gvSmartTargetOnCrvEnd` Target at curve end | PDF 147: pick gần đầu curve, đặt target tại endpoint theo tangent nguồn | M08 `04-04-002` | Có chọn endpoint từ mẫu points; tangent là chord mẫu. Cần derivative/parameter nguồn, sub-pick đúng endpoint; fallback built-in đang dùng ID Target thường. |
| 14 | `gvSmartBlend` Smart Blend | PDF 147: pick hai handle đúng phía, blend giữa hai điểm; sửa target cập nhật curve | M08 `04-04-003`, `005` | Có NURBS blend theo midpoint và hướng line; thiếu semantic handle, hướng pick, History/recompute và persistence. |
| 15 | `gvSmartMSR` Smart MSR | PDF 148–149: bật/tắt filter chỉ target, move/scale và xoay từng handle độc lập | M08 `04-04-004`, `005` | Hiện là chuỗi nhập góc/scale/move một lần, nhận curve thông thường. Cần chế độ filter, gumball từng handle và giữ History. |
| 16 | `gvJoinHistory` Join History | PDF 149: chọn ≥2 curve, giữ cha, tạo child cập nhật theo cha và downstream | M04 `03-13-005`; `01-04` History service | Handler đang Join thường và xóa cha; test yêu cầu xóa. Phải thay semantics và test bằng giữ cha + recipe/recompute; dùng curve nguồn. |
| 17 | `BuilderImageTrace` Image Trace | PDF 149–150: mở ảnh, Threshold/Despeckle/Corner Soften/Invert/Blur/Outline/Preprocess/Prescale/Group/Reset | Chưa thấy L3 riêng; chốt owner, không gán theo trang Join History | Có đọc PNG/BMP/PGM/PPM, threshold và smooth. Thiếu JPG/GIF/TIFF theo manual, preprocess/options, preview cập nhật, group và budget ảnh; alias `gvImageTrace` thiếu. |
| 18 | `BuilderObjectChecker` Object Checker | PDF 150–151: chọn object, Shell và Naked Edges pass/fail; highlight lỗi | AFTER `10-02-005` có ref; M03 health service tái dùng | Có mesh edge/degenerate/volume report. Cần validity/topology nguồn BRep, báo face/edge lỗi, không dùng manifold mesh để chứng nhận BRep. |
| 19 | `BuilderMeshRepair` Mesh Repair | PDF 151: đối tượng → mesh sửa kín, report; tolerance, ExtendedRepair/DeleteInput/SimplePlanes/Milgrain | AFTER `10-02-001`…`005` | Có weld/degenerate/cap/flip từng mesh. Thiếu unite/intersection repair, tolerance mesh nguồn, options, kiểm hậu điều kiện trước thông báo kín; không nhập Netfabb thương mại. |
| 20 | `Builder3dPrinting` 3D Printing | PDF 151–160: mở bộ support tools, support regions, ring/base/cylinder/taper/blend supports | AFTER `10-03-001`…`006` | Có auto sprue và pad theo bbox vật đầu tiên. Thiếu palette, từng generator và control, vùng overhang, nhiều object, kiểm giao/support và export workflow. |
| 21 | `BuilderProfile` Profile | PDF 160–162: đưa curve kín phẳng vào editor hoặc tạo từ library, handles/Select Rail | M04 `04-02-006`, `003`; UI `03-11-007` | Add-on chỉ đặt preset polyline mới; built-in trỏ Profile Placer. Thiếu sửa curve có sẵn, editor/library/save/seam/baseline và giữ identity. |
| 22 | `BuilderProfileMerge` Profile Merge | PDF 162: hai profile kín phẳng chồng nhau → outline mới, giữ gốc | M04 `04-02-006` | Add-on có outline union qua prism mesh; built-in merge section đầu-cuối. Cần curve boolean theo dung sai, holes/coplanarity, giữ gốc; Amount/Flip trên trang chung thuộc End Cap. |
| 23 | `BuilderProfileEndCap` Profile End Cap | PDF 162–163: profile → blended cap, Amount/Flip, preview/handle, Enter; Shift bước 0.10 | M04 `04-02-006` | Add-on có dome mesh; built-in cap phẳng. Cần cap theo semantics surface/curve nguồn, đúng Amount/Flip/handles; kiểm liên tục biên theo mức contract được chốt. |
| 24 | `BuilderCustomRailAdvanced` Custom Rail Advanced | PDF 163–164: rail + Top shape + Side Profile tùy chọn, blends/History; sửa CV shape cập nhật kết quả | M04 `04-02-004`, `008`, `009` | Có nội suy mẫu Top/Side; built-in sao chép rail. Thiếu associative History, blend handles, nguồn curve chính xác và propagation sau edit/reopen. |
| 25 | `BuilderObjectOnCrv` Object on Curve | PDF 164–167: rail + browser/object F4, Start/End/Spacing/Size/Taper/Roll/Z/Mirror/orient modes/styles | M08 `04-07-009` | Có count/spacing/end-scale/twist transforms. Thiếu nhiều mode/options, nguồn arc-length và frame; host hiện transform toàn selection, cần fixture chứng minh rail không bị copy cùng object. |
| 26 | `BuilderSurfaceInset` Surface Inset | PDF 167, phần chung trong ref Object on Curve: surface + curve, inset theo tolerance | Chốt owner; M03 surface service ứng viên | Có pocket mesh CSG thay object; chưa có route BRep, tolerance/trim và bảo toàn loại nguồn. Chuẩn hóa phần ref riêng trước chốt đầy đủ option; không lấy Weld/Reduce trang kế bên làm option Inset. |
| 27 | `BuilderMeshReducer` Mesh Reducer | PDF 167–168: giảm mesh từng mức, ModelUnits/Delete/Reduce/Select/Undo/Weld/WeldAngle/SplitDisjointMeshes | AFTER `10-02-004`; M03 `03-19-006` basic | Có grid clustering và percent; thiếu đủ options, sai số hình học, giữ component/boundary và kiểm topology sau giảm. Mesh người dùng giữ là nguồn mesh. |
| 28 | `BuilderMesh` Mesh Mapper | PDF 168–169: mesh + hai curve Top/Bottom hoặc Left/Right; Maintain Z, Transform/Select/Undo | Chốt owner; M08 `04-07` ứng viên | Handler `to_mesh_command` chỉ copy mesh từ selection, manifest gọi Mesh. Cần mapping thực, source mesh và curve frame; fallback icon đang dùng MeshRepair ID. |
| 29 | `BuilderCurveTransformTool` Curve Transform | PDF 169–171: Hold/Bend Point, Bend/Scale/Straighten/Blend/Mirror Reflect/Rotate và Undo | Chốt owner; M08 `04-08` ứng viên | Có hold/bend/angle theo samples. Thiếu Scale/Straighten/Blend/mirror, handles, ảnh hưởng profile/history và kiểm sai số nguồn. |
| 30 | `BuilderCenterLine` Center Line | PDF 171: surface + hai curve trên mặt, Start; Reset | Chốt owner; M03 curve/surface service ứng viên | Hiện midpoint hai sampled paths hoặc dấu cộng. Cần mean curve rồi pull về mặt đúng trim; decompile có nhánh đủ ba input và fallback MeanCurve, không có căn cứ cho center cross. |
| 31 | `BuilderBooleanBuilder` Boolean Builder | PDF 171–173: Objects/Cutters, Difference/Union/Intersection, KeepCutter/OneByOne và kiểm vật kín | M07 `04-18-004`; `04-16-006` consumer | Có add-on mesh CSG, cache preview và ba mode. Cần BRep route qua kernel hiện có, validity/refusal thay warning-only, invalidation theo source revision, OneByOne/retry và parity Tools/Cutters/F6. |

## Kế hoạch thực hiện và điều kiện đóng từng gói

Các mã T00–T11 là gói công việc của kế hoạch này, không phải milestone ID mới. Mỗi gói cập nhật L3/L4 đúng owner khi làm; không đóng gate người thật bằng test tự động. Các gói AFTER giữ owner và lịch AFTER hiện hành, không trở thành prerequisite cho M04/M07/M08.

### T00 Chốt bảng định tuyến và hợp đồng từng lệnh

Ưu tiên P0 cho phần Ring Rail; phần còn lại làm cùng lệnh tương ứng. Đầu vào là bảng 31 mục và manifest/dispatcher hiện tại. Truy vết binding `BuilderRingRail`, `RingRail`, `gvRingRail`, package Tools/Matrix và đường trực tiếp từ code trước; kiểm thử route tại mốc T11, không chờ đóng toàn bộ bảng 31 route mới triển khai Ring Rail.

- Lập binding table một command icon → owner → aliases → handler add-on/trực tiếp, có cấu hình add-on bật/tắt/không cài. Hai đường được duy trì và triển khai phối hợp. Giữ identifiers cũ; không đổi tên hàng loạt.
- Chuẩn hóa stage table cho từng command: input, prompt, option hợp lệ, current value, accelerator gạch chân, Enter/Esc/Undo/Reset, F6 context. Đối chiếu cả builders và CMD, không dùng index tự trích cũ làm thẩm quyền.
- Chốt owner còn thiếu cho Image Trace, Surface Pullback/Inset, Mesh Mapper, Curve Transform và Center Line; thêm capability/L3 chỉ sau xác nhận owner theo quy ước repo. Bermark cần xác minh nguồn Matrix9; không giả định Book 2 có mục riêng.
- Gắn đúng ref section, tách option trang dùng chung; sửa traceability như `gvInset` đang bị index kéo cả Reduce/Weld và Amount/Flip gán nhầm Profile Merge.
- Đóng gói khi kiểm được 31/31 route trong bảng, alias gốc đã xác minh có test parity, owner chưa chốt được đánh dấu rõ; chưa mở lệnh thiếu implementation như đã hoàn thiện.

### T01 Đưa phép tính add-on về đúng nguồn

Ưu tiên P0 cho capability mà Ring Rail thực sự cần, phụ thuộc phần binding/source contract của chính lệnh trong T00. Các service khác bổ sung theo lệnh sử dụng. Owner M01 `01-02`, `01-03`, `01-05` và ABI add-on; Ring Rail không phải chờ hoàn thiện dịch vụ BRep/mesh hoặc toàn bộ hạ tầng History.

- Tái dùng `Selected.exact` cho curve/surface phù hợp; cung cấp service theo capability cho BRep/derivative/arc-length/pull/measure. Không đưa con trỏ native vào UI hoặc sao chép kernel state sang plugin.
- Với mesh do người dùng tạo/nhập, dùng mesh canonical. Với NURBS/BRep cần mesh tính toán/xuất, dựng theo tolerance riêng có receipt; không lấy `RenderBuffers` mặc định làm nguồn CAD.
- Báo từ chối rõ khi selection bị truncate hoặc không chuyển được nguồn. Giữ placement, units, instance member path và source stamp; không bỏ một phần selection rồi báo thành công.
- Tái dùng host transaction và worker/budget guards hiện có. Chuyển tác vụ nặng ra khỏi `session_step` gọi đồng bộ; preview nhẹ chạy trước nguồn exact, có cancel/stale guard và không thêm Undo khi nâng chất lượng.
- Đóng gói theo command đầu tiên dùng service: cùng nguồn ở Light/High cho cùng phép tính theo tolerance; fixture curve cong/BRep có lỗ/mesh nguồn, placement/instance và quota refusal. Không yêu cầu viết lại toàn lõi trước khi sửa Join History thuần curve.

### T02 Sửa Join History

Ưu tiên P1, thực thi sau Ring Rail; phụ thuộc phần T00 và curve/History cần thiết của T01. Owner M04 `03-13-005`, `01-04`. Giữ các sửa lỗi và bằng chứng đã có; không coi T02 là prerequisite cho Ring Rail.

- Dùng curve canonical; giữ đường cha, tạo child với recipe chứa ordered source IDs và policy join/tolerance. Tái dùng dependency graph/producers/journal hiện có.
- Source edit/recompute cập nhật child và downstream; lỗi không xóa cha, không publish child nửa chừng. Kiểm recipe khi lưu/mở.
- Đổi test hiện đang đòi xóa cha sang oracle từ manual: hai parent IDs còn nguyên, child hợp lệ; di chuyển endpoint làm child/downstream đổi; Undo/Redo/Cancel khôi phục cả geometry và History.
- Nghiệm thu menu/CMD `gvJoinHistory`/F6 nếu có context, và sau reopen. Không đổi thành Join thường để giữ test cũ xanh.

### T03 Thống nhất Metal Weights

Ưu tiên P1, sau Ring Rail và T02; phụ thuộc source measurement của T01. Owner M04 `04-19`, M06 chỉ dùng chung service. Hoàn thiện luồng add-on gọi service đo nguồn dùng chung; giữ các sửa lỗi và bằng chứng đã có.

- Cho built-in và add-on gọi service BRep/density hiện có; thống nhất alias gốc. Thêm Calculate, GV/User/Use Object Materials, report từng object và vật liệu.
- Từ chối open/invalid BRep và thiếu density; không xuất cân nặng có vẻ chính xác chỉ kèm warning. Mesh nguồn có phép đo riêng với validation/uncertainty rõ.
- Oracle: box 10×20×30 mm, volume 6000 mm³; so mass bằng density được khai báo, kiểm DWT theo hệ số đã xác minh. Hai vật liệu, instance/placement, nguồn hở và dữ liệu sai giữ report trước đó; document/Undo không đổi.
- Light/High không làm đổi kết quả BRep; chưa có phép đo mới thì không gắn trạng thái đã đạt.

### T04 Hoàn thiện rail và profile

**T04-A Ring Rail ưu tiên P0 và là lệnh đầu tiên.** Phần rail/profile còn lại ưu tiên P1. Chỉ phụ thuộc phần T00/T01 cần cho capability đang làm; History T02 chỉ liên quan nếu capability sau đó thật sự cần service chung. Owner M04 `04-02`/`03-11`.

#### T04-A Ring Rail phối hợp add-on và trực tiếp

- Đối chiếu M04 `OM9-L3-04-02-001`, entry M03 `03-08-005` và hợp đồng M01 liên quan; dùng local specs đã chuẩn hóa. Chốt bảng input/stage/options/default của Ring Rail trước khi sửa handler; ghi riêng những bảng cỡ/region hoặc nhánh custom chưa đủ căn cứ.
- Điểm sửa phối hợp: `addons/tools/src/lib.rs::ring_rail`, manifest/help trong `addons/tools/addon`, `crates/om9-app/src/session/builder.rs`, `crates/om9-core/src/builder.rs` và `producers.rs::RingRailProducer`; package Matrix dùng lại Tools. Tái dùng và bổ sung service chung cùng kết nối ABI/SDK nếu cần, nối cả adapter add-on và trực tiếp trong cùng đợt triển khai.
- Hoàn thiện region/cỡ, mm/inch, đường kính/chu vi, hướng và vị trí đặt rail; cập nhật preview theo tham số, Enter xác nhận và Esc hủy. Bổ sung custom shape/library và workflow thay rail Yes/No/Cancel theo contract đã xác minh; thiếu nguồn bảng cỡ hoặc tài sản thì báo rõ, không tự tạo dữ liệu thay thế. Các nhánh còn thiếu vẫn là backlog của Ring Rail, không đóng lệnh từ riêng nhánh circle.
- Cả hai đường tạo curve canonical đúng đơn vị và placement. Thao tác tạo/thay rail đi qua một transaction host, giữ định danh/dependency theo contract; hủy hoặc lỗi không để lại kết quả dở dang. F6 sửa lại rail qua route tương ứng và dữ liệu đã lưu; kiểm lưu/mở trước khi đóng capability có persistence.
- Gom triển khai các phần liên quan: (1) tham số/đơn vị + tạo/đặt rail + preview và kết nối cả hai đường; (2) thay rail + sửa lại/F6 + persistence; (3) các region và custom shape/library còn thiếu theo contract. Chỉ bắt đầu kiểm thử khi đã làm được nhiều phần trong đợt hoặc một phần có thể thao tác và xem kết quả trên giao diện. Chỉ đánh dấu toàn lệnh hoàn thành khi đủ các nhánh yêu cầu và nghiệm thu tương ứng.
- Ở mốc kiểm thử, dùng cùng dữ liệu để đối chiếu add-on và trực tiếp: bán kính/chu vi và chuyển đổi mm/inch; đặt rail; thay Yes/No/Cancel; preview không sửa nguồn; menu/CMD; một Undo/Redo; F6 và reopen ở phần sửa lại. Kiểm package Tools độc lập và Matrix gộp Tools. Khi có phần dùng được trên giao diện, kiểm luồng đó trước rồi chạy kiểm thử tự động liên quan để xác nhận hình học/trạng thái. Dùng chính sách T11 bên dưới.

#### T04-B Các rail/profile còn lại

- Sau Ring Rail và T02/T03: Outside Rail/Profile Placer → Profile editor/Custom Rail → Four Rail → Merge/End Cap → Advanced Rail/associative sweep → Ring Resizer.
- Tái dùng builder/producers/handles đã có; bổ sung đúng các gap ở dòng 1–7 và 21–24. Profile Placer phải làm được rail không tròn và đặt từng profile, không chỉ sinh nhóm cố định trên circle.
- Merge dùng overlapping planar profiles và giữ gốc; End Cap có Amount/Flip và kết quả blended phù hợp contract. Kiểm riêng hai nhánh hiện khác nghĩa: mesh dome và planar cap.
- Ring Resizer dùng metal + rail gốc, giữ vùng head theo góc, Symmetrical và nhiều cỡ mesh; file output 3DM/STL có tolerance riêng. Không áp History mới cho resizer khi Book 2 ghi nonhistorical.
- Mỗi capability đóng bằng geometry oracle + preview/handles + menu/CMD/F6 + một Undo + lưu/mở; region/table không có nguồn xác minh thì từ chối rõ.

### T05 Smart Target và Smart Blend có History

Ưu tiên P1; T01 và History service. Owner M08 `04-04-001`…`005`.

- Tạo semantic BlendPoint với handle IDs thay cho Point/Line không liên kết; Add cập nhật cùng target, On Curve End dùng tangent nguồn và vị trí pick.
- Smart Blend ghi dependencies vào đúng handle; thay target cập nhật child. MSR là trạng thái bật/tắt filter, hỗ trợ chọn/rotate từng handle và không chọn nhầm geometry con.
- Nghiệm thu target nhiều handle, hướng pick khác nhau, tangent curve cong, sửa một handle không đổi handle khác; child recompute, Undo/Redo/Cancel, reopen và selection UI thật.

### T06 Surface Pullback và Inset cùng Center Line

Ưu tiên P1; T00 chốt owner và T01. Tái dùng surface/trim/OCCT services hiện có, không ghép ba semantics vào một handler.

- Pullback: input surface + curve kín trên mặt, tạo recess/cap; Drop Depth/Bevel/Insert Only/Cap/Select/Undo.
- Inset: chuẩn hóa riêng section PDF 167 rồi thực hiện tolerance/geometry contract, bảo toàn source type; không đưa mode Mesh Reducer vào command này.
- Center Line: ba input đủ thì mean curve và pull về mặt; hỗ trợ fallback chỉ khi đúng contract đã xác minh, bỏ center-cross giả nghĩa. Oracle trên mặt cong bắt được sai khác giữa midpoint không nằm trên mặt và curve đã pull.
- Fixture: mặt cong có trim/lỗ, curve đảo chiều, off-surface/không kín/invalid; kiểm cả depth, vị trí curve và boundary, một Undo, source unchanged on failure, hai display profiles và xuất khi áp dụng.

### T07 Mesh Repair và Reducer cùng Mesh Mapper

Ưu tiên P1, execution theo AFTER cho `10-02`; Mesh Mapper chốt owner riêng, không biến AFTER thành dependency của mainline.

- Tái dùng các mesh repair/CSG routines đã có sau kiểm tương thích và budget. Repair có before/after counts, united/intersection/island policy và post-validation; thất bại không được thông báo watertight.
- Reducer có unit/options, bound sai số và boundary/component policy. Không suy ra quality chỉ từ số triangle giảm.
- Mesh Mapper thực hiện mesh + top/bottom hoặc left/right curves, Maintain Z và mapping lên surface nếu contract cho phép; giữ nguồn mesh, transform/Select/Undo. Giữ Mesh conversion là lệnh riêng khi phù hợp binding, không dùng nó để thay Mapper.
- Fixtures: lỗ, overlapping shells, normals đảo, nhiều island, UV/seam nếu được hỗ trợ; mapping hai hướng curve và Maintain Z; kiểm error bound/topology/budget/cancel. Add-on mesh và core CSG pass cũ không tự chứng minh workflow này.

### T08 Image Trace và thư viện

Ưu tiên P2; T00 chốt owner và phần source/budget của T01. Có thể triển khai tách hai capability sau khi chốt owner.

- Image Trace: thêm format theo manual, preprocessing/threshold/despeckle/corner soften/blur/outline/invert/prescale/group/reset, preview và giới hạn input/output. Tái dùng decoder có sẵn khi phù hợp; chọn thêm dependency chỉ sau rà giấy phép, không kéo binary pipeline thương mại của bản gốc vào sản phẩm.
- Thư viện: browser/provider tài sản người dùng hoặc được cấp quyền, category/size/material và previews. Giữ presets OM9 như một nguồn được đặt tên đúng; không gọi chúng là bộ dữ liệu Bermark/Stuller đã port.
- Nghiệm thu ảnh có góc/lỗ/speckle với oracle topology/deviation, ảnh hỏng/quá lớn; thư viện thiếu file, import lỗi, undo insert, placement/units và reload catalog.

### T09 Bộ công cụ in 3D

Ưu tiên P2; owner AFTER `10-03`, giữ nguyên lịch milestone.

- Đổi entry thành palette đúng chức năng; làm lần lượt support regions, ring sprue, base/ring, cylinder/tapered, blend supports theo L3 hiện có.
- Tái dùng sprue/pad hiện có đúng vai trò, thêm control/placement và xử lý nhiều object. Mesh in phải có tolerance/validation riêng với display.
- Nghiệm thu overhang được nhận diện, support giao nguồn theo policy, các kích thước generator, Undo từng thao tác và xuất/đọc lại mesh in. Chưa có bài thử in thực thì không tuyên bố chất lượng sản xuất.

### T10 Boolean Builder dùng cùng kernel owner

Ưu tiên P1; owner M07 `04-18-004`, phụ thuộc T01 và dịch vụ Boolean BRep hiện có.

- Giữ UI Object/Cutter và ba mode; route BRep gọi worker/kernel chính thức đã có, mesh nguồn có route mesh riêng. Không suy ra Boolean Builder đã đạt từ test Solid Boolean Union.
- Kiểm vật kín/valid trước chạy; bổ sung KeepCutter, OneByOne/retry và thông báo input lỗi. Cache phải gồm source revision/content, document và input/mode; bbox + vertex count hiện tại không đủ phân biệt hai nguồn khác hình.
- Tái dùng preview nhẹ và publication guards đã có; hủy/stale/quota không sửa nguồn. Cùng owner từ Tools/Cutters/CMD/F6.
- Nghiệm thu ba mode với volume/topology oracle, disjoint/tangent/empty/multiple cutters/placement; một Undo, KeepCutter và lỗi/cancel. Export BRep không được âm thầm nhận mesh Boolean như CAD source.

### T11 Nghiệm thu và đồng bộ tiến độ

**Chỉ bắt đầu kiểm thử sau khi đã triển khai nhiều phần liên quan thành một đợt đủ lớn, hoặc phần đang làm đã có thể kiểm tra trên giao diện.** Trong giai đoạn ghép các thay đổi nhỏ, tập trung triển khai liên tục; không chạy test sau từng helper, file hoặc chỉnh sửa nhỏ. Build/check để giải quyết lỗi biên dịch hoặc mở được giao diện thực hiện khi cần, không biến thành một vòng hồi quy sau mỗi sửa. Tác vụ chỉ sửa kế hoạch không chạy build/test sản phẩm.

Khi đạt mốc, nếu đã có luồng dùng được trên giao diện thì thao tác luồng đó trước; sau đó chạy bộ kiểm thử tự động tập trung cho hình học, transaction và trạng thái của đợt thay đổi. Nếu đợt nhiều phần mới ở tầng service, chạy kiểm thử tập trung cho service/adapter rồi tiếp tục nối giao diện; chưa ghi nghiệm thu UI từ test service. Chạy hồi quy rộng phù hợp một lần khi đợt ổn định; chỉ mở rộng hoặc lặp lại khi lỗi hoặc thay đổi tiếp theo ảnh hưởng bằng chứng đã có.

- Chạy tests core/service, host transaction và real add-on package qua OM9; chứng minh bật/tắt add-on chọn route nào. Test event thuần trong add-on không có document/Undo nên không thay test host.
- Với Ring Rail, tại mốc kiểm thử ưu tiên luồng giao diện đã dùng được, rồi geometry oracle và đối chiếu add-on/trực tiếp với cùng đầu vào. Không lặp bộ Boolean/History/Metal Weights nếu không sửa service dùng chung tác động đến chúng. Kiểm native package ở mốc nghiệm thu tương ứng; lưu bằng chứng còn dùng được và ghi rõ gate pending.
- Kiểm icon đúng biến thể, mặc định, inline options/accelerator, menu/CMD/F6, preview trước exact khi có chờ, Wireframe/Shaded, pick/snap, cancel/stale/quota, Undo/Redo và persistence liên quan. Đo time-to-preview và time-to-commit riêng.
- Với thay đổi phụ thuộc source/display: cùng nguồn/cùng tolerance tính toán và xuất, hai mức hiển thị. STL/3DM là test interoperability; nếu mở batch đo openNURBS thì cập nhật roadmap và báo đúng batch/fixture theo AGENTS.
- Cập nhật `runtime_implementation`, status/traceability, README, live ledger và từng dòng checklist theo phạm vi thực chứng minh. Giữ manual gate pending đến khi reviewer thực ký.
- Build `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, tái dùng tối đa hai target. Chạy public-source audit và prune validation runs sau batch. Không stage nguồn decompile/manual/private assets.
- Sau coding có đổi nghiệp vụ, trình bày before/after và nội dung skill đề xuất để xin lưu theo consent policy; không sửa skill trong tác vụ plan này.

## Điểm bắt đầu cụ thể

Điểm tiếp tục sau đợt 2026-11-17: hai icon trái dùng một core Ring Rail; shared Yes/No/Cancel, current pointer và worker cập nhật dependency trong một Undo đã có 39 kiểm tra tập trung đạt. Native rail/profile/sweep worker đạt 1 fixture; chưa nghiệm thu desktop/package và chưa đóng lệnh. Tiếp tục triển khai custom shape/library theo quy tắc đổi cỡ đã chốt, tiếp tục các region đã có nguồn và nghiệm thu UI. Đọc [bằng chứng hiện tại](../validation/2026-11-17-ring-rail-shared-input.md); giữ toàn bộ phạm vi 31 lệnh và thứ tự ưu tiên bên dưới.

Bắt đầu **T00 cho Ring Rail → phần T01 cần thiết → T04-A Ring Rail phối hợp add-on và trực tiếp**. Gom tham số/đơn vị, tạo/đặt rail và preview thành đợt đầu, nối cả hai route; kiểm thử khi đợt đủ lớn hoặc đã thao tác được trên giao diện. Sau đó hoàn thiện thay/sửa lại và các nhánh còn thiếu. Sau Ring Rail tiếp tục T02, T03, T04-B, T05, T06 và T10; các gói AFTER theo lịch owner.

Tác vụ coding tiếp theo đọc `addons/tools/src/lib.rs::ring_rail`, Ring Rail command/settings trong `addons/tools/addon/om9_addon.toml` và `addons/matrix/addon/om9_addon.toml`, đường trực tiếp trong `crates/om9-app/src/session/builder.rs`, core builder/`RingRailProducer`, cùng M04 `OM9-L3-04-02-001`. Truy vết hai route từ code và chốt luồng tạo/đặt rail, rồi triển khai các phần liên quan trước khi đến mốc kiểm thử T11. Không cần đọc lại toàn PDF hoặc toàn Ghidra export. Việc cập nhật kế hoạch này không tự bắt đầu coding hoặc tiếp tục mục tiêu đang tạm dừng.

## Kiểm tra đã chạy trong lượt audit

| Kiểm tra | Kết quả và giới hạn |
|---|---|
| `cargo test -p om9-addon-tools --lib -j 2` | 10 passed, 0 failed; có test khởi động 30 command nhưng không phải 30 test nghiệm thu |
| `cargo test -p om9-addon-setting-tools --lib boolean -j 2` | 1 passed, 0 failed, 6 filtered; fixture mesh Boolean |
| Inventory 31 icon, 30 Tools command và Boolean owner | Đối chiếu source/manifest; trạng thái DLL thực tế của app chưa kiểm |
| Native menu và Center Line | Exact VA và branch/call đối chiếu VB6/Ghidra; C# dùng token/RVA đúng nguồn, không gán thành VA VB6 |
| Desktop/GPU, Light/High, Undo/reopen các lệnh trong audit | Chưa chạy mới; là điều kiện trong các gói triển khai |

Log và public audit receipt được lưu cùng [inventory](../validation/2026-11-16-tools-menu-audit.json). Không nâng bất kỳ lệnh nào lên hoàn tất từ lượt kiểm tra và lập kế hoạch này.
