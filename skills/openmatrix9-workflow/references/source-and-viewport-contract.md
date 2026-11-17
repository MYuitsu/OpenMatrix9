# Hình học nguồn và viewport trong cùng bàn giao

Người dùng yêu cầu ngày 2026-11-13: khi triển khai hoặc sửa lệnh CAD, phải làm cả phía viewport và phía dữ liệu hình học chính xác. Đây là yêu cầu cho công việc sau này; cập nhật hướng dẫn không tự khởi chạy các lệnh còn thiếu trong inventory.

## Kiểm tra từng lệnh: tái dùng, bổ sung và ghi danh sách

Chỉ đạo bổ sung được người dùng duyệt 2026-11-16: mỗi lần được giao kiểm tra,
triển khai hoặc sửa một lệnh CAD, áp dụng luồng nguồn chính xác và viewport
tách biệt, có preview nhẹ trong lúc chờ tác vụ nguồn khi lệnh cần tính lâu.
Mặc định công việc kiểm tra lệnh bao gồm bổ sung phần thiếu của **chính lệnh
được giao**, không dừng ở đề xuất. Yêu cầu rõ là chỉ phân tích/chỉ chạy test/
không sửa code vẫn giữ phạm vi đó; ghi phần thiếu thay vì tự sửa. Lượt cập
nhật skill không tự khởi chạy quét hay triển khai toàn bộ danh mục lệnh.

- Đọc code và bằng chứng trước khi sửa. Phần đã có phải được đánh dấu và tái
  dùng, kiểm lại binding/lifetime của đường sản phẩm hiện tại; không viết lại
  worker, cache hay pipeline chỉ để đáp ứng hình thức của quy tắc.
- Kiểm riêng bốn phần: nguồn canonical đúng semantics/dung sai; representation
  viewport độc lập; preview nhẹ trước kết quả exact khi có chờ; commit/cập nhật
  scene, Undo và các consumer chịu ảnh hưởng. Có Light/High hoặc một worker
  exact không tự chứng minh đã có preview sớm.
- Với tính toán lâu, cho nhánh preview rẻ từ tham số, proxy đã cache hoặc mesh
  hiển thị chạy độc lập với tác vụ nguồn chính xác. Không bắt preview đợi nguồn
  kết quả cuối mới dựng. Preview cũng phải có giới hạn CPU/RAM/GPU; không chạy
  việc nặng trên UI để đổi chỗ freeze. Primitive có thể dùng unit mesh và
  transform; Boolean tái dùng nhánh mesh-preview hiện có nếu phù hợp.
- Preview có trạng thái tạm và job/revision riêng, không trở thành nguồn đo/
  xuất CAD hoặc identity topology chính xác. Khi worker xong, kiểm input,
  context, cancel/stale, binding và budget rồi chuẩn bị scene và commit theo
  transaction. Dùng một bước Undo cho thay đổi nguồn đã commit; nâng chất
  lượng hoặc thay preview không thêm Undo. Lỗi/hủy giữ nguồn và lịch sử đã
  commit, loại preview/kết quả cũ đúng phiên. Không tự cho lệnh sau tính CAD
  trên preview; phụ thuộc exact phải đợi hoặc có queue đã thiết kế rõ.
- Kết quả exact có thể tiếp tục vẽ bằng Light; độ mịn viewport không đổi dung
  sai nguồn. Việc tách owner model không tự chứng minh mọi payload opaque
  sạch display hoặc mọi đường lưu/mở đã hoạt động.
- Lệnh hoàn tất ngay bằng phép biến đổi/buffer hiện có không cần thêm thread
  hay bản hình học thứ hai chỉ để có worker. Lệnh không tạo/sửa hình có thể
  ghi preview là KHONG_AP_DUNG kèm căn cứ cụ thể; vẫn kiểm nguồn/representation
  chịu ảnh hưởng. Không dùng ngoại lệ này để né một tác vụ CAD đang chặn UI.

Duy trì **một danh sách chung trong checkout**:
`docs/validation/command-source-viewport-checklist.md`. Mỗi lần làm lệnh, cập
nhật dòng theo ID thực và mode/route đã rà: phần có sẵn được tái dùng, phần
vừa bổ sung, phần còn thiếu, code/bằng chứng và phạm vi kiểm chứng. Phân biệt
CHUA_RA, THIEU_CAN_BO_SUNG, MOT_PHAN và DA_XAC_MINH; KHONG_AP_DUNG chỉ dùng cho
hạng mục có căn cứ. Không coi CHUA_RA là đã xác định thiếu triển khai.

Chỉ đánh dấu DA_XAC_MINH trong phạm vi đã kiểm đủ phần áp dụng; biên dịch,
code tồn tại hoặc đạt một fixture không đóng cả lệnh/mọi alias. Alias chỉ gộp
khi đã xác nhận cùng owner và mode. Nếu gặp lỗi nền chung như persistence,
ghi blocker và giữ MOT_PHAN; không tự mở dự án sửa toàn lõi ngoài lệnh đang
được giao, cũng không báo lệnh hoàn tất. Danh sách trỏ tới evidence/ledger,
không sao chép toàn bộ nhật ký hoặc đặt tỷ lệ hoàn thành cho mọi lệnh chưa rà.

## Phạm vi thực hiện

- Trước khi sửa, xác định nguồn canonical của input/output: Curve, NURBS, B-rep/topology, placement hoặc mesh do người dùng tạo/nhập. Truy vết caller tính toán và viewport/preview/picking/snap liên quan; không chỉ sửa helper kernel hoặc hình vẽ.
- Trong cùng bàn giao, thực hiện phần hình học nguồn và phần viewport chịu ảnh hưởng, cùng liên kết giữa chúng. Nếu một phía đã đáp ứng, tái dùng và xác minh đường hiện có; ghi căn cứ không cần sửa. Không âm thầm để phía còn thiếu thành công việc tương lai rồi báo hoàn tất lệnh.
- Phía nguồn: đúng input, topology/trim, miền tham số, placement, units và dung sai phép tính. Nêu rõ kết quả chính xác, xấp xỉ theo dung sai hay mới kiểm bằng lấy mẫu. Đọc từ nguồn không tự chứng minh thuật toán chính xác.
- Phía viewport: preview, hình sau commit, Wireframe/Shaded và chọn/bắt điểm phải khớp nguồn trong phạm vi lệnh. Đoạn/tam giác hiển thị có thể tìm ứng viên; cạnh/mặt dùng cho CAD phải được gắn và xác minh với đúng nguồn, phiên bản, scene/camera liên quan. Không dùng index hiển thị như identity nguồn nếu chưa chứng thực; loại kết quả stale.
- Dung sai tính toán/xuất không bị thay bởi độ mịn viewport. Với ExtractWireframe, Silhouette và lệnh phụ thuộc cách nhìn/hiển thị theo hợp đồng, xác định riêng yếu tố được phép phụ thuộc (tập đường, density, hướng nhìn) và độ chính xác hình học kết quả. Không ép mọi kết quả bất biến khi hợp đồng cho phép chúng thay đổi.
- Mesh tác giả/nhập có thể chính là nguồn; giữ semantics của lệnh mesh. Preview hoặc mesh dựng tạm cho phép tính phải có vai trò và sai số riêng, không mặc định là mesh viewport. Không tự tạo hai pipeline, cache hay representation mới khi có thể tái dùng đường đã đúng.

## Kiểm chứng và báo cáo

- Dùng oracle hình học độc lập phù hợp cùng kiểm preview/scene và thao tác chọn liên quan; menu/CMD dùng cùng owner. Khi độ mịn display có thể ảnh hưởng lệnh, kiểm hai cấu hình hiển thị với cùng nguồn/dung sai tính toán và cấu hình xuất khi liên quan; đối chiếu bất biến hoặc thay đổi được phép theo hợp đồng.
- Với trạng thái được lưu hoặc thay đổi, kiểm lưu/mở, Undo/Redo và lỗi/hủy trong phạm vi bị tác động; hình học và scene sau publication phải tương ứng, không tạo kết quả một phần. Không mở lại mọi bộ kiểm thử cho sửa nhỏ không liên quan.
- Phân biệt kiểm kernel/controller, sự kiện egui mô phỏng, GPU ngoài màn hình và desktop thật. Scene đúng không chứng minh GPU đã present; ảnh đẹp không chứng minh CAD đúng. Nếu thiếu một tầng cần thiết thì ghi rõ mức bàn giao và phần chưa xác nhận; không tự mở cửa sổ desktop ngoài yêu cầu.
- Hồ sơ bàn giao ghi riêng: phía nguồn, phía viewport và liên kết chọn/phiên; mỗi phía có bằng chứng hoặc căn cứ không bị ảnh hưởng. Chỉ kết luận hoàn tất trong phạm vi đã kiểm cả hai phía. Không gắn việc này với điều kiện phải hoàn tất source-only/eviction của lịch sử trước.

Tra danh sách nhóm lệnh liên quan trong checkout: `docs/validation/2026-11-13-curve-solid-source-display-inventory.md`. Danh mục là nơi chọn việc có phạm vi, không phải xác nhận mọi lệnh đã chạy hay yêu cầu tự động triển khai toàn bộ.
