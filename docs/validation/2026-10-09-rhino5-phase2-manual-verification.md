# Kiểm chứng Phase 2 từ Rhino 5

Trong Rhino 5 đang mở, chạy:

```text
_-RunPythonScript "H:\FreeCAD-src\build\om9-dev\tests\rhino5_verify_phase2.py"
```

Script mở một phiên Rhino kiểm thử riêng cùng các phiên OM9 có profile riêng. Phiên Rhino của người dùng tiếp tục mở, không đổi document, object, selection, đơn vị, tolerance hoặc trạng thái chưa lưu. Không cần đóng bản vẽ trước khi chạy. Các cửa sổ kiểm thử được mở ẩn; Command History hiển thị tiến độ. Lần kiểm chứng dưới đây mất khoảng 42 giây cho workflow con; máy khác có thể lâu hơn.

Script dùng runtime Rust `H:/FreeCAD-src/build/om9-phase2-rust-sdk`, xác minh source/runtime bằng `phase2-build.json`, không chọn bản build cũ. Python ở đây là bootstrap/test chạy qua API IronPython 2.7 bắt buộc của Rhino 5; logic sản phẩm vẫn thuộc Rust và cầu nối native đã nghiệm thu. PowerShell điều phối các gate hiện hữu; không thay đổi các gate hoặc snapshot nghiệm thu cũ.

Phạm vi: ba curve rational, periodic và có placement; Export Selected thật từ Rhino, sửa CV/weight/knot trong OM9, Undo/Redo, vẽ curve mới, Cancel, Rebuild, Join, kiểm tra từ chối Join periodic kín, mở lại FCStd khi thiếu file nguồn, export current geometry sang V5, Open/SaveAs thật trong Rhino và nhập lại chính các file SaveAs vào OM9. Tolerance hình học của gate là 0,001 mm; basis/weight dùng kiểm tra riêng. Snap/cache/mesh và hiệu năng đa luồng có các gate FreeCAD riêng; script này kiểm chứng vòng trao đổi curve của Phase 2, không chứng nhận toàn bộ openNURBS.

Mỗi lần chạy tạo `H:/FreeCAD-src/build/rhino5-phase2-user/<GUID>/phase2-verification.json`, kèm `rhino-launch.log`, `saved-reread.log`, `progress.json`. JSON chứa đường dẫn tới report Rhino, report nhập lại, hash runtime và số kiểm tra. File đầu vào của bước nhập lại lấy từ đúng lần Rhino vừa chạy, không tìm report mới nhất toàn máy. Khi lỗi, Command History báo FAIL và đường dẫn log; các lỗi được ghi lại, không thành PASS. Các gate có timeout; nếu timeout, xem PID trong log để nhận diện phiên test còn đang chạy. Không tự đóng phiên Rhino của người dùng.

## Kết quả thực tế ngày 2026-10-09

- Rhino **5.14.522.8390**, FreeCAD **27.1.0 / 49113**, trên Windows.
- Ba fixture đạt: **125 kiểm tra Rhino**, **35 kiểm tra host OM9**, **55 kiểm tra nhập lại SaveAs**.
- Harness ứng dụng gọi entrypoint từ một Rhino có curve được chọn, document chưa lưu, đơn vị cm và tolerance 0,002: **8/8 kiểm tra đạt**; ảnh chụp trạng thái trước/sau trùng nhau. Harness đóng phiên riêng của nó; entrypoint người dùng không gọi Exit.
- Chu trình test trước: RED ở `manual verification entry point exists`, rồi GREEN trên Rhino thật. Lần thử harness đầu tiên dùng API RuntimeSerialNumber không có ở Rhino 5 và đã được sửa sang GetHashCode trước RED chức năng; không phải lỗi sản phẩm.
- Tool suite: **58/58 đạt**, lệnh `rtk proxy python -m unittest discover -s tools/tests -v` tại `H:/FreeCAD-src/build/om9-dev`.
- Manifest SHA256: `4a42ffdc42d72eb93e23bb675a8b8b7e7f21a0ffcc410ae28742c0742e680564`.
- Native module SHA256: `e59021ed71a7aa48c16b49ca6602bc5a55e0a504341f6452f29b67b2baf7b5a7`.

Bằng chứng lần chạy mới:

- [Report entrypoint](H:/FreeCAD-src/build/rhino5-phase2-user/1cfcb3fcb8f94f6483a4dd0a428a6736/phase2-verification.json).
- [Report bảo toàn phiên gọi](H:/FreeCAD-src/build/rhino5-phase2-verify-session-test/c0bf73b36b8e4893bb59f7d9fdbfd43c/results.json).
- [Report Rhino](H:/FreeCAD-src/build/rhino5-phase2-rust/6ca62824b0a7426ea676d73c29870a32/rhino5-results.json).
- [Report nhập lại SaveAs](H:/FreeCAD-src/build/om9-dev/build/modeling_phase2_saved_reimport-1/652334b65e4e42f5a66a6990ebcb2655/results.json).

Đây là một batch regression bổ sung để chứng minh cách chạy thủ công và bảo toàn document đang mở; ba fixture trao đổi được chạy lại bằng gate cũ. **0 batch đã chuẩn bị còn chờ; 7 gói full openNURBS vẫn mở; tổng số batch tương lai chưa xác định.** Tiếp theo người dùng chạy lệnh trên Rhino của mình; kế hoạch sản phẩm tiếp theo vẫn là Phase 3 BRep modeling. Không đổi phạm vi nghiệm thu Phase 2 hoặc tích hợp vào checkout sản phẩm chính.

## Lần chạy của người dùng và sửa cảnh báo STA

Người dùng đã chạy entrypoint từ Rhino5 có Clayoo/V-Ray/Matrix và gửi kết quả PASS. [Report thực tế](H:/FreeCAD-src/build/rhino5-phase2-user/0c18337bd6d4465586fa5d138b8d4f59/phase2-verification.json) xác nhận 3/3 fixture,125 Rhino/35 OM9/55 nhập lại SaveAs, đúng manifest/native module phía trên. Vòng trao đổi mất khoảng37 giây. Plugin startup xuất hiện trong log phiên gọi; fixture vẫn chạy trong profile test riêng, không tuyên bố kiểm chứng nghiệp vụ của Clayoo/V-Ray.

IronPython cảnh báo `Thread.Sleep` trên luồng STA của script. Giữ nguyên `RhinoApp.Wait()` và khoảng chờ100ms, thay Sleep bằng `Thread.CurrentThread.Join(100)` theo [tài liệu Microsoft về Thread.Join và message pumping](https://learn.microsoft.com/en-us/dotnet/api/system.threading.thread.join). Không tắt hoặc lọc cảnh báo trong script người dùng. Không đổi logic hình học, runtime Rust hoặc gate nghiệm thu cũ.

Batch entrypoint hiện tại được mở rộng bằng một assertion kiểm cảnh báo STA, không cộng các lần chạy lại thành batch mới:

- [RED](H:/FreeCAD-src/build/rhino5-phase2-verify-session-test/341fe9513d4e4e168085ffcc12c42d4a/results.json): workflow hình học PASS nhưng assertion STA thất bại, bắt được đúng cảnh báo người dùng báo.
- [GREEN](H:/FreeCAD-src/build/rhino5-phase2-verify-session-test/b7dd2836462d4f42b69479e1e84974ff/results.json): **9/9 assertion đạt, warnings=[]**, bản vẽ trước/sau trùng nhau.
- [Report sau sửa](H:/FreeCAD-src/build/rhino5-phase2-user/8c147a737e0849b29c89ff3a18769872/phase2-verification.json): 3/3 fixture,125 Rhino/35 OM9/55 nhập lại SaveAs đạt. Tool suite58/58 và source/runtime manifest verification chạy lại đạt.

Lần chạy của người dùng đã được ghi nhận; không cần yêu cầu chạy lại chỉ vì cảnh báo cũ.0 batch đã chuẩn bị còn chờ;7 gói full openNURBS vẫn mở;tổng batch tương lai chưa xác định. Bước sản phẩm tiếp theo vẫn là kế hoạch Phase3 BRep modeling.
