# Rhino 5 ExtrudeCrv và luồng F6 Extrude Lines

OM9-TSPLINE-073 xuất hiện dưới macro _tsExtrudeLines trong nhóm T-Splines / 3Crv của Matrix. Book 2 trang 95–96 mô tả hai điểm cho Direction và một khoảng cách đùn. Hồ sơ này đối chiếu các bước có cùng tên trong Rhino 5. Chưa có bản chạy plug-in T-Splines để đo định dạng đối tượng kết quả, thứ tự các đường nối hoặc giá trị khi cài mới. [Gemvision Matrix Book 2, trang 95–96](https://s3.us-east-2.amazonaws.com/gvwebsite/files/support/matrix/matrix_8_manual_book2.pdf#page=95) · [McNeel Rhino 5 ExtrudeCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/extrudecrv.htm)

| Mục đã đối chiếu trong Rhino 5 x64 | Địa chỉ và dòng trong bản phân tích | Hành vi quan sát |
|---|---|---|
| Tên lệnh và thông số | 0x1400234a0, dòng 37913–37918. 0x1400234b0, dòng 37922–37932 | Tên ExtrudeCrv. Phần ghi cấu hình có Distance, BothSides, DeleteInput và SplitAtTangents. |
| Vòng hỏi khoảng cách | 0x140023570, dòng 38174–38231 và 38436–38545 | Hướng có nhánh CRhinoGetDirection riêng. Khi nhận hướng hợp lệ, nhánh quay về vòng nhập giữ khoảng cách trước đó. Điểm thông thường hoặc số nhập theo nhánh tính khoảng cách mới rồi dựng hình. |
| SetBasePoint | Dòng 38760–38784 | Điểm vừa chọn thay gốc đo, sau đó quay về cùng vòng nhập. Disassembly tại 0x1400241d2 xác nhận đích nhảy trở lại vòng ở 0x1400236f4. |
| Dấu của trục đùn mặc định | 0x14002fa10, dòng 46181–46252 | Nếu đã tìm được mặt phẳng, lấy pháp tuyến và chọn dấu dương theo thành phần tọa độ có độ lớn trội. Khi bằng nhau, ưu tiên Z rồi X. Nếu thiếu mặt phẳng, lấy hướng pháp tuyến CPlane đang dùng, cuối cùng có thể dùng Z thế giới. |
| ToBoundary | 0x140021c60, dòng 36888–37118. Nhánh gọi trong lệnh ở dòng 38545–38602 | Hỏi một mặt chặn, có tùy chọn Direction. Sao chép BRep của đối tượng và thu gọn các mặt nền trước khi tính. Việc chọn ToBoundary xóa cờ BothSides đã nhận diện qua nhánh tùy chọn. |
| SplitAtTangents | 0x140024a30, dòng 38812–38841. Hàm công bố 0x140031230, dòng 47281–47301 | Biến nhận tùy chọn tại offset 0x3e6 bật lời gọi SplitFacesAtTangents trên kết quả BRep. Lời gọi SplitKinkyFaces có cờ riêng. Nhánh trả đối tượng extrusion nhẹ đi đường khác. Cần phân biệt ba trường hợp khi đối chiếu số mặt của hai cung nối tiếp tuyến. |
| Gia công mặt chặn và nắp | 0x140025c50, dòng 39571–39804. Dòng 38609–38635 trong vòng công bố kết quả | Các nhánh đưa mặt đã đùn và hình học mặt chặn tới phép xử lý BRep. Khi yêu cầu nắp, gọi phép đóng lỗ phẳng cho từng BRep. Cần đo đầu ra ở mép trim, tiếp tuyến và mặt chặn hở trước khi chọn phép dựng của OM9. |

Đã kiểm trực tiếp địa chỉ hàm trả tên lệnh và đích nhảy của vòng nhập trong PE có image base 0x140000000. SHA-256 của Rhino.exe là 8daacaa0a9a7ade3c82793309fdff421306b736fa57bf1342b01369d0f14089b. SHA-256 của bản xuất Rhino.exe.c là 7a4a9d4fe3cdb94b9c42b71b3ed8468bd7fdbbe2d49da5f78063d72ef5b7711e. Mức tin cậy cao cho nhánh Direction của Rhino 5. Phép áp dụng vào T-Splines dựa trên thứ tự bước và hình minh họa trong manual Matrix.

Phép đo tiếp theo dùng một profile nằm dưới mặt phẳng nghiêng và dưới một mặt cong có biên trim. So đường đầu mút, giao tuyến, cách chọn phía, đối tượng được giữ lại và Undo. Với Solid, kiểm một đường kín phẳng, đường hở, profile có lỗ và hai đường kín rời nhau. Đối chiếu kết quả bật/tắt SplitAtTangents bằng hai cung nối tiếp tuyến. [Kiểm thử OM9](2026-11-22-f6-matrix9.md#extrude-lines)

Rhino 5 đi qua từng kết quả tại dòng 38609–38635 và gọi phép đóng lỗ phẳng cho từng BRep. OM9 áp dụng tùy chọn Solid theo từng đối tượng đường cong được chọn. Tệp OM9 đã lưu và mở lại cho khối chữ nhật 6 mặt và trụ 3 mặt. Kiểu đối tượng do bản T-Splines gốc tạo ra vẫn cần đo trực tiếp.

Trong OM9, cùng một mặt cong có thể cắt mặt đùn ở hai độ cao. Fixture xác nhận đường giao liên tục về phía đã chọn, giao tuyến bị cắt bởi lỗ trim và thể tích khi đóng Solid. Bấm mặt trên của hộp trong viewport cho Z=6 ngay cả khi mặt dưới cách profile chỉ 4 đơn vị. Cả đối tượng BRep của OM9 và hình học 3DM mở lại đều qua phép thử. [Bằng chứng của lệnh](2026-11-22-f6-matrix9.md#extrude-lines)

## SplitAtTangents trong bản chạy Rhino 5.14.522.8390

Đã chạy trực tiếp lệnh ExtrudeCrv trên Rhino 5 Windows x86 trong hai phiên riêng có scheme mới. Mỗi đường hở gồm hai cung tròn bán kính 1 và 2, nối ở (1, 1, 0). Góc giữa hai tiếp tuyến là 0° hoặc 90°. Chiều cao đùn bằng 3. Mỗi lần gọi tạo một đối tượng. Hồ sơ này bổ sung dữ liệu chạy cho phần phân tích PE x64 ở trên.

| Đầu vào và kiểu xuất | SplitAtTangents=No | SplitAtTangents=Yes |
|---|---|---|
| Hai cung tiếp tuyến, Polysurface | BRep 1 mặt, 4 cạnh | BRep 2 mặt, 7 cạnh |
| Hai cung gãy 90°, Polysurface | BRep 2 mặt, 7 cạnh | BRep 2 mặt, 7 cạnh |
| Hai cung tiếp tuyến hoặc gãy, Extrusion | Một Extrusion | Một Extrusion |
| Solid hình sân vận động, Polysurface | BRep 3 mặt, 3 cạnh | BRep 6 mặt, 12 cạnh |

Profile sân vận động gồm hai đoạn dài 2 và hai nửa đường tròn bán kính 1. Cả hai khối đều kín, thể tích bằng 21,424777957681755, tương ứng 3 × (4 + π). Ở scheme mới, lần chạy đầu tiên chưa nhập tùy chọn cho 3 mặt. Sau khi đặt Yes, lần chạy kế thừa giá trị vừa dùng. Với đối tượng Extrusion nhẹ, gọi ToBrep(false) sau lệnh cho 1 mặt, ToBrep(true) cho 2 mặt trong cả bốn trường hợp.

Phần chữ của manual Rhino 5 mô tả Yes và No cho hai cung theo thứ tự ngược với đầu ra đo trực tiếp. Với OM9, dùng số mặt và cạnh đã đo khi chọn cách dựng khối. Hai phiên chạy gồm 8 và 9 lần gọi ExtrudeCrv. Phép kiểm tra tên lệnh tsExtrudeLines trong bản cài dùng đo trả về false. Hình dây và số đường nối do plug-in Matrix tạo vẫn cần đối chiếu riêng. [McNeel ExtrudeCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/extrudecrv.htm) · [McNeel UseExtrusions](https://docs.mcneel.com/rhino/5/help/en-us/commands/useextrusions.htm)

Hai lần gọi bổ sung dùng chính tùy chọn ToBoundary của Rhino với mặt chặn hữu hạn ở Z=3. Tắt SplitAtTangents cho 3 mặt và 3 cạnh. Bật cho 6 mặt và 12 cạnh. Cả hai khối đều kín. Thể tích đo được lần lượt là 21,424777958961812 và 21,424777960241862. Tổng cộng đã chạy 19 trường hợp ExtrudeCrv, ngoài bốn quan sát riêng qua API ToBrep.
