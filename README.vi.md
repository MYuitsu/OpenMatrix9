# OpenMatrix9

**CAD trang sức nguồn mở trên FreeCAD.** Rust xử lý logic; FreeCAD cung cấp hình học và giao diện.

[English](README.en.md) · [Cài đặt](#installation) · [Releases](https://github.com/MYuitsu/OpenMatrix9/releases)

![OpenMatrix9](Resources/branding/introduction.png)

## Mục lục

- [Donate](#donate)
- [Cài đặt chi tiết](#installation)
- [Tiến độ](#progress)
- [Release notes](#release-notes)
- [Giấy phép](#license)

<a id="donate"></a>
## Donate

Ủng hộ phát triển CAD, 3DM và công cụ trang sức. Mọi đóng góp đều tự nguyện. Cảm ơn bạn!

- **PayPal:** gửi tới **`nguyenthaiduy277@gmail.com`** trong PayPal.
- **MoMo:** người nhận **NGUYEN THAI DUY**; quét mã dưới đây.

<img src="docs/images/donate/momo-qr.png" alt="MoMo — NGUYEN THAI DUY" width="240">

<a id="installation"></a>
## Cài đặt

### 1. Chuẩn bị FreeCAD

Gói phát hành hiện tại dành cho **Windows x64 và FreeCAD 1.1.4 chính thức**.
Bạn không cần source FreeCAD, Rust hoặc Visual Studio để cài và sử dụng.

1. Mở [Release FreeCAD 1.1.4](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4),
   tải bộ cài **Windows x64** và cài FreeCAD theo hướng dẫn của bộ cài.
2. Mở FreeCAD một lần. Vào **Help → About FreeCAD**, kiểm tra phiên bản **1.1.4**.
   Thư mục cài đặt có thể mang tên `FreeCAD 1.1` dù phiên bản là 1.1.4.
3. Lưu bản vẽ và đóng FreeCAD trước khi cài hoặc thay plugin OM9.

### 2. Tải SetupLauncher

Mở **[Release 0.3.5-202610121520](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.3.5-202610121520)**, bấm **Assets** và tải gói **SetupLauncher**:

| File | Tải xuống |
|---|---|
| **Launcher — khuyên dùng** | [OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.3.5-202610121520/OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip) |
| Đối chiếu SHA256 | [SHA256SUMS.txt](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.3.5-202610121520/SHA256SUMS.txt) |

Bản **Source code** dùng cho phát triển; cài OM9 bằng ZIP **SetupLauncher** ở trên. Bản phát hành này được đánh dấu **Pre-release**.

### 3. Cài bằng launcher

1. Nhấp phải ZIP **SetupLauncher** → **Extract All / Giải nén tất cả**.
   Giải nén toàn bộ trước khi mở launcher.
2. Trong thư mục đã giải nén, giữ **`OpenMatrix9`**, **`launcher-manifest.json`**
   và các file **`Start-OM9`** cùng nhau.
3. Nhấp đúp **`Start-OM9.cmd`**.
4. Launcher cài OM9 vào `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, tạo icon
   **OpenMatrix9** trên **Desktop** và **Start Menu**, rồi mở workbench.
5. Những lần sau, mở bằng icon **OpenMatrix9**. Sau khi cài thành công, bạn có
   thể bỏ thư mục ZIP đã giải nén; plugin đã được cài trong AppData.

OM9 được cài trong thư mục người dùng; không cần chạy launcher hoặc FreeCAD bằng
quyền Administrator. Nếu launcher báo plugin đang được dùng, lưu và đóng FreeCAD
rồi chạy lại; launcher giữ ứng dụng đang mở của bạn.

### 4. Mở và kiểm tra lần đầu

1. Mở **OpenMatrix9** bằng icon đã tạo, hoặc mở FreeCAD 1.1.4 và chọn
   **OpenMatrix9** trong danh sách workbench.
2. Tạo tài liệu mới bằng **File → New**.
3. Chọn **Line**, đặt hai điểm, rồi thử **Undo** và **Redo**.
4. Kiểm tra sidebar, vùng **Command**, viewport và bảng **Layers**.
   Các mục menu bị vô hiệu hóa là chức năng đang được phát triển.
5. Lưu thử dự án `.FCStd`. Hoàn tất hoặc hủy lệnh vẽ trước khi chuyển tài liệu.

Để xác nhận đúng plugin, mở **View → Panels → Python console**, nhập:

```python
import OpenMatrix9Gui
print(OpenMatrix9Gui.__file__)
```

Với cách cài trên, đường dẫn phải trỏ tới `v1-1\Mod\OpenMatrix9\bin\OpenMatrix9Gui.pyd`.

### 5. Cập nhật, khôi phục và gỡ cài đặt

- **Cập nhật:** tải SetupLauncher của Release mới, giải nén toàn bộ, lưu và đóng
  FreeCAD, rồi chạy `Start-OM9.cmd`. Launcher báo bản mới ở nền nhưng việc tải và
  cài vẫn do bạn chủ động thực hiện.
- **Bản sao cũ:** launcher lưu trong `%APPDATA%\FreeCAD\v1-1\OM9-backups`.
  Khi cần khôi phục, đóng FreeCAD, chuyển plugin hiện tại ra ngoài `Mod`, rồi đặt
  bản sao đã chọn trở lại `Mod\OpenMatrix9`, giữ đúng tên thư mục `OpenMatrix9`.
- **Log launcher:** `%APPDATA%\FreeCAD\v1-1\OM9-launcher`.
- **Gỡ plugin:** đóng FreeCAD và chuyển `OpenMatrix9` ra ngoài `Mod`.
  Nếu đã dùng launcher, bỏ các shortcut OM9 khi không còn dùng launcher.
  Các dự án `.FCStd` bạn đã lưu vẫn được giữ.

### 6. Kiểm tra file tải về và lỗi thường gặp

Tải `SHA256SUMS.txt` từ cùng Release. Trong PowerShell tại thư mục Downloads:

```powershell
Get-FileHash -LiteralPath '.\OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip' -Algorithm SHA256
Get-Content -LiteralPath '.\SHA256SUMS.txt'
```

So giá trị `Hash` với dòng đúng tên ZIP trong `SHA256SUMS.txt`.

| Hiện tượng | Cách kiểm tra |
|---|---|
| Không thấy OpenMatrix9 | Kiểm tra FreeCAD 1.1.4, `v1-1\Mod`, `InitGui.py`, rồi khởi động lại. |
| `No module named OpenMatrix9Gui` | Chạy lại `Start-OM9.cmd` từ gói SetupLauncher đã giải nén đầy đủ. |
| `DLL load failed` | Dùng gói Windows x64 đúng FreeCAD 1.1.4; lấy thông báo từ **Report view**. |
| Thiếu icon hoặc menu | Kiểm tra thư mục `Resources` và giữ toàn bộ nội dung của gói. |
| `3DM import worker is missing` | Kiểm tra `bin\OM9ThreeDmImportWorker.exe`; giải nén lại từ cùng ZIP nếu thiếu. |
| Launcher không cài | Giữ `OpenMatrix9`, manifest và các file Start-OM9 cùng nhau; xem log ở bước 5. |
| Cập nhật báo file đang dùng | Lưu và đóng FreeCAD đang dùng OM9, rồi chạy lại launcher. |

Mở **View → Panels → Report view** để đọc lỗi. Khi báo lỗi, gửi phiên bản FreeCAD,
tên ZIP và thông báo trong panel. Giữ plugin và host đúng bản; không chép DLL từ
một phiên bản FreeCAD khác để chữa lỗi tải module.

**Dành cho phát triển:** [build riêng OM9 với SDK chung](docs/build-windows.md).

<a id="progress"></a>
## Tiến độ

**Gumball 0.3.5**: Giữ **Shift** rồi click thêm object hoặc kéo khung chọn để dùng một Gumball chung trong cả bốn view. Move, Scale và Rotate áp dụng quanh tâm cả cụm; một Undo hoàn tác cả thao tác. Đã kiểm tra tại máy: **848 test Rust**, **31 ca native / 1.099 kiểm tra**. [Bằng chứng](docs/validation/2026-10-12-gumball-multi-selection.md). GitHub Actions chỉ build và đóng gói.


**Đang làm:** Core/History — vòng đời đối tượng, rollback và giới hạn tài nguyên.

```mermaid
flowchart LR
    A["Core & History<br/>Đang phát triển"] --> B["CAD & 3DM<br/>Có phạm vi hỗ trợ"]
    B --> C["Builders & Gems<br/>Chưa hoàn tất"]
    classDef active fill:#e8f2ff,stroke:#2563eb,color:#17376b
    classDef available fill:#e6f6ef,stroke:#20845c,color:#14543b
    classDef planned fill:#f3f4f6,stroke:#9ca3af,color:#4b5563
    class A active
    class B available
    class C planned
```

**Snapshot workspace: 2026-10-12.** Đếm phạm vi hỗ trợ đã kiểm chứng, không xác nhận
toàn bộ feature. Một mã có thể xuất hiện ở nhiều nhóm. Phạm vi binary xem Release.

| Nhóm | Đã làm / Tổng | Tiến độ |
|---|---:|---|
| [Thao tác nhanh](docs/releases/2026-10-12-development.md#quick-tools--thao-tác-nhanh) | **3/11** | ▰▰▰▱▱▱▱▱▱▱ 27.3% |
| [Tệp](docs/releases/2026-10-12-development.md#file) | **5/13** | ▰▰▰▰▱▱▱▱▱▱ 38.5% |
| [Khung nhìn](docs/releases/2026-10-12-development.md#view) | **10/15** | ▰▰▰▰▰▰▰▱▱▱ 66.7% |
| [Tiện ích](docs/releases/2026-10-12-development.md#utilities) | **0/22** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Đo lường](docs/releases/2026-10-12-development.md#measure) | **2/17** | ▰▱▱▱▱▱▱▱▱▱ 11.8% |
| [Đường cong](docs/releases/2026-10-12-development.md#curve) | **7/58** | ▰▱▱▱▱▱▱▱▱▱ 12.1% |
| [Bề mặt](docs/releases/2026-10-12-development.md#surface) | **3/40** | ▰▱▱▱▱▱▱▱▱▱ 7.5% |
| [Khối đặc](docs/releases/2026-10-12-development.md#solid) | **6/30** | ▰▰▱▱▱▱▱▱▱▱ 20.0% |
| [Biến đổi](docs/releases/2026-10-12-development.md#transform) | **0/41** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Clayoo / SubD](docs/releases/2026-10-12-development.md#clayoo--subd) | **0/58** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Chạm nổi](docs/releases/2026-10-12-development.md#emboss) | **0/7** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Bộ dựng](docs/releases/2026-10-12-development.md#builder) | **0/16** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Công cụ trang sức](docs/releases/2026-10-12-development.md#tools) | **1/31** | ▱▱▱▱▱▱▱▱▱▱ 3.2% |
| [Đá quý](docs/releases/2026-10-12-development.md#gems) | **0/35** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Ổ đá và chấu](docs/releases/2026-10-12-development.md#settings--ổ-đá-và-chấu) | **0/12** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Công cụ cắt](docs/releases/2026-10-12-development.md#cutters) | **0/11** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Kết xuất](docs/releases/2026-10-12-development.md#render) | **0/23** | ▱▱▱▱▱▱▱▱▱▱ 0.0% |
| [Lõi / dự án / chọn](docs/releases/2026-10-12-development.md#core--project--selection) | **5/11** | ▰▰▰▰▰▱▱▱▱▱ 45.5% |
| [Hiển thị bổ sung](docs/releases/2026-10-12-development.md#view-và-display-bổ-sung) | **10/22** | ▰▰▰▰▰▱▱▱▱▱ 45.5% |
| [Bắt điểm / ràng buộc](docs/releases/2026-10-12-development.md#snap--ràng-buộc) | **4/17** | ▰▰▱▱▱▱▱▱▱▱ 23.5% |
| [Thuộc tính / lịch sử / lớp](docs/releases/2026-10-12-development.md#properties--history--settings--layers) | **7/26** | ▰▰▰▱▱▱▱▱▱▱ 26.9% |
| [3DM / bàn phím](docs/releases/2026-10-12-development.md#3dm-chuyên-biệt--keyboard) | **1/3** | ▰▰▰▱▱▱▱▱▱▱ 33.3% |

<a id="release-notes"></a>
## Release notes

- [Release 0.3.5-202610121520](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.3.5-202610121520): thay đổi, gói tải xuống và giới hạn của bản phát hành.
- [Checkpoint phát triển](docs/releases/2026-10-12-development.md): bảng từng lệnh, lịch sử và bằng chứng đã chuyển từ README.
- [Ledger](docs/openmatrix9-progress.json) · [Báo cáo native](docs/validation/) · [Spec](OpenMatrix9_Codex_Spec_v1/INDEX.md).

<a id="license"></a>
## Giấy phép

[LGPL-2.1-or-later](LICENSE) · [Thông báo bên thứ ba](THIRD_PARTY_NOTICES.md).
OpenMatrix9 là dự án độc lập, dùng Matrix9 làm tham chiếu chức năng; không tuyên bố
liên kết nhà cung cấp, clean-room hoặc xác nhận pháp lý. [Bối cảnh nguồn](docs/public-source-review.md).
