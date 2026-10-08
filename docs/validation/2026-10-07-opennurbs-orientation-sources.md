# Tài liệu thay thế McNeel API: hướng solid và phản chiếu 3dm

Kiểm tra ngày 2026-10-07. Các liên kết GitHub và raw dưới đây đã truy cập được
trong phiên này. Bản ghi này đọc được ngay trong source, không phụ thuộc website
developer.rhino3d.com. Phạm vi: hướng BRep solid và dấu thể tích khi biến đổi.

## McNeel: ý nghĩa chính xác của +2

[Header chính thức, cố định commit của SDK đang dùng](https://github.com/mcneel/opennurbs/blob/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/opennurbs_brep.h)
([bản raw](https://raw.githubusercontent.com/mcneel/opennurbs/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/opennurbs_brep.h)),
mục `SolidOrientation` và `SetSolidOrientationForExperts`:

| Giá trị trả về | Ý nghĩa |
|---|---|
| +1 | Solid, pháp tuyến hướng ra ngoài |
| -1 | Solid, pháp tuyến hướng vào trong |
| +2 | Solid, SDK chưa tính được hướng |
| 0 | Không phải solid |

Header phân biệt SDK độc lập với override trong Rhino SDK. Chỉ đặt hướng bằng
API expert khi đã xác định chắc chắn. `SetSolidOrientationForExperts(0)` có nghĩa
không phải solid; không dùng nó để xóa cache hướng.

[Triển khai chính thức](https://github.com/mcneel/opennurbs/blob/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/opennurbs_brep.cpp)
([bản raw](https://raw.githubusercontent.com/mcneel/opennurbs/eb92af3ba1806b0a34a99aba0d3bda83e3d46083/opennurbs_brep.cpp)),
mục `ON_Brep::SolidOrientation`: cache đã có hướng trả về +/-1; solid chưa có
cache trả về +2. Giá trị nội bộ `m_is_solid==2` nghĩa hướng vào trong, khác với
giá trị +2 của hàm. Vì vậy không sửa trực tiếp trường cache và không coi +2 là +1.

Đây là giới hạn được tài liệu hóa của SDK độc lập. Chưa có bằng chứng từ các
nguồn này rằng thay phiên bản openNURBS sẽ cung cấp bộ tính hướng của Rhino.

Đã kiểm tra thêm tag người dùng hỏi:
[v8.35.26251.13001](https://github.com/mcneel/opennurbs/releases/tag/v8.35.26251.13001)
trỏ tới `eb92af3`, tức cùng commit được pin trong OM9. [Source tại tag](https://github.com/mcneel/opennurbs/blob/v8.35.26251.13001/opennurbs_brep.cpp)
vẫn có nhánh standalone trả +2. Không cần đổi SDK để có chính thay đổi này.

## rhino3dm: đường gọi cũng dùng SDK độc lập

Đã đọc source nhánh mặc định `9.x`, cố định commit
`7eee4d81abec6c29898eb21716e76f2de9a825c1`. Submodule openNURBS tại commit
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, cùng SDK đang dùng trong OM9.
Đây là kiểm tra source, chưa cài hoặc chạy binary rhino3dm.

- [.NET getter](https://github.com/mcneel/rhino3dm/blob/7eee4d81abec6c29898eb21716e76f2de9a825c1/src/dotnet/opennurbs/opennurbs_brep.cs#L3886)
  gọi `ON_Brep_GetInt` rồi chuyển kết quả thành enum; enum có
  [Unknown = 2](https://github.com/mcneel/rhino3dm/blob/7eee4d81abec6c29898eb21716e76f2de9a825c1/src/dotnet/opennurbs/opennurbs_brep.cs#L5483).
- [Native wrapper](https://github.com/mcneel/rhino3dm/blob/7eee4d81abec6c29898eb21716e76f2de9a825c1/src/librhino3dm_native/on_brep.cpp#L785)
  trả trực tiếp `pConstBrep->SolidOrientation()`, không có bộ tính hướng riêng
  trong đường gọi này.
- [Header binding Python/JavaScript](https://github.com/mcneel/rhino3dm/blob/7eee4d81abec6c29898eb21716e76f2de9a825c1/src/bindings/bnd_brep.h#L148)
  còn comment `SolidOrientation`. `IsSolid` chỉ gọi kiểm tra solid của SDK;
  `Flip` chỉ đảo hướng hiện có, không xác định dấu còn chưa biết.

Kết luận trong phạm vi source đã kiểm tra: đổi sang rhino3dm không tự giải quyết
trạng thái +2. OM9 vẫn cần xác định hướng bằng hình học OCCT trước phản chiếu,
giữ winding nguồn và từ chối khi chưa phân loại được. Bản raw cố định commit,
line numbers, SHA256 và submodule pin được lưu tại
`H:/FreeCAD-src/build/rhino3dm-orientation-reference/audit.json`.

## OpenCascade: xác định hướng mà không đảo solid

[OCCT 8: BRepClass3d_SolidClassifier](https://github.com/Open-Cascade-SAS/OCCT/blob/V8_0_0/src/ModelingAlgorithms/TKTopAlgo/BRepClass3d/BRepClass3d_SolidClassifier.hxx)
([raw](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V8_0_0/src/ModelingAlgorithms/TKTopAlgo/BRepClass3d/BRepClass3d_SolidClassifier.hxx)):
`PerformInfinitePoint(tolerance)` được dành cho việc tính hướng solid.

[OCCT 8: BRepLib::OrientClosedSolid](https://github.com/Open-Cascade-SAS/OCCT/blob/V8_0_0/src/ModelingAlgorithms/TKTopAlgo/BRepLib/BRepLib.cxx)
([raw](https://raw.githubusercontent.com/Open-Cascade-SAS/OCCT/V8_0_0/src/ModelingAlgorithms/TKTopAlgo/BRepLib/BRepLib.cxx)):
hàm phân loại điểm vô cực; kết quả IN khiến solid bị đảo, ON/UNKNOWN khiến hàm
thất bại. Vì vậy gọi hàm này cho mọi +2 sẽ chuẩn hóa solid hướng vào trong ra
ngoài và làm mất dấu nguồn.

OCCT có bộ tính hướng, khác với nhánh standalone chưa tính hướng của McNeel.
ON/UNKNOWN vẫn là kết quả được API dự liệu. Những nguồn đã kiểm tra không cung
cấp thống kê tần suất; kết quả bộ test OM9 không dùng để khẳng định mọi solid
ngoài thực tế sẽ phân loại được hoặc lỗi đó thường xuyên/hiếm xảy ra.

Suy luận triển khai OM9: giữ winding của face/trim khi dựng shell; phân loại
solid hợp lệ mà không gọi bước đảo ra ngoài cho +2. IN tương ứng -1, OUT tương
ứng +1. Không đoán khi kết quả chưa xác định. Tính hướng trước affine transform,
rồi nhân dấu với dấu định thức. Làm việc trên bản sao để giữ nguồn gốc/native
payload. Nhánh export Rhino5 bảo toàn native từ chối solid hướng vào trong vì
thử nghiệm Rhino5 trước đó cho thấy document insertion đảo chúng ra ngoài;
export Geometry only dùng outward member và reflected instance đã kiểm chứng.

Phạm vi kỹ thuật: hướng +2 cần BRep chuyển được sang một OCCT solid hợp lệ.
Native-valid BRep chưa chuyển được (ví dụ một BRep chứa nhiều shell rời nhau)
vẫn bị từ chối bảo toàn khi chưa xác định được hướng; không kết luận nguồn hỏng.
Thông báo kèm UUID đối tượng và nguyên nhân bước phân loại. Đây là giới hạn mới
của nhánh bảo toàn +2, cần xử lý thêm trước khi tuyên bố bảo toàn mọi native BRep.

## Kiểm chứng cần thực hiện

Native: +2 inward/outward độc lập, shear/scale không đều, một và hai phản chiếu,
export/reimport, nguồn bất biến và từ chối ghi atomic. GUI: trường hợp block
lồng/shared phải cho signed volumes `[-2880,-720,-576,24]` mm³. Chạy lại cả hai
fixture nhẫn và kiểm tra lifecycle mở/lưu/mở lại trong Rhino5. Kết quả thực tế
được ghi riêng, không coi tài liệu hay source code là bằng chứng test thành công.

Hai sai lệch bounds nghiêm ngặt và các lớp openNURBS chưa hỗ trợ vẫn là công
việc riêng. Hoàn tất hướng +2 không có nghĩa đã hỗ trợ toàn bộ openNURBS.
