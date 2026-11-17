# Chi phí thử nghiệm proof topology

Ngày 2026-11-06, Windows, OCCT 8.0.1, Rust `--release --locked -j 2`, target ngoài checkout `L:/om9-target`. Hai đường chạy xen kẽ trong cùng executable và cùng cấu hình. Số đo theo đồng hồ tường, đơn vị microsecond; 10 mẫu mỗi hình, báo trung vị và lớn nhất, không coi mẫu lớn nhất là p95/p99.

| Phép đo | Fixture | Không proof: trung vị / lớn nhất (µs) | Có proof: trung vị / lớn nhất (µs) |
| --- | --- | ---: | ---: |
| Kernel, đã làm nóng | Box 10 × 10 × 10 mm | 1.737 / 1.806 | 2.752 / 3.963 |
| Kernel, đã làm nóng | Sphere bán kính 5 mm | 2.846 / 2.943 | 2.995 / 3.072 |
| Controller và dựng scene CPU | Box 10 × 10 × 10 mm | 2.011 / 3.547 | 4.665 / 4.718 |
| Controller và dựng scene CPU | Sphere bán kính 5 mm | 21.153 / 21.325 | 22.817 / 23.210 |

Controller tạo mới trước mỗi mẫu, ngoài khoảng đo; khoảng đo gồm tạo B-rep, transaction, dựng scene và, ở đường thử, reservation/proof. Các phép thử không đo GPU, desktop, độ trễ input, peak RAM hay model lớn. Đường kernel dùng callback admission không làm việc; không suy rộng nó thành chi phí của Controller. Mẫu cố định thứ tự baseline trước candidate, không phải nghiên cứu thống kê hoàn chỉnh.

Lệnh đo là `cargo test --release --locked -p om9-kernel -j 2 --lib primitive_topology_capture_release_cost -- --ignored --nocapture` và tương ứng `-p om9-app --lib controller_primitive_topology_release_cost`. Do Box trong fixture tốn thêm khoảng 2,65 ms qua Controller và proof chưa được worker tiêu thụ, đường thử **chưa bật mặc định** trong Sphere/Box. Không kết luận ảnh hưởng tương tác viewport từ số đo này.
