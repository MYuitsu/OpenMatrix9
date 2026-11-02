# Prompt Solid đã được duyệt

Người dùng duyệt ngày 2026-10-23 cho standalone OM9. Đây là quy tắc OM9 đã được
duyệt, không phải chứng nhận hành vi Matrix gốc hay nghiệm thu M03-UI.

## Cylinder Center và 2Point — OM9-SOLID-016

- Trước khi chọn điểm đầu tiên, sau khi đổi sang 2Point, 3Point, FitPoints, AroundCurve hoặc Tangent, OM9 cung cấp Center để quay lại cách dựng tâm/bán kính; chọn bằng phím tắt, chuột hoặc nhập lệnh đều dùng cùng trạng thái và không tạo lịch sử. Đây là quy tắc giao diện OM9 đã kiểm thử mô phỏng, chưa xác minh prompt Matrix gốc. Xem `docs/validation/2026-10-29-solid-cylinder-center-return.md`.

- Center nhận tâm, bán kính và chiều cao; 2Point nhận hai đầu đường kính rồi
  chiều cao. Nhấp phải icon gọi 2Point theo binding đã phục hồi. OM9 chiếu đáy
  vào CPlane đã ghi nhận và đo chiều cao có dấu theo pháp tuyến.
- `Solid=Yes` tạo khối kín; `No` tạo mặt trụ bên mở. Đầu vào sai giữ bước hợp
  lệ, commit là một transaction. Nhập trực tiếp hai số trong phiên `Solid=No`
  bị từ chối cho đến khi chọn tâm, để không âm thầm tạo khối kín.
- Đã kiểm thử controller, tùy chọn, nhấp phải và viewport mô phỏng; solver
  Matrix gốc, desktop thật và các chế độ đáy khác chưa được xác minh.

## Tube cơ bản — OM9-SOLID-017

OM9 nhận tâm và bán kính, hoặc đáy 2Point/3Point/FitPoints/AroundCurve/Tangent + Point, rồi bán kính thứ hai và chiều cao có dấu. FitPoints dùng các điểm trực tiếp hoặc điểm được chọn; AroundCurve dùng tiếp tuyến của curve. Tangent + Point hiện nhận một curve phẳng, điểm tiếp xúc và một điểm ràng buộc; nguồn bị thay đổi ngăn commit và ẩn preview. Có thể nhập hai bán kính theo bất kỳ thứ tự nào; Diameter áp dụng cho cả hai, hai bán kính bằng nhau và chiều cao bằng không bị từ chối trước commit. Sau bán kính đầu, WallThickness số dương tạo vòng thứ hai phía trong, nhỏ hơn bán kính đầu; chọn điểm có thể đặt vòng thứ hai trong hoặc ngoài. Độ dày không bị chia đôi khi Diameter bật. 3Point có Radius sau hai điểm và chọn phía tâm. BothSides mặc định No; khi Yes, chiều cao áp dụng về cả hai phía của mặt phẳng đáy, preview và khối có tổng chiều dài gấp đôi. Preview dây hiện hai vòng mỗi đầu và bốn đường sinh cho mỗi thành, không thay đổi tài liệu. Chuột phải icon gọi 2Point; hình học, Undo, tùy chọn và viewport/menu mô phỏng đã kiểm tra. Chiều nhập WallThickness, thứ tự bán kính và solver chưa đối chiếu Matrix gốc; Tangent nhiều curve, GPU và desktop thật chưa nghiệm thu. Xem `docs/validation/2026-10-29-solid-tube-wall-thickness.md` và các báo cáo Tube trước đó.

Sau khi chọn tâm Center hoặc AroundCurve, Area nhận diện tích theo bình phương đơn vị tài liệu và xác định bán kính vòng đầu; Radius hoặc Diameter thoát chế độ này. Giá trị không hợp lệ giữ phiên. Đã kiểm thử cm², Undo và giao diện mô phỏng; chưa xác minh Matrix gốc. Xem `docs/validation/2026-10-29-solid-tube-area.md`.

## Pyramid Inscribed cơ bản — OM9-SOLID-018

OM9 chọn tâm, góc đa giác đều nội tiếp rồi đỉnh theo chiều cao có dấu trong CPlane đã chụp; `NumSides=` đặt số cạnh trước khi xác định bán kính. Mặc định OM9 là bốn cạnh, chưa xác minh mặc định Matrix. Preview dây và commit theo cùng hướng đáy; giới hạn tài nguyên, Undo và viewport mô phỏng đã kiểm tra. Desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-pyramid-inscribed.md`.

Circumscribed nhận tâm và trung điểm cạnh; bán kính nhập hoặc chọn là bán kính nội tiếp, đa giác được xoay để trung điểm cạnh nằm theo hướng đã chọn. Có thể đổi Inscribed/Circumscribed trước khi chốt bán kính, hoặc Undo về bước đó. Preview và khối đã kiểm tra với hai dấu chiều cao; Matrix gốc và desktop thật chưa xác minh. Xem `docs/validation/2026-10-29-solid-pyramid-circumscribed.md`.

Edge nhận hai đầu cạnh đáy; Flip đổi đa giác sang phía kia trước hoặc sau khi chốt cạnh. Undo sau điểm thứ hai trở về điểm đầu và khôi phục mặt phẳng nhập. Preview, hình học và các đường gọi mô phỏng đã kiểm tra; phía mặc định và hành vi Matrix gốc chưa xác minh. Xem `docs/validation/2026-10-29-solid-pyramid-edge.md`.

Star nhận tâm, hai bán kính luân phiên và chiều cao có dấu; Automatic hoặc Enter đặt hai bán kính bằng nhau. Chuột phải icon và macro `_Pyramid _Numsides=5 _S` mở Star năm cánh. Rust tạo và kiểm tra profile có giới hạn; adapter OCCT dựng BRep. Preview, Undo, thể tích và menu/viewport mô phỏng đã kiểm tra; solver Matrix và desktop thật chưa xác minh. Xem `docs/validation/2026-10-29-solid-pyramid-star.md`.

Solid mặc định Yes tạo khối kín; No bỏ mặt đáy để tạo shell mở, áp dụng cho Center, Circumscribed, Edge và Star. Tùy chọn có thể đổi trong phiên và không tự tạo lịch sử; hình học được kiểm tra topology trước commit. Nhập tắt ba số khi Solid=No yêu cầu chọn tâm trước. Đây là diễn giải OM9; trang trích ghép hai cột, chưa xác định chắc dòng Solid thuộc Pyramid, chưa xác minh Matrix gốc hoặc desktop thật. Xem `docs/validation/2026-10-29-solid-pyramid-open.md`.

AroundCurve chọn một curve rồi chọn tâm trên curve hoặc nhập OnCurve=0..1; pháp tuyến đáy theo tiếp tuyến đã biến đổi của curve. Có thể dùng với Inscribed, Circumscribed, Star và Solid=No; nguồn thay đổi ngăn preview và commit. Undo tâm trở lại bước chọn tâm, Center trở lại dựng thông thường; Edge kết hợp AroundCurve chưa hỗ trợ. Đã kiểm đường thẳng, đường tròn, ba đường chọn tùy chọn và viewport mô phỏng. Trang Pyramid trích ghép hai cột với Tube nên chưa chứng minh AroundCurve là tùy chọn Pyramid gốc; đây là diễn giải OM9, desktop thật cũng chưa xác minh. Xem `docs/validation/2026-10-29-solid-pyramid-around-curve.md`.

## Cone cơ bản — OM9-SOLID-019

OM9 nhận đáy Center, 2Point, 3Point/Radius, FitPoints, AroundCurve hoặc Tangent + Point một curve, rồi chiều cao có dấu; Area và Diameter xác định kích thước đáy. Solid=Yes tạo khối kín, No tạo mặt nón bên có một cạnh biên tại đáy. Chuột phải và `_Cone _P` gọi 2Point; preview gồm vòng đáy và bốn đường sinh. Đã kiểm thử hình học, Undo và giao diện mô phỏng; tiếp xúc nhiều curve, tùy chọn hướng, solver Matrix và desktop thật chưa xác minh. Xem `docs/validation/2026-10-29-solid-cone-basic.md`.

## Cylinder 3Point và Radius — OM9-SOLID-016

- 3Point dựng đáy qua ba điểm trong CPlane đã ghi nhận. Sau hai điểm có thể
  chọn Radius, nhập hoặc chọn bán kính rồi chọn phía tâm; tiếp theo nhập chiều
  cao có dấu. `Solid=Yes/No` vẫn chọn khối kín hoặc mặt bên mở.
- Điểm thẳng hàng và bán kính/phía không hợp lệ giữ các bước trước; Undo và
  commit dùng cùng session. Đã kiểm thử controller, tùy chọn và viewport mô
  phỏng; quy ước solver Matrix gốc và desktop thật chưa được xác minh.

## Cylinder AroundCurve — OM9-SOLID-016

- Chọn một curve và tâm trên curve hoặc nhập `OnCurve=0..1`, rồi xác định bán
  kính đáy và chiều cao có dấu. OM9 dựng đáy vuông góc tiếp tuyến và giữ nguồn.
- Dấu phiên bản nguồn được kiểm tra trước commit; nguồn cũ ẩn preview và bị từ
  chối. Đã kiểm thử nguồn bị xóa, controller và viewport mô phỏng; nguồn bị sửa,
  `Solid=No` kết hợp AroundCurve, solver Matrix và desktop thật chưa xác minh.

## Cylinder FitPoints — OM9-SOLID-016

- FitPoints nhận các điểm nhập trực tiếp hoặc tập điểm/control point/đỉnh mesh
  đã chọn mà bộ gom chung hỗ trợ; Enter fit đường tròn đáy rồi nhập chiều cao.
  Nguồn được giữ; fit suy biến hoặc vượt giới hạn không commit.
- OM9 dùng bộ fit đại số hiện có, không duy trì History liên kết. Đã kiểm thử
  điểm trực tiếp, point object đặt vị trí, giới hạn và viewport mô phỏng; chưa
  xác minh solver Matrix, desktop thật hay chọn control point trong lúc lệnh chạy.

## Cylinder Tangent + Point — OM9-SOLID-016

- Chọn một curve, chọn tiếp điểm hoặc `OnCurve=0..1`, chọn Point và một điểm
  ràng buộc khác, rồi nhập chiều cao. OM9 dùng bộ giải tiếp tuyến phẳng chung;
  giữ nguồn và từ chối commit nếu dấu phiên bản nguồn không còn hợp lệ.
- Đã kiểm thử hai phía, nguồn bị xóa và viewport mô phỏng. Nghiệm tiếp xúc
  nhiều curve, các lựa chọn Tangent khác, solver Matrix và desktop thật chưa
  được xác minh.

## Torus Center và 2Point — OM9-SOLID-015

- Center nhận tâm, bán kính vòng và bán kính ống; 2Point nhận hai đầu đường
  kính vòng rồi bán kính ống. Nhấp phải icon gọi 2Point như binding đã phục hồi.
- OM9 đo vòng trong CPlane được ghi nhận và đo tiết diện ống trong mặt phẳng
  xuyên tâm/pháp tuyến tại điểm vòng đã chọn; có thể nhập số dương. Bán kính
  ống phải nhỏ hơn bán kính vòng theo adapter hiện tại. Điểm sai giữ bước hợp
  lệ; Undo lùi bước và commit một transaction.
- Đã kiểm thử controller, nhấp phải và viewport mô phỏng. Phép chiếu tiết diện,
  solver Matrix gốc, desktop thực và các chế độ Torus khác chưa được xác minh.

## Torus 3Point — OM9-SOLID-015

- Chọn ba điểm trên vòng đáy rồi chọn hoặc nhập bán kính ống. OM9 chiếu các
  điểm vào CPlane ghi nhận tại điểm đầu và dựng đường tròn ngoại tiếp; điểm
  trùng hoặc thẳng hàng bị từ chối và giữ các điểm trước để thử lại.
- Undo khi nhập lùi một điểm, commit tạo một transaction. Đã kiểm thử controller,
  tùy chọn và viewport mô phỏng; chưa xác minh solver Matrix gốc hoặc desktop thật.
- Sau hai điểm có thể chọn `Radius`, chọn hoặc nhập bán kính ít nhất bằng nửa
  dây cung, rồi chọn phía tâm. Undo trở lại bước chọn phía rồi bán kính. OM9
  dùng điểm thứ ba tổng hợp trên vòng; quy ước phía chưa xác minh ở Matrix gốc.

## Torus AroundCurve — OM9-SOLID-015

- Chọn một curve, chọn tâm hoặc nhập `OnCurve=0..1`, rồi xác định bán kính vòng
  và bán kính ống. OM9 dựng mặt phẳng vòng vuông góc tiếp tuyến và giữ nguồn.
- Dấu phiên bản nguồn được kiểm tra trước các bước tiếp theo; nguồn thay đổi
  hoặc bị xóa làm lệnh từ chối. Chọn điểm gần nhất hiện yêu cầu placement đồng
  dạng. Đã kiểm thử controller, tùy chọn và viewport mô phỏng; chưa xác minh
  solver Matrix gốc, desktop thực và các curve tổng quát.

## Torus preview — OM9-SOLID-015

- Khi xác định bán kính ống, Center, 2Point, 3Point, Radius và AroundCurve có
  preview dây gồm hai vòng và bốn tiết diện, tính cùng tâm và bán kính với
  commit. Bán kính không hợp lệ hoặc nguồn AroundCurve cũ ẩn preview.
- Preview không tạo transaction. Kiểm thử controller và viewport mô phỏng đã
  qua; chưa xác minh hiển thị GPU/desktop hay preview của Matrix gốc.

## Torus Vertical — OM9-SOLID-015

- Vertical hiện dùng luồng tâm/bán kính: hướng bán kính được chiếu vào CPlane,
  mặt phẳng vòng đi qua hướng đó và pháp tuyến CPlane. Undo sau khi chọn vòng
  khôi phục CPlane đã ghi nhận để chọn hướng lại.
- Có preview và commit cùng phép tính; đã kiểm thử controller, tùy chọn và
  viewport mô phỏng. Chưa xác minh Matrix gốc, desktop thật hay kết hợp Vertical
  với 2Point, 3Point và AroundCurve.

## Ellipsoid from Center — OM9-SOLID-013

- OM9 nhận tâm rồi ba đầu trục; trục đầu nằm trong CPlane được ghi nhận khi
  chọn tâm, trục thứ hai vuông góc trong mặt phẳng đó, trục thứ ba theo pháp
  tuyến. Có thể nhập số dương tại từng bước trục hoặc tọa độ thế giới x,y,z.
- Trục không hợp lệ giữ các điểm trước để thử lại; Undo bỏ điểm cuối. Commit
  tạo một BRep và một transaction. Ba bán trục nhập trực tiếp vẫn được hỗ trợ.
- Đây là cách diễn giải OM9 đã kiểm thử controller và mô phỏng viewport; chưa
  xác minh cách chiếu điểm của Matrix gốc hay viewport desktop.

## Ellipsoid AroundCurve — OM9-SOLID-013

- Chọn một curve, chọn tâm trên curve hoặc nhập `OnCurve=0..1`, sau đó chọn
  hoặc nhập ba bán trục. OM9 dựng mặt phẳng đáy vuông góc tiếp tuyến; trục
  trong mặt phẳng lấy từ CPlane được chiếu vào mặt phẳng đó.
- Nguồn được giữ và kiểm tra dấu phiên bản trước các bước sau; nguồn bị xóa
  hoặc thay đổi làm lệnh từ chối thay vì commit hình từ dữ liệu cũ. Chọn điểm
  gần nhất hiện chỉ hỗ trợ placement đồng dạng. Đây là diễn giải OM9; solver
  Matrix gốc, desktop thật và các curve tổng quát chưa được xác minh.

## Ellipsoid preview — OM9-SOLID-013

- Ở bước trục thứ ba, Center, Diameter, Corner, FromFoci và AroundCurve hiển
  thị ba ellipse dây dùng cùng tâm, khung và bán trục với commit. Chiều cao
  bằng không hoặc nguồn AroundCurve không còn hợp lệ không tạo preview.
- Preview không tạo transaction. Đã kiểm thử controller và viewport mô phỏng;
  chưa xác minh màn hình GPU/desktop, các bước trước hay preview Matrix gốc.

## Ellipsoid Corner — OM9-SOLID-013

- Chọn hai góc đối diện của hình chữ nhật bao ellipse đáy rồi chọn hoặc nhập
  bán trục thứ ba. OM9 chiếu góc thứ hai vào CPlane được ghi nhận tại góc đầu;
  tâm là trung điểm, hai bán trục đáy là nửa kích thước theo hai trục CPlane.
- Hình chữ nhật suy biến hoặc chiều cao bằng không bị từ chối và giữ điểm hợp
  lệ; Undo lùi bước. Đây là cách diễn giải OM9 đã kiểm thử controller, lựa chọn
  tùy chọn và viewport mô phỏng, chưa xác minh solver Matrix hay desktop thật.

## Ellipsoid FromFoci và MarkFoci — OM9-SOLID-013

- Chọn hai tiêu điểm, một điểm trên ellipse đáy và trục thứ ba. OM9 chiếu tiêu
  điểm thứ hai và điểm trên ellipse vào CPlane đã ghi nhận; bán trục lớn bằng
  nửa tổng khoảng cách đến hai tiêu điểm. Tiêu điểm trùng hoặc ellipse suy biến
  bị từ chối, giữ các điểm hợp lệ để thử lại.
- `MarkFoci=No` là mặc định OM9; `Yes` tạo hai điểm cùng BRep trong một
  transaction, Undo/Redo áp dụng cho cả ba. Thiếu khả năng xác minh mặc định,
  phép chiếu và solver của Matrix gốc; UI desktop chưa được nghiệm thu.

## Ellipsoid Diameter — OM9-SOLID-013

- Nhấp phải icon Ellipsoid hoặc chọn Diameter / `_Ellipsoid _D` bắt đầu chọn
  hai đầu trục thứ nhất; tâm là trung điểm. Tiếp theo chọn trục thứ hai và thứ
  ba. OM9 chiếu đầu thứ hai vào CPlane đã ghi nhận, đo hai bán trục còn lại
  theo các hướng vuông góc. Nhập số ở bước đầu kia là chiều dài đường kính.
- Điểm sai giữ các bước hợp lệ; Undo bỏ điểm cuối, commit là một transaction.
  Đã kiểm thử controller, nhấp phải và viewport mô phỏng; chưa chứng minh cách
  chiếu điểm của Matrix gốc hay hành vi trên desktop thực tế.

## Sphere 3Point và 4Point — OM9-SOLID-014

- Với 3Point, điểm thứ ba thẳng hàng bị từ chối và hai điểm trước được giữ để
  chọn lại. Với 4Point, điểm thứ ba thẳng hàng hoặc điểm thứ tư đồng phẳng với
  đường tròn bị từ chối, giữ các điểm hợp lệ và không tạo transaction.
- Ở bước điểm thứ tư, OM9 hiển thị ba vòng dây xem trước theo đúng tâm và bán
  kính sẽ dùng khi commit; vị trí đồng phẳng không có preview. Đây là hành vi
  OM9 đã kiểm thử controller, chưa xác minh Matrix gốc hoặc viewport desktop.

## Text — OM9-SOLID-008

- Solid > Text hỏi nội dung, rồi Height. Giữ khoảng trắng và chữ số trong nội dung;
  không diễn giải chúng thành tham số kích thước ở bước nhập nội dung.
- Height sai cho nhập lại tại bước Height. Esc hủy mà không sửa tài liệu.

## Boolean Two Objects — OM9-SOLID-004

- Solid > Boolean Two Objects hỏi Operation, nhận `U`/`Union`, `D`/`Difference`,
  `I`/`Intersection`, không phân biệt hoa thường.
- Lựa chọn sai giữ prompt để nhập lại.
- Trong slice BRep OM9, có thể chọn tối đa hai BRep trong viewport khi lệnh
  đang chạy; Undo bỏ lựa chọn cuối và Cancel không tạo transaction. Người dùng
  đồng ý lưu ngày 2026-10-28; điều này chưa chứng minh preview hay năm trạng
  thái kết quả đã hoàn tất.
- Quy tắc preview BRep người dùng đồng ý lưu ngày 2026-10-28: hai đầu vào tạo
  preview của Union, A−B, B−A, Intersection hoặc XOR. Next hoặc nhấp viewport
  chuyển kết quả; Enter xác nhận. `DeleteInput=No` mặc định giữ cả hai đầu
  vào; `Yes` xóa cả hai. Preview rỗng không được xác nhận; Undo/Cancel không
  commit. Đây là quy tắc OM9, không chứng minh mặc định Matrix gốc hay nghiệm
  thu M03-UI.
- Quy tắc hiển thị được người dùng duyệt lưu ngày 2026-10-28: ẩn tạm hai đầu
  vào trong viewport khi có preview, kể cả preview rỗng; khi kết thúc hoặc
  hủy, hiển thị theo trạng thái tài liệu. Không sửa thuộc tính ẩn hoặc tạo
  transaction chỉ để ẩn tạm. Kiểm GPU thực và nghiệm thu desktop riêng.

## Boolean Difference — OM9-SOLID-001

Người dùng đồng ý lưu quy tắc này ngày 2026-10-28. Đây là quy tắc OM9,
không phải bằng chứng mặc định Matrix gốc hoặc nghiệm thu M03-UI.

- Chọn nhóm target rồi nhóm cutter; hợp từng nhóm trước khi trừ.
- `DeleteInput` mặc định `No` để giữ cutter; chọn `Yes` để xóa cutter.
- Undo/Cancel khi đang chọn không tạo transaction; thao tác thành công commit
  một transaction.

## Boolean Union — OM9-SOLID-003

Người dùng đồng ý lưu quy tắc này ngày 2026-10-28 cho slice BRep của OM9;
không coi đây là chứng nhận mặc định Matrix gốc hoặc nghiệm thu M03-UI.

- Chọn ít nhất hai BRep rồi Enter để hợp.
- `DeleteInput` mặc định `No` giữ đầu vào; chọn `Yes` để xóa đầu vào.
- Undo/Cancel khi đang chọn không tạo transaction.

## Boolean Intersection — OM9-SOLID-002

Người dùng đồng ý lưu quy tắc này ngày 2026-10-28 cho slice BRep của OM9;
không coi đây là chứng nhận mặc định Matrix gốc hoặc nghiệm thu M03-UI.

- Chọn hai nhóm BRep theo thứ tự, hợp từng nhóm rồi lấy phần giao.
- `DeleteInput` mặc định `No` giữ cả hai nhóm; chọn `Yes` để xóa cả hai.
- Giao rỗng bị từ chối; Undo/Cancel khi đang chọn không tạo transaction.

## Boolean Split — SolidBooleanSplit

Người dùng đồng ý lưu ngày 2026-10-28 cho slice BRep OM9; không coi đây là
bằng chứng mặc định Matrix gốc hoặc nghiệm thu M03-UI.

- Chọn tập target rồi tập cutter; cùng BRep có thể thuộc cả hai tập.
- Tạo các phần riêng không trùng; `DeleteInput=No` mặc định giữ cả hai tập,
  `Yes` xóa từng đầu vào một lần.
- Undo/Cancel khi đang chọn không tạo transaction; thành công commit một lần.

Menu và CMD dùng chung dispatcher và một transaction cho mỗi thao tác thành công.
Việc duyệt quy tắc prompt không đóng gate M03-UI; người dùng xác nhận chưa duyệt UI.

## Extract Surface nhiều mặt

Quy tắc OM9 được người dùng duyệt lưu ngày 2026-10-28; không tự coi là bằng
chứng mặc định Matrix gốc hoặc nghiệm thu M03-UI.

- Chọn nhiều mặt BRep trên một hoặc nhiều nguồn rồi Enter.
- `Copy=No` tách các mặt, `Copy=Yes` giữ nguồn; chọn toàn bộ mặt với
  `Copy=No` thay nguồn bằng các mặt rời.
- Undo khi chọn bỏ mặt cuối; commit và Undo sau commit áp dụng nguyên tử.

## Pipe Cap=Round trên đường tròn đầy đủ — OM9-SOLID-011

Icon Solid Multi Pipe (`SolidGVDMultiPipe`) và macro Matrix `_Pipe _Multiple` mở trực tiếp cùng phiên Pipe Multiple: chọn nhiều curve, Enter để tiếp tục nhập bán kính, commit một transaction. Menu, CMD và hình học hai đường thẳng đã kiểm mô phỏng; hỗ trợ hình học của Multiple vẫn theo các giới hạn Pipe được ghi bên dưới, desktop thật chưa nghiệm thu. Xem `docs/validation/2026-10-29-solid-multi-pipe-menu.md`.

Khi chọn điểm bán kính đầu/cuối trên một curve đơn, Pipe hiển thị vòng preview
trong tiết diện vuông góc tiếp tuyến; preview không commit và tôn trọng ngân
sách điểm. Đã đối chiếu đường thẳng, đường tròn đặt lệch và Radius/Diameter.
Vòng preview cũng dùng đường được chọn đầu tiên trong Multiple và hiển thị tại
trạm trung gian đang nhập. Đã kiểm Multiple trên đường thẳng và trạm trên cung;
vòng tiết diện. Với một đường thẳng, Pipe có thêm preview
thân dây gồm các vòng và đường nối, theo nội suy Local/Global, kể cả khi chờ
xác nhận; Thick hiển thị cả hai thành và Cap=Round có các cung bán cầu ở hai đầu.
Đã kiểm thứ tự bán kính trong/ngoài, bán kính tại
1/4, cực nắp Round, Cancel và viewport mô phỏng; đây không phải preview BRep.
Chain dùng tiếp tuyến
đoạn đầu/cuối; trạm trên chuỗi thẳng dùng trục chuỗi. Đã kiểm chuỗi gấp khúc,
curve–cạnh BRep và trạm thẳng trong mô phỏng. Đầu mút cạnh BRep được lưu sau
khi chọn; mỗi lần dùng vẫn kiểm source stamp, epoch và selection, không gọi
trích cạnh OCCT lặp lại. Đã kiểm preview lặp và xóa nguồn; hiệu năng/bộ nhớ
đỉnh, sửa placement/geometry, curve tổng quát, desktop thực và Matrix gốc còn mở.

Pipe Local mỏng với hai bán kính khác nhau trên một curve hở và Cap=Flat/None
dùng hàm cubic smoothstep theo chiều dài polyline, có đạo hàm bằng không ở hai
đầu về mặt toán học; loft qua 64 mẫu là xấp xỉ, không bảo đảm đạo hàm BRep bằng
không. Đã kiểm đường thẳng, cung, Cap=None, tùy chọn Global/Local, viewport mô
phỏng và Undo. Đây là diễn giải OM9 chưa xác minh solver Matrix; nhiều đường,
curve kín và desktop còn mở. Local Round với hai bán kính khác nhau được kiểm
trên đường thẳng và một cung hở cho cả mỏng và Thick: ghép bán cầu OCCT riêng
theo hướng đoạn đầu/cuối của đường lấy mẫu. Pipe mỏng Local/Global với Round
nhận trạm trung gian trên một đường hở; đã kiểm một trạm trên đường thẳng/cung
và viewport mô phỏng. Global Round không có trạm cũng đã kiểm thành đơn và
Thick với hai bán kính khác nhau trên cung hở. NURBS tổng quát, tự giao, trạm Thick hoặc nhiều đường,
native peak memory và Matrix gốc còn mở. Local mỏng nhận trạm trung gian trên một
đường hở: mỗi đoạn giữa các trạm dùng smoothstep theo chiều dài mẫu. Đã kiểm
một trạm trên đường thẳng và cung với Cap=Flat/None, viewport mô phỏng và Undo;
hai trạm chọn ngược thứ tự trên một NURBS bậc hai tốc độ không đều đã kiểm
thể tích tham chiếu và Undo. Curve tổng quát và solver Matrix chưa xác minh.
Local Thick nhận hai cặp
bán kính biến thiên trên một curve hở: dựng hai loft smoothstep rồi trừ cho
Cap=Flat hoặc tạo hai thành hở cho Cap=None. Đã kiểm đường thẳng, cung, thể tích,
thành hở, từ chối hai thành trùng, viewport mô phỏng và Undo; đường cong tổng
quát, hiệu năng/đỉnh bộ nhớ native và solver Matrix gốc còn mở.

Pipe Multiple nhận điểm bán kính đầu/cuối theo đường được chọn đầu tiên và
áp dụng giá trị chung cho toàn bộ đường; các đầu ra commit trong một transaction.
Đã kiểm hai đường thẳng, đường tròn chọn đầu cùng đường thẳng, Radius/Diameter,
viewport mô phỏng và Undo. Multiple Global mỏng nhận trạm trung gian trên tập
đường thẳng: vị trí tương đối đo theo đường đầu tiên được áp dụng cho từng
đường; một curve không hỗ trợ khiến cả tập bị từ chối không commit. Trạm được
chuyển bằng cùng phân số tham số chuẩn hóa, không phải tỉ lệ chiều dài cung;
đã kiểm cặp cung hở và đường thẳng theo cả hai thứ tự, đường kín bị từ chối,
viewport mô phỏng và Undo. Multiple Global mỏng với trạm và Cap=Round đã kiểm
trên cặp cung–đường thẳng theo hai thứ tự, với viewport và từ chối nguyên tử
khi gặp đường kín. Đây là diễn giải OM9; curve tổng quát, Thick, Matrix gốc và
desktop thực còn mở.
Khi Multiple dựng Round trên curve, kiểm tra ngân sách tạm phải tính cả các
kết quả đã dựng đang được giữ; giới hạn số output được kiểm là từ chối nguyên
tử. Fixture này chưa đo bộ nhớ đỉnh trong OCCT hoặc chứng minh mọi giới hạn.
Đã kiểm thêm hai trạm trên NURBS có tốc độ tham số không đều ghép đường
thẳng, cùng Cap=None trên cặp cung–đường thẳng; không suy rộng sang mọi curve.

Người dùng đồng ý lưu ngày 2026-10-28: với đường tròn đầy đủ và bán kính
không đổi, đường dẫn khép kín không có đầu để gắn nắp; `Cap=Round` tạo cùng
khối xuyến như `Cap=Flat`, kể cả `Thick`. Đây là cách diễn giải OM9 đã kiểm
thử trong phạm vi này, chưa xác minh hành vi Matrix gốc hay nghiệm thu desktop.

Pipe `ShapeBlending=Global` nhận hai bán kính khác nhau trên curve hở đơn với
Cap=Flat hoặc None, dùng loft tiết diện theo chiều dài polyline lấy mẫu. Đã
kiểm cung tròn hở, chọn điểm, CMD/menu, viewport mô phỏng, thể tích trong sai
số 3% và Undo; đường kín có hai bán kính khác nhau bị từ chối và có thể sửa
đầu vào. Đây là cách dựng xấp xỉ OM9; NURBS tổng quát, Cap=None trên curve,
Thick trên curve khác, Round, Matrix gốc và desktop thực chưa được nghiệm thu.

Pipe `Thick` với Global trên curve hở đơn dựng hai loft bán kính trong/ngoài
và trừ hoặc để hai thành hở theo Cap=Flat/None. Đã đo thể tích cung tròn biến
thiên và kiểm thành hở, Undo và viewport mô phỏng. Đây là hình học xấp xỉ từ
đường lấy mẫu; hình học tổng quát, Round, đường kín, Matrix gốc và desktop
thực còn mở. Với một NURBS bậc hai không hữu tỉ, đã kiểm thêm cả ống thường
và Thick, Cap=Flat/None, thể tích kín trong sai số 3%, topology hở và Undo.

Ở mode `Diameter`, giá trị số được hiểu là đường kính; điểm chọn trên tiết
diện vẫn xác định vị trí thành ống và lấy khoảng cách vuông góc đến trục làm
bán kính, giống mode `Radius`. Đã kiểm đường thẳng, Thick, tiết diện trung gian,
placement, CMD và viewport mô phỏng. Đây là diễn giải OM9; đường cong tổng quát,
Matrix gốc và desktop chưa được xác minh.

Điểm chọn bán kính ở đầu một curve đơn dùng mặt phẳng vuông góc với tiếp tuyến
đầu hoặc cuối đã biến đổi theo placement; điểm nằm trên tiếp tuyến bị từ chối.
Đã đo thể tích cho đường tròn đầy đủ ở hai mặt phẳng, Radius/Diameter và kiểm
viewport mô phỏng. Cung tròn một phần và một NURBS bậc hai không hữu tỉ đã
được kiểm với điểm lệch dọc hai tiếp tuyến và thể tích trong sai số 3%; curve
được lấy mẫu khi dựng nên chưa chứng minh hình học chính xác. NURBS tổng quát,
tiếp tuyến tại chỗ gãy, Matrix gốc và desktop chưa được nghiệm thu.

Pipe Global mỏng Cap=Flat/None nhận vị trí và bán kính tiết diện trung gian
trên một curve hở; vị trí chọn gần curve yêu cầu placement giữ tỉ lệ. Rust đưa
trạm vào các điểm mẫu và nội suy theo chiều dài polyline giữa các trạm; trạm
trùng hoặc đầu mút bị từ chối. Đã kiểm một cung và một NURBS bậc hai, Undo và
viewport mô phỏng; hai trạm chọn ngược thứ tự trên cung, Cap=Flat/None và Undo
đã kiểm. Curve kín và placement không giữ tỉ lệ bị từ chối không commit. Nhiều
trạm trên curve tổng quát, Thick, Round, độ chính xác tổng quát, Matrix gốc và
desktop còn mở.
Một NURBS bậc hai có tốc độ tham số không đều với hai trạm đã được kiểm bằng
tích phân thể tích độc lập trong sai số 0,3%; fixture phân biệt nội suy theo
chiều dài với nội suy theo tham số. Đây vẫn là một fixture xấp xỉ riêng lẻ.

## Đùn biên kín và nền tảng Boss

- Đùn không nắp một đa giác kín phải giữ đủ các thành, kể cả thành nối điểm
  cuối về điểm đầu; đùn dây hở chỉ tạo thành cho các đoạn đã có. Đã kiểm tra
  BRep, diện tích và số cạnh hở trên fixture hình chữ nhật.
- Kernel Boss đã kiểm tra một profile đa giác song song với một mặt BRep phẳng
  hữu hạn, kể cả mặt có lỗ: cắt mặt, nối thành đùn, giữ lỗ cũ và từ chối profile
  chồng lên lỗ. Lệnh OM9 nhận polygon kín, đường tròn đầy đủ, NURBS phẳng kín
  và polycurve kín gồm cung tròn/đoạn thẳng hoặc NURBS liên tục/đoạn thẳng;
  ellipse hữu tỉ, nửa tròn và cung Bézier bậc hai ghép đoạn thẳng đã kiểm thử,
  Enter, rồi chọn BRep phẳng hoặc shell các mặt đồng phẳng; polygon, tròn và
  ellipse NURBS đã kiểm với mặt phẳng nghiêng. Giữ curve, thay hình học mặt biên
  và commit một transaction. Undo khi chọn bỏ profile cuối; Cancel không commit.
  Chỉ hỗ trợ phép đặt mặt giữ góc và tỉ lệ, phép đặt đường tròn giữ hình tròn;
  Hai mặt đồng phẳng đã kiểm với profile nằm trong một mặt và cắt qua đường
  nối; mặt gập hoặc có mặt rời bị từ chối trong fixture. Polygon với mặt nghiêng
  được cắt thành theo mặt trước khi nối; mặt cắt qua profile hoặc phủ không
  đủ bị từ chối. Với NURBS trọng số dương, giao khoảng chiếu hộp bao BRep và
  hình bao điểm điều khiển giúp giảm từ chối sai sát mặt nghiêng; cách này
  cũng áp dụng cho polycurve gồm NURBS và đoạn thẳng, fixture ellipse bốn đoạn
  đã kiểm thử. Khoảng vẫn bảo thủ và có thể từ chối profile hợp lệ. Mặt cong, polycurve
  tổng quát, profile chồng nhau và hành vi gốc còn mở.
  Trường hợp OM9 đã kiểm thêm: polygon, đường tròn hoặc ellipse NURBS nằm trong
  mặt trụ đầy đủ, pháp tuyến vuông góc trục trụ, đùn theo chiều dương của pháp
  tuyến từng profile, cắt và nối với mặt trụ trong một transaction. Profile chồng nhau hoặc
  nối không đa tạp bị từ chối. Đảo thứ tự polygon đổi hướng;
  cách chọn hướng này là diễn giải OM9, chưa xác minh Matrix gốc.
  Polygon, đường tròn và ellipse NURBS có pháp tuyến xiên trục đã kiểm thử ở
  hai chiều, với diện tích đối chiếu độc lập; từ chối khi thành đùn không nối
  đủ vào mặt trụ hữu hạn. Đường cong xiên tổng quát và hướng gần song song trục
  chưa được nghiệm thu.
  Mặt trụ đã cắt và trụ đã biến đổi thành spline chưa nghiệm thu. NURBS kẹp đầu, miền đầy
  đủ, không tuần hoàn được chèn nút chính xác khi cần rồi tách thành đoạn Bézier
  trước khi đùn; ellipse, fixture hữu tỉ trơn và polycurve NURBS–đoạn thẳng
  có nút đơn đã kiểm trên trụ, kể cả đảo chiều đoạn và toàn polycurve.
  Giữ nguồn nhưng tạo thêm ranh giới mặt trong BRep kết quả; miền cắt, nút gián
  đoạn, tuần hoàn và sai số tổng quát chưa được nghiệm thu.
  Đo diện tích mặt đùn dùng bản chia tạm theo liên tục và miền tích phân;
  nếu chia bốn phần tạo BRep không hợp lệ thì thử chia hai phần và vẫn kiểm
  tra tính hợp lệ, không âm thầm bỏ kiểm tra khi cả hai thất bại.
  ba ellipse và fixture Bézier ghép đoạn thẳng đã khớp oracle độc lập trong
  0,0000001 mm², không chứng minh độ chính xác cho mọi bề mặt. Hiển thị NURBS
  dùng 256 bước mỗi khoảng nút, giữ nút kể cả khoảng rất ngắn và tôn trọng
  chiều đảo của từng đoạn; từ chối nút gián đoạn hoàn toàn. Sai số hiển thị
  tổng quát, NURBS tuần hoàn và polycurve lồng chưa được nghiệm thu.

## Truncated Cone — OM9-SOLID-020 (phần đã kiểm)

Chọn tâm đáy, bán kính đáy, chiều cao có dấu rồi bán kính đỉnh; có thể nhập
tọa độ CMD và giá trị hoặc chọn điểm; chiều cao đã kiểm bằng chuột mô phỏng
trên viewport Side View.
Khung CPlane được lưu tại điểm tâm;
chiều cao âm đi theo chiều âm pháp tuyến. Diameter đổi giá trị số bán kính
thành đường kính; chọn điểm vẫn dùng khoảng cách. Input sai giữ bước hiện tại.
Area tại bước kích thước đáy nhận diện tích theo đơn vị vuông của tài liệu và
đổi sang bán kính; chọn Radius hoặc Diameter thoát mode Area. Đỉnh tiếp tục
nhận Radius/Diameter, không nhận Area. Centimet vuông, chuyển mode, input sai
và ba đường kích hoạt tùy chọn đã được kiểm thử.
Solid mặc định Yes theo hành vi OM9 hiện có tạo BRep kín; Solid=No tạo một
mặt bên hở hai mép, bỏ cả hai nắp. Bán kính đáy và đỉnh bằng nhau dùng hình
trụ thay vì cone OCCT. Diện tích, topology, Undo, hai chiều cao và các đường
kích hoạt tùy chọn đã được kiểm; cách hiểu Solid=No và mặc định gốc của Matrix
chưa được xác minh. Kích thước rất nhỏ, bán kính gần bằng nhau và bộ nhớ native
chưa được nghiệm thu.
Undo bỏ input cuối, Cancel không tạo transaction và commit có thể Undo.
2Point chọn hai đầu đường kính trong CPlane đã lưu; nhấp phải icon gọi mode này.
3Point chọn ba điểm để dựng đường tròn trong CPlane đã lưu, từ chối điểm trùng
hoặc thẳng hàng; Undo cho phép chọn lại điểm. Sau hai điểm có thể chọn Radius,
nhập bán kính hoặc chọn điểm cách điểm đầu một bán kính rồi chọn phía của dây
cung để xác định tâm; từ chối bán kính nhỏ hơn nửa dây và hướng nằm trên dây.
Hai phía, preview, Undo và CPlane Front đã được kiểm; phép đo điểm và quy tắc
chọn phía là diễn giải OM9, chưa xác minh solver Matrix gốc.
FitPoints nhận điểm nhập/chọn trực tiếp hoặc tập đối tượng chọn trước: Point,
đỉnh Mesh, các pole NURBS curve/surface và đầu Line; có áp dụng placement.
Enter khớp đường tròn bằng bộ khớp đại số dùng chung với Circle, sau đó nhập
chiều cao và bán kính đỉnh; Undo/Cancel không commit phần chọn. Chọn cả đối
tượng lấy toàn bộ điểm tương ứng; chọn riêng control point, độ chính xác trên
dữ liệu nhiễu/không đồng phẳng, hướng pháp tuyến và solver Matrix gốc còn mở.
AroundCurve chọn một curve, chọn tâm gần curve hoặc nhập OnCurve=0..1; đáy
vuông góc tiếp tuyến đã biến đổi theo placement, rồi nhập bán kính/chiều cao/
bán kính đỉnh. Chọn điểm gần curve hiện yêu cầu placement giữ tỉ lệ; OnCurve
có thể dùng placement affine tổng quát. Giữ curve nguồn, kiểm tra source stamp
trước commit; line và circle, viewport mô phỏng và Undo đã kiểm. Độ chính xác
điểm gần NURBS tổng quát, tiếp tuyến tại chỗ gãy và hành vi Matrix gốc còn mở.
Tangent + Point: chọn một curve, chọn tiếp điểm, chọn Point rồi chọn một điểm
khác để xác định đường tròn đáy tiếp xúc tại tiếp điểm; hai phía, line/circle,
preview, Undo và viewport mô phỏng đã kiểm. Tiếp tuyến phải song song CPlane;
chọn gần curve yêu cầu placement giữ tỉ lệ. Các ràng buộc tiếp xúc với nhiều
curve, điểm riêng lẻ trong không gian, độ chính xác tổng quát và solver Matrix
gốc chưa được nghiệm thu.
Nếu nguồn AroundCurve hoặc Tangent bị xóa sau khi đã chọn tiếp điểm/tâm,
preview biến mất và commit bị từ chối; kiểm thử bảo đảm không thêm transaction.
Mã kiểm source stamp cho cả thay đổi nguồn, nhưng fixture hiện mới kiểm xóa;
thay hình học/placement và thao tác desktop thực còn cần kiểm.
Preview đáy, hướng cao và hai vòng/đường nối dùng cùng phép tính điểm với commit;
hai chiều cao đã được kiểm so với hình kết quả.
Đường nhập trực tiếp `TCone <đáy> <đỉnh> <cao>` vẫn được giữ để tương thích.
Sau khi chọn 2Point, 3Point, FitPoints, AroundCurve hoặc Tangent, ba số trực
tiếp không được bỏ qua mode đang chọn; Tangent cũng không nhận bán kính dạng
số trước khi hoàn tất ràng buộc đáy. Dữ liệu sai không tạo transaction và có
thể tiếp tục nhập. Đây là quy tắc OM9 đã kiểm, chưa xác minh Matrix gốc.
Thứ tự dựa trên manual; chiều cao có dấu và cách dùng CPlane là lựa chọn OM9,
chưa xác minh solver Matrix gốc. Các nhánh Tangent khác, None/Vertical và độ
chính xác preview tổng quát chưa được nghiệm thu.
# Cylinder height preview — OM9-SOLID-016

OM9 previews two 64-segment rings joined by four generators at the Cylinder height prompt, using the captured base frame, radius and signed height; zero height leaves only the base ring. Preview does not change the document, and invalidated curve sources suppress it. Center and 2Point geometry plus the simulated Center viewport workflow have been checked; GPU display and original Matrix behavior are not verified. See `docs/validation/2026-10-29-solid-cylinder-wire-preview.md`.


### Slab straight-line foundation (partial, 2026-10-29)

Recovered menu evidence routes `SolidSlab` to `_Slab`; the local normalized text does not fully define the command. A Rust-only straight-line profile/extrusion foundation has measured capped-BRep tests for signed offset, signed height and an OM9 BothSides interpretation. Slab remains unavailable: general curves, miter joins, options, viewport, transaction and original behavior are not established. See `docs/validation/2026-10-29-solid-slab-line-foundation.md`.


### Slab open-polyline foundation (partial, 2026-10-29)

Safe Rust can form a mitered strip for an open planar polyline with signed offset and height, bounded size and conservative rejection of degeneracy or self-intersection. An L-shaped fixture measures both offset-side BRep volumes; this is not yet an enabled command. General curves, robust tolerances, options, viewport, transaction and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-polyline-foundation.md`.


### Slab internal publication (partial, 2026-10-29)

A hidden adapter can build and publish an open line/polyline Slab from a source stamp and resolved world parameters. It applies placement, preserves the source and checks stale source before publication; a translated L-shaped fixture verifies BRep volume/bounds and Undo/Redo, stale deletion and closed-curve refusal. The command stays unavailable: interactive input, unit conversion, preview, general curves, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-internal-publication.md`.


### Slab staged development session (partial, 2026-10-29)

A hidden, non-user-facing session accepts one preselected open line/polyline, positive Distance followed by side selection, signed Height in document units, ThroughPoint, BothSides, Undo and Cancel, and uses source-checked publication. A centimeter fixture verifies volume and stale/invalid retry; current-plane and BothSides choices remain provisional. Slab still cannot be invoked from Menu/CMD, and side/through-point, InCPlane, preview, general curves, desktop and Matrix-original behavior remain open. The earlier no-OCCT Sphere/Box build failure was subsequently fixed; see `docs/validation/2026-10-29-solid-no-occt-build.md`. See `docs/validation/2026-10-29-solid-slab-staged-development.md`.


### Slab point preview in development session (partial, 2026-10-29)

For a preselected line, the hidden session accepts an offset-side point and height point; wire preview shares the strip/extrusion geometry with commit, including BothSides. Three signed/BothSides controller fixtures and one simulated two-view pointer fixture pass. Side calculation now uses the nearest polyline segment; see `docs/validation/2026-10-29-solid-slab-nearest-segment.md`. The actual command, nearest-segment side selection, other options, general curves, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-point-preview.md`.


### Slab nearest-segment side (partial, 2026-10-29)

In the hidden session, preview and point acceptance derive signed offset from the nearest open-polyline segment; tied closest segments with inconsistent offsets are refused. A second-segment L fixture checks volume/bounds, ambiguity and Undo, with simulated viewport input. The user command, general curves, complete options, desktop and original behavior remain open. See `docs/validation/2026-10-29-solid-slab-nearest-segment.md`.


### Slab Distance and ThroughPoint development session (partial, 2026-10-29)

In the hidden session, a positive Distance sets offset magnitude and the next point selects its side; ThroughPoint clears that magnitude and takes signed distance from the picked point. Undo returns from height to side selection. Preview and commit agree in tested line cases. User command, InCPlane, general curves, desktop and original Matrix behavior remain open. See `docs/validation/2026-10-29-solid-slab-distance-throughpoint.md`.


### Slab development source pick (partial, 2026-10-29)

The hidden Slab session accepts one preselected open line/polyline or a source chosen after session start. Simulated viewport tests cover source pick, overlapping curve candidates, stale popup after Cancel/restart and Undo. User command remains unavailable; general curves, InCPlane, popup row clicks, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-source-pick.md`.


### Slab source plane in development (partial, 2026-10-29)

For tested open line/polyline sources, the hidden session derives a plane normal from placed source geometry; collinear inputs use a projected preferred CPlane normal. InCPlane=Yes uses the captured CPlane only when the source segments are compatible, otherwise rejects the option; No restores the source-derived plane. Controller and simulated two-view YZ fixtures pass. This is provisional OM9 behavior: incompatible-source projection, general curves, desktop and Matrix-original semantics remain open. See `docs/validation/2026-10-29-solid-slab-source-plane.md`.


### Slab full-circle internal publication (partial, 2026-10-29)

A hidden publication path supports a full circle with circularity-preserving placement by deriving exact annulus parameters in Rust and using OCCT cylinder difference. Rotated/translated fixtures verify inward/outward offsets, negative height/BothSides, volume and Undo/Redo; invalid radius and nonuniform placement are rejected. The hidden interactive session now also accepts a full circle; user command, other curves, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-circle-publication.md`.


### Slab full-circle development session (partial, 2026-10-29)

The hidden session accepts full circles under circularity-preserving placement; point input selects radial offset, preview shows the annulus and the source-checked publication produces the BRep. Inward/outward controller fixtures and simulated viewport selection/height pass. Numeric circle Distance, rotated/mirrored interaction, partial arcs, other curves, desktop and Matrix-original behavior remain open; the user command stays disabled. See `docs/validation/2026-10-29-solid-slab-circle-session.md`.


### Slab arc internal publication (partial, 2026-10-29)

A hidden adapter constructs an exact offset-arc boundary in Rust and extrudes it via OCCT. A rotated/translated quarter-circle fixture verifies inward/outward offsets, BothSides, analytic sector volume and Undo/Redo. Arc selection/preview, seam-crossing and reflex arcs, other curves, user command, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-arc-publication.md`.


### Slab arc development session (partial, 2026-10-29)

The hidden session accepts open circular arcs, computes point offset on the bounded arc and previews the exact-profile extrusion as a wire. Quarter-circle and seam-crossing controller fixtures plus a simulated viewport fixture check volume and Undo. Reflex and transformed arc interaction, general curves, user command, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-slab-arc-session.md`.


### Slab open NURBS internal publication (partial, 2026-10-29)

A hidden path fits an offset for a nonperiodic open planar NURBS, checks 33 sampled expected points against 0.0001 mm and extrudes the resulting profile. A translated rational quarter-circle fixture checks volume and Undo/Redo. The check is sampled and one-sided, not a global accuracy or topology proof; A hidden development session now has limited NURBS interaction; periodic/closed curves, desktop and user command remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-publication.md`.


### Slab NURBS development session (partial, 2026-10-29)

The hidden session accepts nonperiodic open NURBS, selects offset using a bounded nearest-point/tangent calculation and samples a fast wire preview; commit independently fits and sample-checks the offset. A rational quarter-circle controller and simulated viewport fixture verify volume. Preview is approximate, and global accuracy/topology, other NURBS forms, desktop and user command remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-session.md`.

### Slab reversed NURBS development session (partial, 2026-10-29)

The hidden Slab session also accepts reversed nonperiodic open planar NURBS. Offset fitting accounts for traversal direction so the selected side is preserved; a reversed rational quarter-circle fixture checks volume, Undo and simulated viewport interaction. The 33-point fit check is sampled and does not prove global accuracy or topology. Periodic/closed NURBS and public Slab command support remain open. See `docs/validation/2026-10-29-solid-slab-reversed-nurbs.md`.

### Slab NURBS planarity safeguard (partial, 2026-10-29)

Before offset fitting, the hidden Slab path rejects a positive-weight NURBS whose control hull exceeds the requested plane tolerance; unlike sparse sampling, this bounds plane deviation along the full curve. This sufficient test is conservative, especially for trimmed domains. The hidden session now infers the plane from the control hull; preview remains sampled, and public Slab and global offset/topology validation remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-planarity.md`.

### Slab NURBS source-plane inference (partial, 2026-10-29)

The hidden session derives the source plane from all validated, placed NURBS control points and checks the control hull for InCPlane compatibility. A narrow planar knot span missed by 65 samples selects its actual XZ plane and rejects the incompatible XY CPlane; a nonplanar variant is rejected before the session starts. This inference is conservative for trimmed or nearly planar curves. Preview remains sampled; full command, periodic/closed NURBS, global offset/topology and original Matrix behavior remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-plane-inference.md`.

### Slab NURBS knot-aware preview (partial, 2026-10-29)

The hidden session samples uniform stations plus starts and midpoints of active knot spans, retaining actual fractions for tangents and accounting for reversed curves. A narrow knot-span fixture absent from 65 uniform points appears in the preview. This is still approximate; narrow-span closest-point picking, global fit/topology and the public Slab command remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-knot-preview.md`.

### Slab NURBS knot-aware side pick (partial, 2026-10-29)

The hidden session seeds side selection with knot-aware preview stations, refines the best station and retains it if refinement is worse. A narrow-span fixture checks offset in preview; existing forward/reversed NURBS fixtures pass. The search remains approximate and does not certify global nearest points or resolve all ambiguous corners. Public Slab and global offset/topology validation remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-knot-pick.md`.

### Slab competing NURBS knot spans (partial, 2026-10-29)

The hidden session refines every active NURBS knot span as well as the best sampled neighborhood, under a work budget. A controller preview fixture with competing spans now picks the true 0.4 offset instead of 0.6 in both directions. Refinement remains approximate for higher-degree multimodal spans, ambiguous corners and large-curve resource behavior; public Slab and offset topology remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-competing-spans.md`.

### Slab NURBS conflicting side picks (partial, 2026-10-29)

The hidden session rejects a pick if distinct, approximately equally near refined NURBS candidates imply conflicting signed offsets; equal candidates with the same signed offset remain allowed. Controller fixtures check both curve directions, retry and unchanged Undo/document state. Detection depends on bounded approximate search and tolerances; higher-degree multimodal spans, coincident corners, global topology and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-nurbs-ambiguous-pick.md`.

### Slab degree-one NURBS (partial, 2026-10-29)

The hidden session and publication route continuous open nonperiodic degree-one NURBS through the exact placed polyline path, preserving rational weights, traversal and trimmed domain endpoints. A translated L-shaped fixture verifies mitered BRep volume and Undo/Redo in both directions; discontinuous interior knots are refused. Desktop behavior, higher-degree corners, global topology and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-degree-one-nurbs.md`.

### Slab degree-one NURBS simulated viewport (partial, 2026-10-29)

A simulated viewport test selects a rational degree-one L source after hidden-session start, chooses side and height, checks BRep volume within pixel-pick tolerance and Undo for both traversal directions. The no-default-features application check passes. This does not validate the actual desktop or public Menu/CMD; Slab remains disabled. See `docs/validation/2026-10-29-solid-slab-degree-one-viewport.md`.

### Slab disjoint collinear segments (partial, 2026-10-29)

The hidden polyline path allows disjoint collinear boundary segments while still rejecting overlapping or touching segments. Horizontal and vertical-plane fixtures verify closed BRep volume; interactive and desktop behavior for this fixture remain untested. Closed polygon sources, general topology robustness and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-collinear-segments.md`.

### Slab closed-polyline foundation (partial, 2026-10-29)

Rust computes nested mitered rings for closed planar polylines and rejects tested intersections, edge collapse and invalid containment. Rectangle and concave-L fixtures, including direction reversal and inward/outward offsets, produce measured closed BReps by extruding and subtracting rings in a test. It is not connected to document publication or interaction; broader topology, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-polyline-foundation.md`.

### Slab closed-polyline internal publication (partial, 2026-10-29)

The hidden source-stamped publication adapter extrudes and subtracts the two rings of a placed closed line polycurve or continuous degree-one NURBS, retaining the input and inserting one Undoable BRep. Translated rectangle fixtures verify volume, bounds, direction reversal, Undo/Redo and invalid-offset atomicity. The interactive session, closed-polygon viewport, arbitrary topology and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-polyline-publication.md`.

### Slab closed-polyline development session (partial, 2026-10-29)

The hidden session accepts closed line polycurves and continuous degree-one NURBS, chooses offset from the nearest segment, previews both mitered rings and commits through the atomic publication path. Controller fixtures verify both traversal directions, inward/outward and BothSides; simulated viewport fixtures verify source selection, BRep volume and Undo. Arbitrary topology, periodic/higher-degree closed curves, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-polyline-session.md`.

### Slab concave closed-polygon options (partial, 2026-10-29)

A hidden-session concave L fixture verifies outward Distance and inward ThroughPoint, measured volume, preview-to-BRep bounds, Undo and Cancel. This covers one shape in controller tests, not general polygon topology or the desktop. Public Slab and higher-degree/periodic closed curves remain open. See `docs/validation/2026-10-29-solid-slab-concave-closed-options.md`.

### Slab closed NURBS foundation (partial, 2026-10-29)

A Rust foundation fits offset rings for closed clamped nonperiodic planar NURBS and checks sampled fit and polygon topology. A rational ellipse fixture verifies outward/inward, reverse and BothSides BRep volumes against an independent perimeter formula. The checks do not certify global fit or topology; document publication, interaction, periodic NURBS and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-nurbs-foundation.md`.

### Slab closed NURBS internal publication (partial, 2026-10-29)

The hidden source-stamped adapter extrudes and subtracts fitted closed nonperiodic NURBS rings and inserts one Undoable BRep while retaining the source. Translated rational ellipse fixtures check reversed traversal, inward/outward, negative height, BothSides, measured volume and invalid-offset atomicity. Sampled fit/topology, interaction, periodic NURBS and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-nurbs-publication.md`.

### Slab closed NURBS development session (partial, 2026-10-29)

The hidden session captures clamped nonperiodic closed NURBS, previews separate sampled rings, resolves an approximate knot-aware side pick and publishes fitted rings. Rational ellipse controller fixtures check inward/outward, BothSides and reversal; a simulated viewport fixture checks selection, numeric distance, BRep volume and Undo. Preview and commit are approximations, sampled checks do not prove global topology, and periodic curves, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-closed-nurbs-session.md`.

### Slab closed NURBS preview guard (partial, 2026-10-29)

The hidden preview applies the same sampled polygonal topology guard used before fitting and suppresses preview for the tested collapsed inward ellipse offset; changing the distance permits retry without document or Undo mutation. Fit and OCCT can still reject a displayed preview, and global correctness and desktop responsiveness are unverified. See `docs/validation/2026-10-29-solid-slab-closed-nurbs-preview-guard.md`.

### Slab closed NURBS shifted seam (partial, 2026-10-29)

A full-domain nonperiodic closed NURBS with shifted seam is evaluated using its unchanged closed basis at the original seam; the source retains its seam. Ellipse controller fixtures check both directions, volume and Undo; a seam on an open basis is rejected. Periodic/partial-domain seam cases, viewport/desktop and global offset topology remain open. See `docs/validation/2026-10-29-solid-slab-closed-nurbs-shifted-seam.md`.

### Slab shifted NURBS seam simulated viewport (partial, 2026-10-29)

A simulated viewport test selects an ellipse with a shifted seam, sets Distance, picks side and height, measures BRep volume and Undo in both traversal directions. Genuine periodic NURBS, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-shifted-seam-viewport.md`.

### Slab certified periodic NURBS subset (partial, 2026-10-29)

The evaluator and scene admit full-domain periodic NURBS with certified repeated poles/weights and periodic knots; Slab converts that basis to an equivalent closed nonperiodic record for the hidden session and publication without changing the source. Degree-two controller and simulated viewport fixtures verify BRep volume and Undo; uncertified metadata is rejected. Other periodic representations, seam parameter semantics, global fit/topology, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-slab-certified-periodic-nurbs.md`.

### Periodic NURBS repeated seam evaluation (partial, 2026-10-29)

The evaluator uses the final nonempty left span at a repeated right-boundary knot. A certified rational cubic fixture verifies seam continuity and exact extrusion for a simple seam; a double-knot seam evaluates through D1; a later exact trimmed-segment adapter fixes the tested OCCT tessellation failure, as recorded below. Other periodic forms and public Slab remain open. See `docs/validation/2026-10-29-solid-periodic-repeated-seam-evaluation.md`.

### Periodic double-seam exact extrusion (partial, 2026-10-29)

For the tested closed non-clamped rational cubic with repeated seam knots, the OCCT extrusion adapter builds two exact trimmed B-spline edges; forward and reversed bodies have independently checked volume. A hidden Slab controller fixture verifies offset volume and Undo. Other knot arrangements, precision scales, viewport/desktop and general topology remain open. See `docs/validation/2026-10-29-solid-periodic-double-seam-extrusion.md`.

### Periodic double-seam viewport (partial, 2026-10-29)

The tested rational cubic double-seam source can be selected and used to create/Undo Slab through a simulated viewport in both directions. OCCT extrusion also passes signed-height and two affine knot-parameter scale fixtures. Arbitrary periodic representations, extreme ranges, actual desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-periodic-double-seam-viewport.md`.

### Periodic double-seam parameter normalization (partial, 2026-10-29)

The OCCT exact-segment adapter normalizes the repeated-seam B-spline parameter domain before splitting, addressing a tested 1e-9 knot-scale mass-integration failure without changing source geometry. Fixtures also cover a large parameter origin, translation and signed height. Other extreme scales, periodic forms, desktop and public Slab remain open. See `docs/validation/2026-10-29-solid-periodic-double-seam-parameter-scale.md`.

### Periodic knot certification precision (partial, 2026-10-29)

Repeated-periodic knot certification scales tolerance to the period and knot roundoff and refuses an insufficiently resolved period. A tiny nonperiodic mismatch previously admitted by the absolute tolerance is rejected; valid narrow and translated test cases pass. This is conservative, not a proof for all periodic representations. See `docs/validation/2026-10-29-solid-periodic-knot-certification-scale.md`.

### Rib straight planar foundation (partial, 2026-10-29)

Recovered menu routes `SolidRib` to `_Rib`; it does not establish the algorithm. A Rust development adapter interprets one-sided in-plane strip offset and reuses the planar Boss kernel to join straight Rib walls to a bounded plane. A line fixture measures shell area/topology for both directions and signs. This is an OM9 interpretation, not a complete or enabled Rib command; original offset semantics, other modes, boundaries and interaction remain open. See `docs/validation/2026-10-29-solid-rib-straight-planar-foundation.md`.

A hidden development session accepts disjoint linear curves, checks source stamps and commits one boundary mutation with Undo/Redo. These tests establish only this one-sided OM9 development interpretation; the documented offset modes, taper, full interaction, desktop validation and the public command remain open. See `docs/validation/2026-10-29-solid-rib-development-session.md`.
