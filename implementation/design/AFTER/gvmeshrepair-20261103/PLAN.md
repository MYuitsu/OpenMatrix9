# Kế hoạch gvMeshRepair cho OM9

Ngày 2026-11-03. Trạng thái: **nghiên cứu và đề xuất; chưa triển khai hay nghiệm thu**. Phạm vi chủ sở hữu `OM9-L2-10-02`, các L3 sửa mesh/giảm mesh và báo cáo; kết nối xuất STL thuộc `OM9-L2-01-12`. Chủ sở hữu runtime, kiểm thử và giao diện giữ theo milestone; yêu cầu mới này không tự chứng minh tính năng AFTER đã hoàn tất.

## Mục tiêu và giới hạn thực tế

- Đầu vào mesh người dùng, B-rep/NURBS có nguồn hợp lệ, nhiều đối tượng và phép đặt trong hệ tọa độ tài liệu. Kết quả giúp in 3D: kín, hướng nhất quán, manifold theo định nghĩa kiểm tra, không suy biến/tự giao ở mức đã kiểm chứng; mọi lỗi còn lại được trình bày rõ. Không tuyên bố đã sẵn sàng sản xuất nếu kiểm tra thiếu, hết ngân sách hoặc còn lỗi.
- Giảm dung lượng STL khi có thể **dưới dung sai không gian đầu ra**, bảo vệ mép sắc, lỗ, rãnh, ổ đá, thành mỏng và các mảnh có chủ đích. Không đặt mục tiêu giảm % hay kích thước file cứng bất chấp độ chính xác; nếu không giảm an toàn, giữ nguyên và giải thích.
- Với B-rep gốc: tạo mesh xuất riêng từ nguồn và kiểm tra khoảng cách đến nguồn, không suy ra độ chính xác từ ảnh viewport. Với mesh nhập: so sánh với mesh gốc; không thể khôi phục độ chính xác hình học đã mất trước khi nhập.
- **Lấp lỗ tạo ra hình học chưa biết**: không thể đồng thời bảo đảm đúng mặt gốc và kín với mọi đầu vào. Trình bày phần bù là tái dựng, kiểm riêng thay đổi trên bề mặt đã tồn tại và diện tích/hình học được tạo; chỉ gọi là xác thực theo nguồn nếu có nguồn tham chiếu đủ và phép kiểm chứng thích hợp. Lỗ có thể là thiết kế có chủ đích; không tự động bịt mọi biên.
- Mặc định giữ nguyên đối tượng gốc; lệnh tạo kết quả mới và có Undo/Redo. Sửa lỗ, hợp các vật thể hoặc xóa thành phần cần được giải thích và xác nhận trong preview. Không thay đổi nghiệp vụ Matrix/skill trước khi trình bày khác biệt và được chấp thuận.
- STL nhị phân có 84 byte header/count và 50 byte mỗi tam giác: 1.000.000.084 byte tương ứng 20 triệu tam giác. Không nhầm dung lượng tệp với đỉnh RAM. STL không lưu đơn vị hay topology dùng chung, đỉnh là `f32`; lưu `f64` trong nội bộ đến bước ghi cuối và kiểm tra file đã lượng tử hóa.

## Nghiên cứu và đối chiếu

- Matrix reference: `docs/analysis/2026-11-03-gvmeshrepair-behavior.md`. Lớp managed tạo tam giác, gọi Netfabb, kiểm mesh, tách phần rời và gọi Rhino MeshRepair. Tài liệu nói sửa lỗ/đỉnh trùng, nhưng thuật toán Netfabb là hộp đen; nguồn phục hồi có dấu Matrix 8.5 nên chưa chứng minh tương đương runtime Matrix9. Mặc định và các tham số là tham khảo UI, không sao chép nhánh Milgrain giới hạn bằng số chia khi cần đảm bảo dung sai.
- OM9 `crates/om9-core/src/mesh_repair.rs`: có weld/loại mặt, hướng, `fill_holes` dạng quạt về centroid, kiểm tự giao và QEM-guided reducer. Weld theo ô lượng tử chưa phải chứng minh khoảng cách/láng giềng và bảo toàn thành mỏng; hole fan có thể sai với biên lõm/không phẳng; reducer theo số mặt chưa chứng minh sai số và topology, nên **không dùng nguyên trạng làm đường sản xuất**. Hàm self-intersection ở file này quét từng cặp O(n²), không tự mở rộng cho mesh lớn.
- OM9 `mesh_csg.rs` có repair giao cắt theo nonzero winding và kiểm embedding với giới hạn; evidence `docs/validation/2026-10-23-mesh-csg-curved.md` chỉ chứng minh fixture được kiểm, đầu vào đóng/hướng đúng và ngân sách giới hạn. Không coi là bộ sửa mesh hở tổng quát. `session/stl.rs` đã có capture, kiểm topology, lượng tử hóa, ghi tạm/đổi đích và receipt; receipt hiện nêu tự giao và sai lệch toàn cục chưa được kiểm.
- [OCCT meshing](https://github.com/Open-Cascade-SAS/OCCT/wiki/mesh) là nguồn tạo mesh từ B-rep theo deflection/góc; [thử nghiệm cầu của OM9](../../../../docs/validation/2026-11-03-sphere-stl-mesher-memory.md) đã chứng minh lợi ích Delabella trên một fixture, không đại diện mọi hình. Đánh giá từng loại hình học, không thay thuật toán trên toàn bộ scene chỉ vì một kết quả.
- [CGAL mesh repair](https://doc.cgal.org/latest/PMP_Mesh_repair/index.html) mô tả orientation, vá lỗ và cảnh báo bản vá có thể tự giao. [CGAL bounded Hausdorff](https://doc.cgal.org/6.2/Polygon_mesh_processing/index.html) mô tả ước lượng với sai số tính toán bị chặn; nghiên cứu làm oracle độc lập, không gọi lấy mẫu là chứng nhận. Các gói CGAL liên quan có [giấy phép GPL](https://doc.cgal.org/latest/Manual/packages.html), đánh giá tương thích trước bất kỳ phụ thuộc phân phối nào.
- [meshoptimizer](https://github.com/zeux/meshoptimizer) có giảm mesh và giới hạn lỗi tuyệt đối, MIT, nhưng là C/C++/FFI và thước đo riêng của bộ giảm **không tự chứng nhận** Hausdorff hai chiều, topology hoặc f64; chỉ là ứng viên benchmark sau kiểm giấy phép/FFI/chất lượng. Ưu tiên Rust và tái sử dụng dịch vụ hiện có nếu đáp ứng oracle, không quyết định thư viện trước kết quả. GPU rendering/LOD không phải bộ sửa hoặc bảo đảm sai số xuất STL.

## Hợp đồng độ chính xác và bảo toàn

1. Người dùng đặt dung sai tuyến tính xuất trong đơn vị tài liệu và góc riêng; mặc định theo tài liệu như luồng xuất đang có. Mọi tính toán và báo cáo chuyển đúng về hệ tọa độ xuất, kể cả scale không đều, mirror, dịch chuyển lớn và đổi đơn vị.
2. Chia ngân sách sai số có kiểm soát cho mesh nguồn, sửa/phép hợp, giảm mesh và lượng tử hóa STL; tổng các **cận trên đã chứng minh trong cùng hệ tọa độ và cùng miền so sánh** không được vượt dung sai. Không chia cứng tỷ lệ khi chưa có benchmark; thuật toán không cung cấp bound phải bị xem là chưa chứng minh. Cho phép trường hợp giảm được chính xác bằng 0 sai số hình học mà không cần ép phần ngân sách không tồn tại.
3. `QEM cost`, OCCT deflection và kiểm tra số mẫu chỉ là tiêu chí/ước lượng riêng, không tự bảo đảm sai số Hausdorff tuyệt đối. Với mesh đối chiếu: ưu tiên oracle hai chiều có cận trên và sai số số học được kiểm soát; nếu chỉ lấy mẫu thì ghi rõ số mẫu, phương pháp, sai số quan sát và **chưa chứng nhận**. Với B-rep: đối chiếu hai chiều tới B-rep thực bằng oracle có phạm vi chứng minh; nếu không có, báo lấy mẫu/không xác minh, không nâng thành bảo đảm.
4. Repair làm thay đổi topology hoặc điền lỗ: báo riêng vùng bị thay đổi, sai lệch trên các phần còn so sánh được, và phần tạo mới không có dữ liệu gốc; không cộng số đo ở các miền khác nhau thành một chứng nhận tổng quát. Trường hợp thay đổi topology có chủ ý có thể được chấp nhận sau xác nhận nhưng không nhận nhãn bảo toàn hình dạng gốc.
5. Kiểm tra trước và sau encode STL, sau đó đọc lại tệp tạm để đối chiếu tam giác, hữu hạn, lượng tử hóa, sập tam giác, số bytes và topology phù hợp. Chỉ công bố file sau kiểm tra; lỗi/hủy bảo toàn đích. Tách index topology của mesh nội bộ và soup STL được tái dựng với quy tắc ghép tọa độ rõ ràng; báo sai khác/không xác minh thay vì gộp đỉnh vượt ngưỡng.

## Thứ tự thực hiện

### P0 — Fixture, baseline và oracle (điều kiện chặn)

- Chuẩn bị bộ fixture công khai/tự tạo, tránh mô hình riêng tư: cầu giải tích (nhiều kích thước/dung sai), hộp và mặt phẳng, hình xuyến, nhẫn/milgrain, lỗ nhỏ/rãnh/thành mỏng, nhiều vật thể chạm/gần nhau, cavity, mesh nhập hở/không manifold/tự giao, biên lõm/không phẳng, scale không đều/mirror và tọa độ lớn. Thêm mẫu thực khi người dùng cho phép và giữ riêng tư.
- Đo release cùng fixture/phần cứng, `-j 2`, target ngoài checkout. Ghi số tam giác và bytes STL, median/p95/p99 thời gian theo nhiều lượt, đỉnh working set và peak commit Windows, sai số hai chiều có phạm vi cụ thể, số lỗi topology/tự giao, kết quả encode/decode. Với bản cũ ghi rõ những chỉ số chưa thể đo, không suy ra kết quả từ mẫu khác. Tách RAM/RSS, cấp phát OCCT và kích thước STL.
- Xây validator độc lập có kết quả `pass/fail/unknown`, budget thời gian/bộ nhớ và cơ chế hủy: xác thực indices/tam giác, cạnh biên và non-manifold, link đỉnh, winding và shells/cavities, trùng/đối diện, giao cắt cả coplanar, khoảng hở và lượng tử hóa. Xác định rõ xử lý tiếp xúc hợp lệ giữa thành phần. BVH/kiểm tra robust phù hợp; benchmark và từ chối khi không đủ khả năng, không trả `pass` khi timeout. So sánh đối chứng oracle độc lập/giải tích có phiên bản và giấy phép rõ.
- Thiết lập oracle sai số hai chiều và kiểm thử sai số của chính oracle; phép lấy mẫu là fallback phân loại riêng. Chỉ tiếp tục các khẳng định accuracy sau khi có phép kiểm thích hợp; nếu oracle chưa xong vẫn có thể phát hành chẩn đoán, không quảng cáo bảo đảm dung sai.

### P1 — Chẩn đoán và preview không phá hủy

- Một service Rust trả `AnalysisReport` gồm source revision/IDs/units/placement, lỗi trước sửa và vị trí/loại, số thành phần, kích thước/tam giác ước tính, trạng thái kiểm chứng. Menu Tools và CMD `gvMeshRepair` dùng chung dispatcher; giữ icon, default invocation và phím tắt gạch chân theo contract. Hiển thị viewport highlight và danh sách lỗi, hỗ trợ hủy.
- Chụp dữ liệu bất biến, đặt trước RAM có quản lý theo chính sách 60% hiện có ở mức đã triển khai thực sự, tính đỉnh khi cùng giữ nguồn/kết quả/scratch; kiểm RAM trống và từ chối an toàn. Không tuyên bố chặn mọi cấp phát OCCT nếu chưa có đo và reservation phù hợp. Worker không khóa UI; source đổi thì preview lỗi thời không được ghi đè document, người dùng có thể chạy lại.
- Gate: menu/CMD, chọn nhiều đối tượng, đơn vị và hủy/ngoại lệ; fixture có lỗi và fixture sạch cho kết quả đúng, không tạo history hay chỉnh dữ liệu.

### P2 — Sửa bảo thủ, có xác nhận

- Kiểm/chuẩn hóa index, loại mặt suy biến hoặc trùng chỉ khi phân biệt được trường hợp hợp lệ; xử lý winding theo từng shell và cavity. Weld chỉ khi đủ điều kiện topology, khoảng cách thực và budget dịch chuyển; tránh ghép hai bề mặt gần nhau/đỉnh chuỗi bắc cầu. Kết quả độc lập nguồn, kiểm chứng lại sau từng bước.
- Dùng CSG repair hiện có **chỉ** cho phạm vi đầu vào/budget đã hỗ trợ; trước thao tác hợp nhiều đối tượng chọn chính sách vật liệu/union và cảnh báo mảnh/tính chất bị thay đổi. Không xóa mảnh nhỏ tự động. Không dùng hole fan centroid cho biên tùy ý.
- Phân loại lỗ/biên; chỉ đề xuất vá khi có vòng đơn hợp lệ và điều kiện hình học được kiểm, bằng triangulation có kiểm self-intersection, winding, manifold và preview vùng vá. Khác biệt giữa khe hàn có nguồn tham chiếu và khoảng trống cần tái dựng phải hiện rõ. Khi không suy ra ý định, yêu cầu xác nhận cụ thể; không hứa mọi lỗi đều sửa được. Hoàn tác giao dịch khi validation thất bại.
- Gate: vòng lõm, vòng không phẳng, rãnh có chủ đích, hai lớp gần nhau, cavity, các lỗi trùng/giao; test lỗi/hủy/Undo/Redo, mã màu preview, report trước/sau; xác nhận unsupported là lỗi rõ ràng, nguồn và Redo nguyên vẹn.

### P3 — Giảm mesh có ràng buộc (tách biệt P2)

- Với B-rep, ưu tiên mesh trực tiếp từ nguồn ở cấu hình xuất và kiểm sai lệch; so sánh nhiều cấu hình/thuật toán trên cùng tolerance, tránh sinh mesh quá dày rồi mới giảm nếu không cần. Giữ Delabella chỉ trong phạm vi cầu đã kiểm; không tự bật LOD hay giảm resolution viewport.
- Với mesh cần giảm, thử QEM/edge collapse trong Rust với priority queue, local adjacency/BVH và kiểm tra ràng buộc link condition, orientation, manifold, intersections và cạnh/đặc trưng cần khóa sau mỗi thay đổi phù hợp. Cập nhật đặc trưng động; không dùng ngưỡng số tam giác hoặc QEM cost làm bằng chứng sai số. Bảo vệ các mảnh, lỗ, khe, bề dày; kiểm riêng những đặc trưng dưới tolerance và không tự xóa. So sánh ứng viên thư viện, chi phí native/giấy phép, thời gian và bộ nhớ với baseline trước quyết định.
- Tính/kiểm cận sai số nguồn tới kết quả sau giảm; nếu không thể chứng minh thì giữ kết quả trước giảm hoặc cho preview nhưng ghi chưa xác minh, không gọi là giữ chính xác. Tính cả lỗi chuyển đổi f32 và placement theo hợp đồng. Dừng khi lỗi/topology không đạt, không ép file size; thông báo số tam giác/bytes trước sau và lý do dừng.
- Gate: sphere có oracle giải tích, mặt phẳng, lỗ và thành mỏng; kiểm chênh lệch hai chiều, genus/thành phần/khe, bình thường hướng, tự giao và STL đọc lại. Test số lượng tam giác không giảm trong trường hợp không thể giảm an toàn; không khẳng định luôn nhỏ hơn.

### P4 — Tích hợp kết quả và xuất STL

- UI phân biệt **Kiểm tra**, **Sửa**, **Giảm**, **Xuất**; trình bày trước/sau, dung sai, tình trạng kiểm chứng, phần được thêm, cảnh báo phần hở, thao tác không thể chứng minh, số tam giác và dung lượng dự kiến. Xem trước không thay bản vẽ; chấp nhận một transaction; Undo/Redo khôi phục geometry, IDs/selection/layer và metadata đúng hợp đồng.
- Xuất lấy capture bất biến tại thời điểm bấm, độc lập viewport; bình thường có thể tiếp tục chỉnh sửa. Reuse worker/receipt/ghi file tạm của STL hiện tại. Một export active, hỗ trợ hủy theo khả năng; đích không bị thay khi lỗi; đóng/New/Open theo lifecycle đã chốt. `gvMeshRepair` không tự thay đổi STL export hay âm thầm lấp lỗ trong một lệnh Xuất bình thường.
- Đối chiếu Matrix: original `DeleteInputObjects` mặc định tắt, `ExtendedRepair` bật trong managed reference; quyết định OM9 và điểm khác biệt như preview/xác nhận được đưa người dùng duyệt trước khi ghi business rule vào skill. Test cả menu/CMD, project save/reopen, object/layer/selection, escape/focus, hủy trong native work và worker kết thúc, race document replacement; không tuyên bố native call hủy tức thời nếu API không hỗ trợ.

### P5 — Đo và nghiệm thu release/desktop

- Chạy Rust tests phù hợp, independent STL validator, public-source audit và dọn artifact theo quy định. Test debug và release riêng, runtime DLL đầy đủ, target ngoài checkout, `-j 2`; không can thiệp phiên OM9 đang mở.
- So sánh trước/sau cùng máy/fixture/cấu hình và nhiều lượt; công bố median/p95/p99, đỉnh process commit/working set, số triangles/bytes, sai số có phạm vi và mức chứng minh, topology/tự giao, thời gian UI và hủy. Bảng tách candidate accepted/refused/unknown; tuyệt đối không nói `0 lỗi` khi kiểm không hoàn tất.
- Kiểm desktop thật riêng: lựa chọn, preview, lỗi, xác nhận, Undo/Redo, xuất và hủy; kiểm bằng slicer/viewer độc lập và roundtrip tệp thật. Không dùng headless làm bằng chứng UI hoạt động. Cập nhật milestone/ledger/README theo bằng chứng, không đánh dấu toàn bộ tính năng hoàn thành từ một fixture.

## Điều kiện hoàn thành tối thiểu

1. Kết quả được tuyên bố kín phải vượt qua kiểm tra topology và tự giao tương ứng, không có `unknown`; vật thể hở được báo rõ và không bị dán nhãn kín.
2. Không đưa ra bảo đảm giữ dung sai nếu không có phép kiểm chứng thích hợp hai chiều cho miền được so sánh; phần bù lỗ được nhận diện riêng. Không đổi đối tượng gốc hoặc mất Undo/Redo khi lỗi/hủy.
3. Giảm mesh chỉ xuất khi kiểm chứng sau encode vẫn đạt hợp đồng; không thể giảm an toàn thì xuất chất lượng ban đầu hoặc cho người dùng quyết định theo trạng thái kiểm chứng, không hứa % giảm.
4. Dùng nguồn/hệ tọa độ/đơn vị chính xác, không dùng mesh viewport thay cho dữ liệu xuất; binary STL sau đọc lại hợp lệ và giữ nguyên đích nếu thất bại.
5. Có số đo release, test geometry/topology/interaction và kiểm desktop theo phạm vi thực; mọi giới hạn còn lại được công khai.
