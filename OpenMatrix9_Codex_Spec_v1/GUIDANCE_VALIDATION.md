# Kiểm tra gói hướng dẫn — 2026-10-07

607 feature có hợp đồng thiết kế, bảng đầu vào/tham số, hướng dẫn tích hợp,
code Rust riêng và ca nghiệm thu. Các ví dụ dùng thư viện request-planner độc
lập; không được gắn vào production crate hoặc làm command được kích hoạt.

```text
cargo test --manifest-path OpenMatrix9_Codex_Spec_v1/examples/rust/Cargo.toml
```

Kết quả: **12 tests passed**. Bốn kiểm thử tổng hợp chạy trên toàn bộ 607
contract: plan đúng ID/effect, từ chối option lạ, không bỏ qua role bắt buộc
và từ chối NaN cho mọi option số. Tám kiểm thử chung kiểm tra input thiếu,
NaN tại point/option, revision thiếu/cũ, input lặp và snapshot độc lập.
Đây là kiểm thử cấu trúc request, chưa kiểm chứng solver/geometry native.

Kiểm tra tài liệu đã qua: đủ 607 IDs/code blocks, JSON và YAML đồng nhất,
không có liên kết Markdown cục bộ hỏng và không còn dẫn nguồn tài liệu/listing
trong các tài liệu được viết lại. 607 implementation records được giữ nguyên:
16 `partially_implemented` và 591 `not_started`; không tái đánh giá trạng thái
native trong lần này. Hash của ba tệp đầu vào được xác nhận không đổi.

`public_source_audit.py` đã chạy và **chưa qua**: còn 290 artifact riêng/nhị
phân đã có trong cây dự án, gồm ba tệp đầu vào của package và 287 artifact
trong thư mục nghiên cứu. Gói hướng dẫn không được xem là toàn cây sẵn sàng
công bố. MANIFEST chỉ bao gồm tài liệu và mã mẫu được tạo/cập nhật.

Khung C++ commit cần kiểm thử với SDK/native
adapter khi feature tương ứng được triển khai. Unknown defaults, tolerance,
topology và các nhánh chưa có adapter vẫn cần nghiệm thu riêng.

File đầu vào và thông báo giấy phép được bảo toàn. Lần cập nhật hướng dẫn
không tự xác nhận quyền công bố các đầu vào riêng đang có trong cây dự án.

## Đối chiếu nền Rhino thành nhóm 00 — 2026-10-09

Đã viết lại nền Rhino thành [specs/00-rhino-core](specs/00-rhino-core/README.md),
gồm 12 chương RCORE-01–12. Mỗi chương có hợp đồng hành vi/dữ liệu, ma trận
FreeCAD–OM9, điểm vào source, giới hạn, công việc adapter Rust-first và fixture
nghiệm thu. RHINO_CORE_REQUIREMENTS.md giữ vai trò trang chuyển tiếp.

Mức bằng chứng của lần này là `source_inspected`; toàn bộ fixture native mới
có trạng thái `not_run`. Nhãn `UI+API` chỉ xác nhận code giao diện và API đã
được đối chiếu, không xác nhận bản FreeCAD cài sẵn có module đó hoặc hành vi
đã tương đương Rhino. Không chạy build, Rhino roundtrip hay kiểm thử geometry
runtime trong lần viết tài liệu này. Kết quả 12 Rust tests ở phần trên thuộc
lần kiểm tra 2026-10-07, không phải nghiệm thu của nhóm 00.

[CAPABILITY_LOOKUP.json](specs/00-rhino-core/CAPABILITY_LOOKUP.json) là bảng tra
capability riêng, không thêm command vào catalog 607 feature.
[SOURCE_BASELINE.json](specs/00-rhino-core/SOURCE_BASELINE.json) lưu Git HEAD,
metadata phiên bản và SHA256 file nguồn được liên kết. Checkout OM9 có thay
đổi cục bộ; HEAD không phải chứng nhận working tree sạch hoặc binary hiện tại.

Kiểm tra cấu trúc gồm đủ 12 chapter IDs, capability IDs duy nhất, nhãn trạng
thái hợp lệ, fixture IDs, liên kết Markdown cục bộ và hash/byte count trong
MANIFEST. Khi đối chiếu snapshot đầu phiên, FEATURES.json, FEATURES.yaml,
IMPLEMENTATION_STATUS.md và stats.json đã được cập nhật ngoài các chỉnh sửa
nhóm 00 vào 08:52 ngày 2026-10-09; cả bốn khớp manifest hiện tại. Catalog vẫn
có 607 feature, stats hiện ghi 26 `partially_implemented` và 581 `not_started`.
FEATURE_LOOKUP.json và ALL_FEATURES.md giữ hash đầu phiên. Lần viết nhóm 00
giữ nguyên sáu file ở trạng thái quan sát cuối phiên, không ghi đè hoặc lùi
những cập nhật đó. SOURCE_BASELINE lưu cả hash đầu phiên và hash được giữ lại.
Các fixture được mô tả là yêu cầu nghiệm thu; kiểm tra cấu trúc không chạy
các fixture đó và không thay đổi trạng thái native của 607 feature.

Kết quả kiểm cấu trúc: **12 chương, 148 capability, 116 fixture mô tả chưa
chạy, 299 liên kết cục bộ hợp lệ, 121 file evidence có hash và 662 mục
MANIFEST khớp hash/byte count**. SHA256 của PDF người dùng cung cấp vẫn khớp
bản ghi ban đầu; PDF không được sao chép vào thư mục nhóm 00.

Chạy lại `python tools/public_source_audit.py`: exit code **1**, còn **290**
lỗi artifact riêng/nhị phân; danh sách lỗi trùng lần audit trước. Không có
lỗi thuộc thư mục `specs/00-rhino-core`. Audit cũng ghi 510 icon bindings và
510 SVG. Chưa thể coi toàn cây dự án là đã qua kiểm tra nguồn công khai.

## Bổ sung từ code Rhino/Matrix decompile — 2026-10-09

Đã bổ sung 12 chương RCORE và tám feature giao diện: Matrix Interface, Main
Menu, F6, Layers, Project Manager, Shade Mode, View Manager và Layout Tools.
[UI_DETAILS](specs/00-rhino-core/UI_DETAILS.md) mô tả context, Pin/dispatch,
panel/pick/window, numeric controls, mode/material scope và các đường view/layout.
[DECOMPILE_AUDIT](specs/00-rhino-core/DECOMPILE_AUDIT.md) ghi phạm vi và phần
export còn thiếu. Theo yêu cầu người dùng, các phần decompile trình bày
hành vi trực tiếp, không kèm bảng tham chiếu code/dòng/hash.

Đã làm rõ base point Ortho/Planar, unit metadata/scale, requested/effective
Rebuild options, validity preconditions, History Update Off, scale Detail
parallel, material side effects khi đổi shade và các nhánh View Manager/Layout.
Implementation notes của tám feature được điền theo supported slice đã đọc;
defaults local được phân biệt với default UI/document và policy OM9.

Kiểm tra cấu trúc: 12 chương, 148 capability giữ nguyên, 155 ca nền tảng
và 11 ca UI bổ sung, tất cả có trạng thái `not_run`. Frontmatter, exact ID,
status và code Rust mẫu của tám feature được đối chiếu không đổi. Sáu file
catalog/status giữ nguyên SHA256 đầu lần bổ sung này; catalog vẫn có 607
feature, 26 partially_implemented và 581 not_started. Liên kết Markdown
cục bộ, cột bảng và fixture lookup đã được kiểm tra.

Bộ export ngoài repository có 3.529 file và giữ nguyên nội dung. Các kết
luận được đối chiếu nội bộ với 69 file nguồn, phạm vi dòng và hash; số file
kiểm kê chưa có nghĩa mọi thuật toán đã được đọc. Có 76 ghi nhận hành vi/
giới hạn sau khi đọc thêm helper View Manager và các handler Layout.
Không đưa source decompile, binary hoặc bitmap thương mại vào package.

MANIFEST được cập nhật cho tài liệu lần này và có 664 mục khớp hash/bytes.
Đồng thời đồng bộ chín record cũ chưa khớp file đã có trước snapshot đầu
lần này: FEATURES.json/YAML, IMPLEMENTATION_STATUS và sáu feature spec khác.
Nội dung của chín file đó được giữ nguyên, không ghi đè bằng bản cũ.
SOURCE_BASELINE vẫn là snapshot đối chiếu trước; sáu file OM9 dẫn trong
snapshot đã đổi, còn các file lõi FreeCAD trong baseline giữ hash. Lần này
không tái đánh giá native từ những chỉnh sửa implementation khác.

Chạy `tools/public_source_audit.py`: exit code 1, còn 290 lỗi artifact riêng/
nhị phân; tập lỗi trùng bản audit trước và không có lỗi thuộc nhóm 00.
Audit hiện ghi 511 icon bindings và 510 SVG. Build, native UI, geometry,
renderer, Rhino 3DM roundtrip và các fixture mô tả chưa chạy; kiểm tra tài
liệu không nâng trạng thái native hoặc thay nghiệm thu các chức năng đó.

## Kiểm tra lại native bridge sau export mới — 2026-10-09

Bản export mới đã có thân hàm native: file C có 3.603.082 byte và file
header có 545.387 byte. Kiểm kê hiện có 3.530 file; số 3.529 ở lần kiểm
tra trước mô tả bộ export trước khi người dùng thay bản C và thêm header.
Đã kiểm tra trực tiếp thân hàm RebuildCurve, FitCurve, RebuildSurface,
FitSurface, kiểm tra tính hợp lệ BRep và RebuildEdges. Số 2.763 tên export
gắn với thân hàm trong bản C là kết quả kiểm kê văn bản, không chứng nhận
mọi hàm đã được decompile đúng hoặc mọi thuật toán đã được đọc.

Đã cập nhật bảy chương RCORE-01, 02, 03, 05, 06, 07 và 08, cùng README và
DECOMPILE_AUDIT. Nội dung làm rõ thứ tự degree/count, chuẩn hóa số control
point/degree, tolerance của Fit, chuyển đổi NURBS, scale đơn vị, selector
kiểm tra BRep, ràng buộc face index, CPlane và vòng đời callback khi Get
trả về bình thường. Các ghi chú thiếu thân hàm bridge đã được sửa. Những
nhánh gọi tiếp vào SDK/kernel và giới hạn suy luận kiểu ABI vẫn được nêu;
không coi output tolerance của SurfaceFit là chứng nhận sai số lớn nhất.
Phần giải thích hành vi tiếp tục không kèm tham chiếu code decompile.

Kiểm tra cấu trúc và đối chiếu nội bộ: 12 chương, 148 capability, 155 ca
nền tảng và 11 ca UI giữ nguyên; sáu file catalog/status giữ nguyên hash
đầu lần kiểm tra này. Catalog vẫn có 607 feature, 26 partially_implemented
và 581 not_started. Capability rows, fixture IDs, liên kết cục bộ, cột
bảng, hash nguồn và phạm vi dòng bằng chứng đã được kiểm tra. MANIFEST
giữ 664 mục, chỉ cập nhật hash/byte count cho tài liệu của lần này.

Chạy lại tools/public_source_audit.py: exit code 1 với cùng tập 290 lỗi
artifact riêng/nhị phân; không có lỗi thuộc nhóm 00. Build, native UI,
geometry runtime, Rhino 3DM roundtrip và các fixture chưa chạy. Trạng
thái nghiệm thu runtime vẫn là not_run; đọc code không nâng trạng thái
triển khai của FreeCAD hoặc OM9.

## Tách nguồn tham chiếu riêng khỏi workspace — 2026-10-09

Theo yêu cầu người dùng, toàn bộ thư mục code tham chiếu Matrix và ba PDF
gốc đã được bảo toàn ở ngoài FreeCAD-src. Đã sao chép 10.662 file, đối chiếu
SHA256 và byte count của cả nguồn/bản sao, rồi gỡ bản tương ứng trong
workspace. Manifest bảo toàn nằm cùng kho riêng; bộ specs chỉ giữ tài liệu
phân tích và code do OpenMatrix9 viết. Thao tác không chứng nhận mọi thông
tin trong các nguồn đã được diễn giải hết; nguồn còn nguyên để đọc bổ sung.

[Quy tắc nguồn riêng](../docs/private-reference-policy.md) và registry cục bộ
được Git ignore giúp giải quyết các tên nguồn/đường dẫn lịch sử. Guide có ví
dụ decompile hiện được ignore trong khi chờ quyết định chuyển riêng; không
đưa guide vào commit. Không commit, push hoặc sửa lịch sử Git trong lần này.

Audit public source sau khi tách nguồn trả về exit code 0, không có lỗi;
290 mục của các lần trước mô tả trạng thái trước khi di chuyển. Việc audit
đạt chỉ xác nhận các quy tắc kỹ thuật của script, không thay nghiệm thu
native, chứng nhận toàn bộ source hay xác minh giấy phép. Catalog, trạng
thái triển khai, fixture và file thực thi không được thay đổi bởi thao tác
di chuyển; MANIFEST specs giữ 664 mục.

## Giữ hướng dẫn Rhino 5 công khai trong ref — 2026-10-09

Theo chỉ dẫn mới của người dùng, bản User's Guide Rhino 5 gốc được giữ ở
[ref/rhino5](../ref/rhino5/README.md). PDF có 284 trang, 13.533.999 byte;
SHA256 của bản người dùng cung cấp khớp bản tải chính thức từ McNeel. Bản
PDF và các thông báo gốc không bị chỉnh sửa. Metadata nguồn ghi rõ tài liệu
công khai để tải xuống và không đổi giấy phép của tài liệu thành LGPL.

Bảng tra có 12 RCORE chapter IDs, trang PDF/trang in, Command Help đúng
phiên bản và các bước đối chiếu input/defaults/units/tolerance, source host
và fixture. Những phần không đủ coverage trong User's Guide được chỉ sang
Help/API/fixture thay vì gán hành vi chưa chứng minh cho PDF. Các liên kết
Help đã đọc, chỉ thấy trong PDF hoặc không fetch được có trạng thái riêng.

AGENTS, README core và quy tắc nguồn được nối tới bảng tra. Audit và Git
ignore chỉ cho phép đúng bản PDF đã ghim hash/bytes cùng ba file metadata
được nêu tên; không bỏ chặn toàn bộ ref hay PDF. Các nguồn Matrix, code
decompile và tài nguyên riêng tiếp tục ở ngoài workspace hoặc bị ignore.
Việc bổ sung tài liệu không đổi catalog, fixture, SOURCE_BASELINE lịch sử
hoặc checkpoint/native status hiện hành; không commit/push trong lần này.


Kiểm tra tooling đã qua 31 tests; source audit hiện có zero errors. Khi cập
nhật MANIFEST, sáu record chapter đã không khớp từ trước snapshot lần giữ
hướng dẫn này được đồng bộ với nội dung P0 hiện hành: 01, 04, 05, 07, 09 và
10. Chỉ hash/byte count được đồng bộ; nội dung và kết quả P0 của sáu chương
được giữ nguyên. Sáu file catalog/status, CAPABILITY_LOOKUP và SOURCE_BASELINE
giữ hash snapshot đầu lần này; không nâng trạng thái bằng chứng do thêm PDF.
