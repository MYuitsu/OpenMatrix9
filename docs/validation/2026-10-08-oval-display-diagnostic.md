# Nhẫn oval: khác biệt Wireframe OM9/Rhino — 2026-10-08

Nguyên nhân chính của hình răng cưa là cách vẽ Wireframe hiện tại của workspace: `CoreWorkspace.cpp` ép shared providers sang Flat Lines, áp SoDrawStyle::LINES theo từng viewport và dùng SoRenderManager::WIREFRAME. Vì thế các tam giác tessellation của mặt được vẽ thành đường. Rhino wireframe cho hình CAD hiển thị đường biên/isocurve; không phải toàn bộ cạnh tam giác tessellation. Khác góc nhìn, zoom và nền cũng ảnh hưởng, nhưng không giải thích phần răng cưa.

Đây là giới hạn display đã ghi trong `skills/openmatrix9-workspace-contract/references/grid-and-viewports.md`, không phải bằng chứng reader làm nhẫn thành bánh răng. Không đổi runtime/thuật toán hình học trong đợt chẩn đoán này.

## Bằng chứng trên cùng file người dùng

- 31 bản ghi nguồn; nhập preserve tạo29 root objects,52 hình học tính cả block definitions.
- Probe độc lập openNURBS/OCCT: 11 đường cong x257 điểm và 312 mặt x169 điểm, sai lệch lớn nhất **1.112884370092863e-14 mm**. Chỉ đo basis evaluation; không thay bằng chứng toàn bộ trim/topology/block-transform hoặc Rhino5 roundtrip.
- Wireframe và tăng độ mịn vẫn tạo đường chéo tam giác; tăng độ mịn tăng số đường. Menu viewport **Shaded** vẽ thân nhẫn liên tục.
- Chuyển cùng document sang native Part CAD Wireframe vẽ các đường biên nhẫn trơn. Hash Shape của các Part::Feature giữ nguyên qua việc đổi chế độ; nguồn3DM giữ nguyên SHA-256. Không sửa dữ liệu để tạo ảnh so sánh.
- [OM9 polygon Wireframe](oval-display-20261008/Wireframe-iso.png), [OM9 Shaded](oval-display-20261008/om9-shaded-iso.png), [native CAD Wireframe](oval-display-20261008/native-cad-wireframe-iso.png), [results](oval-display-20261008/results.json), [hash bindings](oval-display-20261008/manifest.json).

File gốc: `C:/Users/nguye/Downloads/nhan oval 11.91x8.37x4.97.3dm`, SHA-256 `5cdd84533c2dca20ef441fcbe2f1edd0226cc3ab02805caa4af60ef3dbba9997`. Binary chẩn đoán GUI là runtime sửa GIL, không thay source chính hoặc push.

## Cách xem hiện tại và phần còn thiếu

Trong OM9 chọn mũi tên cạnh tên viewport → **Shaded** để xem nhẫn với mặt tô. Wireframe muốn gần Rhino cần adapter vẽ cạnh CAD/isocurve theo viewport thay polygon wireframe; vẫn phải giữ independence4views, picking, document/selection và workbench exit/re-entry. Adapter đó **chưa được sửa/kiểm chứng** ở đợt giải thích này. Không đổi nghiệp vụ đã lưu trong skill.

Một bộ chẩn đoán, một fixture nhẫn. Attempt đầu lỗi serializer Quantity, retry sửa helper; không tính retry thành bộ mới. Chẩn đoán cuối hoàn thành và source không đổi.0 bộ Rhino chuẩn bị đang chờ;7 đợt kế hoạch chưa đóng; tổng số bộ tương lai chưa xác định. Native suite count51 không tăng bởi executable diagnostic không đăng ký ctest. Next: nếu ưu tiên cải thiện display, triển khai CAD Wireframe độc lập từng view và kiểm chứng picking/render/lifecycle; tiếp tục remaining full-exchange matrix riêng.
