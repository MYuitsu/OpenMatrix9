> **OpenMatrix9 0.0.1 (experimental, Windows x64)**: [Download release](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.1) · [Installation guide](INSTALL.vi.md). Use the matching FreeCAD portable host supplied with this release.

# OpenMatrix9 — Tiếng Việt

[English](README.en.md) · [Danh sách chi tiết từng lệnh](README.md)

![OpenMatrix9 — Workbench CAD trang sức nguồn mở](Resources/branding/introduction.png)

OpenMatrix9 là workbench CAD trang sức đang phát triển cho FreeCAD, lấy Matrix9
làm tham chiếu chức năng. Rust quản lý danh mục lệnh, trạng thái và hành vi;
C++/Qt tích hợp hình học và giao diện native FreeCAD; Python đăng ký workbench.

## Mục lục

- [Tổng quan](#overview)
- [Ủng hộ phát triển / Donate](#donate)
- [Quy ước tính hoàn thành](#completion-rules)
- [Bảng theo dõi theo nhóm lớn](#group-progress)
- [Chức năng đang sử dụng được](#supported-workflows)
- [Tương thích Rhino 3DM / openNURBS](#three-dm)
- [Build và cài đặt](#build)
- [Kiểm tra](#checks)
- [Tài liệu và cập nhật tiến độ](#documentation)
- [Giấy phép](#support-license)

<a id="overview"></a>
## Tổng quan

Giao diện tổ chức lệnh theo nhóm CAD và trang sức, có vùng CMD, bốn viewport,
menu F6 và các điều khiển hiển thị. Lệnh chưa được hỗ trợ vẫn hiển thị nhưng
bị vô hiệu hóa. Dự án có **510 liên kết icon SVG tự thiết kế**; số icon
không phải số chức năng đã hoàn thành.

<a id="donate"></a>
## Ủng hộ phát triển / Donate

Nếu OpenMatrix9 hữu ích với bạn, bạn có thể donate để hỗ trợ phát triển và kiểm
thử các chức năng CAD, import/export 3DM và tương thích openNURBS.
Mọi đóng góp đều tự nguyện. Cảm ơn bạn đã hỗ trợ dự án!

- **PayPal:** gửi tới **`nguyenthaiduy277@gmail.com`** trong PayPal.
- **MoMo:** quét mã QR dưới đây. Tên người nhận: **NGUYEN THAI DUY**.

![Mã QR MoMo donate cho NGUYEN THAI DUY](docs/images/donate/momo-qr.png)

<a id="completion-rules"></a>
## Quy ước tính hoàn thành

Cập nhật cho source gộp ngày **2026-10-09**, gồm nhánh CAD/P0 và phần 3DM
nâng cao từ main. Các báo cáo kiểm chứng giữ nguyên phiên bản và phạm vi
kiểm thử ban đầu; số kiểm thử trước đây không xác nhận phiên bản sau gộp.
Nền tảng P0 và tương thích đầy đủ Matrix9/Rhino vẫn chưa hoàn tất.

Bảng dưới đây tổng hợp
[bảng chi tiết từng lệnh](README.md#tiến-độ-lệnh--command-progress), đối chiếu với
[hồ sơ kiểm chứng](docs/openmatrix9-progress.json) và
[trạng thái Spec v1](OpenMatrix9_Codex_Spec_v1/IMPLEMENTATION_STATUS.md).

- **Hoàn thành phạm vi hiện tại:** mục 🟢 trong bảng chi tiết, có đường thực thi
  và kiểm tra cho phạm vi được ghi rõ. Các tùy chọn còn thiếu vẫn được giữ trong
  tài liệu từng lệnh; trạng thái này không xác nhận đầy đủ tương thích Matrix9/Rhino.
- **Một phần:** mục 🟡, đã có nền tảng nhưng còn giới hạn hoặc phần chờ tích hợp.
  Không cộng mục này vào số hoàn thành.
- **Chưa hoàn thành:** mục 🔴, chưa có đường thực thi được xác nhận trong bảng nguồn.
- **Tổng:** số mục catalog hoặc lệnh bổ sung được liệt kê trong từng nhóm.
  Tỷ lệ = số hoàn thành phạm vi hiện tại / tổng của nhóm × 100, làm tròn một chữ số.

Một mã có thể xuất hiện ở nhiều nhóm. Không cộng các dòng để tính tiến độ toàn
dự án: danh sách có **514 mã duy nhất**, gồm **64 🟢, 5 🟡, 445 🔴**.
Editor dùng chung danh mục Curve; Custom và Reset là điều khiển sidebar,
không được tính thêm thành nhóm chức năng CAD.

Spec v1 dùng đơn vị khác: **607 feature**, hiện ghi **32 `partially_implemented`** và **575 `not_started`**. Một feature có thể
đã kiểm chứng phần hỗ trợ nhưng vẫn triển khai một phần theo toàn bộ spec.

<a id="group-progress"></a>
## Bảng theo dõi theo nhóm lớn

Mỗi dòng là một nhóm lớn. Nhấn tên nhóm để xem từng lệnh, phạm vi hỗ trợ và giới hạn.

| STT | Nhóm chức năng lớn | Hoàn thành phạm vi hiện tại / Tổng | Tỷ lệ | Một phần | Chưa hoàn thành |
|---:|---|---:|---:|---:|---:|
| 1 | [Thao tác nhanh (Quick tools)](README.md#quick-tools--thao-tác-nhanh) | **3/11** | 27.3% | 0 | 8 |
| 2 | [Tệp (File)](README.md#file) | **5/13** | 38.5% | 0 | 8 |
| 3 | [Khung nhìn (View)](README.md#view) | **10/15** | 66.7% | 0 | 5 |
| 4 | [Tiện ích (Utilities)](README.md#utilities) | **0/22** | 0.0% | 0 | 22 |
| 5 | [Đo lường (Measure)](README.md#measure) | **2/17** | 11.8% | 0 | 15 |
| 6 | [Đường cong (Curve)](README.md#curve) | **7/58** | 12.1% | 0 | 51 |
| 7 | [Bề mặt (Surface)](README.md#surface) | **3/40** | 7.5% | 0 | 37 |
| 8 | [Khối đặc (Solid)](README.md#solid) | **6/30** | 20.0% | 0 | 24 |
| 9 | [Biến đổi (Transform)](README.md#transform) | **0/40** | 0.0% | 0 | 40 |
| 10 | [Clayoo / SubD](README.md#clayoo--subd) | **0/58** | 0.0% | 0 | 58 |
| 11 | [Chạm nổi (Emboss)](README.md#emboss) | **0/7** | 0.0% | 0 | 7 |
| 12 | [Bộ dựng (Builder)](README.md#builder) | **0/16** | 0.0% | 0 | 16 |
| 13 | [Công cụ trang sức (Tools)](README.md#tools) | **1/31** | 3.2% | 0 | 30 |
| 14 | [Đá quý (Gems)](README.md#gems) | **0/35** | 0.0% | 0 | 35 |
| 15 | [Ổ đá và chấu (Settings)](README.md#settings--ổ-đá-và-chấu) | **0/12** | 0.0% | 0 | 12 |
| 16 | [Công cụ cắt (Cutters)](README.md#cutters) | **0/11** | 0.0% | 0 | 11 |
| 17 | [Kết xuất (Render)](README.md#render) | **0/23** | 0.0% | 0 | 23 |
| 18 | [Lõi / dự án / chọn đối tượng](README.md#core--project--selection) | **5/11** | 45.5% | 0 | 6 |
| 19 | [Khung nhìn và hiển thị bổ sung](README.md#view-và-display-bổ-sung) | **10/22** | 45.5% | 0 | 12 |
| 20 | [Bắt điểm / ràng buộc](README.md#snap--ràng-buộc) | **4/17** | 23.5% | 0 | 13 |
| 21 | [Thuộc tính / lịch sử / thiết lập / lớp](README.md#properties--history--settings--layers) | **7/26** | 26.9% | 0 | 19 |
| 22 | [Lệnh 3DM chuyên biệt / bàn phím](README.md#3dm-chuyên-biệt--keyboard) | **1/3** | 33.3% | 2 | 0 |


Join/Surface History policy native và Explode adapters đã kiểm chứng supported slice trên SDK; xem hợp đồng [History](docs/features/history-native-contract.md) và [Edit](docs/features/edit-native-contract.md).

<a id="supported-workflows"></a>
## Chức năng đang sử dụng được

- **Dự án và giao diện:** New, Open, Save, Save As, Notes, Undo/Redo, chọn/xóa
  đối tượng, bốn viewport, các view cơ bản, zoom, ảnh tham chiếu và menu F6.
- **Đường cong:** Line, Polyline, Interp Curve, Rebuild, Rectangle, Circle và Ellipse
  trong phạm vi được ghi tại [tài liệu chức năng](docs/features/).
- **Bề mặt và khối:** Sweep1, Sweep2, Loft, Box, Sphere và bốn lệnh Boolean;
  xem [trạng thái triển khai](OpenMatrix9_Codex_Spec_v1/IMPLEMENTATION_STATUS.md).
- **Chỉnh sửa và đo:** Trim, Join, Explode, Distance, Angle; bắt điểm End,
  Point, Midpoint và ràng buộc Ortho trong các công cụ được hỗ trợ.

Các nhánh nâng cao, History liên kết, lớp và những adapter đặc biệt còn giới
hạn riêng. Xem [hợp đồng Edit](docs/features/edit-native-contract.md) và
[các báo cáo kiểm chứng](docs/validation/) trước khi dựa vào một tùy chọn cụ thể.

<a id="three-dm"></a>
## Tương thích Rhino 3DM / openNURBS

Import/export 3DM đang được triển khai một phần. Phạm vi hiện có gồm chuyển đổi
points/curves, surface/BRep và mesh được hỗ trợ, metadata hình học cơ bản,
snapshot nguồn trong FCStd và xuất hình học nhắm archive Rhino 5.

Giữ dữ liệu trong snapshot không đồng nghĩa đã hiển thị, chỉnh sửa hoặc xuất lại
đầy đủ. Structural block preservation, materials/textures, annotation và
History/plugin userdata còn giới hạn. Xem phần trao đổi 3DM trong
[README chi tiết](README.md) để đối chiếu phạm vi nâng cao đã hỗ trợ,
bằng chứng ứng dụng Rhino 5 riêng và các phần còn thiếu.

<a id="build"></a>
## Build và cài đặt

Đọc [hướng dẫn Windows](docs/build-windows.md) để chọn build FreeCAD cùng OM9,
build OM9 riêng với SDK, hoặc dùng binary tương thích với bản FreeCAD đã cài.
Module cần SDK/ABI khớp; chỉ clone vào `Mod` chưa đủ để sử dụng.

Baseline tài liệu là Windows x64, MSVC và Qt6, với Rust hỗ trợ edition 2024,
CMake, Python và dependencies của đúng FreeCAD SDK. Linux/macOS chưa được
kiểm chứng cho module này. Chi tiết tích hợp: [build và validation](docs/build.md).

<a id="checks"></a>
## Kiểm tra

Chạy tại thư mục OpenMatrix9, trong môi trường build tương ứng:

```powershell
cargo test --manifest-path rust/Cargo.toml
python -m unittest discover -s tools/tests
python tools/public_source_audit.py
```

Các macro native và cách chọn executable/dependencies được mô tả trong
[hướng dẫn kiểm tra](docs/build.md). Số test đã qua xác nhận phạm vi kiểm thử,
không thay thế số chức năng hoàn thành. Workspace hiện còn artifact tham chiếu
riêng được source audit ghi nhận; xem kết quả audit và hồ sơ tiến độ trước khi phát hành.

<a id="documentation"></a>
## Tài liệu và cập nhật tiến độ

- [README chi tiết](README.md): từng mã catalog, ý nghĩa, trạng thái và giới hạn.
- [Ledger tiến độ](docs/openmatrix9-progress.json): bằng chứng, lần chạy test,
  phần chưa kiểm chứng và bước tiếp theo.
- [Chỉ mục Spec v1](OpenMatrix9_Codex_Spec_v1/INDEX.md): tra cứu feature theo nhóm và ID.
- [Báo cáo native](docs/validation/): phạm vi kiểm chứng của từng đợt triển khai.
- [Build Windows](docs/build-windows.md) và [build/validation](docs/build.md).

Sau khi triển khai và kiểm tra một chức năng, cập nhật ledger, spec liên quan
và bảng chi tiết trong `README.md`; sau đó đồng bộ số liệu, ngày và giới hạn
trong cả `README.vi.md` và `README.en.md`. Giữ cùng đơn vị đếm và cùng trạng thái
ở hai bản; không nâng trạng thái chỉ vì có icon, tên lệnh hoặc test validator.

<a id="support-license"></a>
## Giấy phép

Dự án khai báo **LGPL-2.1-or-later**: xem [LICENSE](LICENSE) và
[thông báo bên thứ ba](THIRD_PARTY_NOTICES.md). OpenMatrix9 là dự án độc lập;
không tuyên bố liên kết với nhà cung cấp Matrix9, phát triển clean-room hoặc
đã được xác nhận pháp lý cho việc triển khai. Bối cảnh nguồn và publication
audit được ghi trong [README chi tiết](README.md) và
[source review](docs/public-source-review.md).
