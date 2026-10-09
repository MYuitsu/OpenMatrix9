# Hướng dẫn code mẫu OpenMatrix9

Các feature spec là hợp đồng thiết kế và hướng dẫn triển khai OpenMatrix9. Mỗi file giữ ID, tên lệnh tương thích, trạng thái triển khai và evidence kiểm thử hiện có, đồng thời có phần đầu vào, options, quy trình xử lý và mẫu Rust riêng.

## Phạm vi mẫu

Mẫu Rust tạo **operation plan sau khi kiểm tra request**, dùng thư viện mẫu độc lập trong [`examples/rust`](examples/rust/README.md). Nó có thể biên dịch và kiểm thử riêng, nhưng không tạo geometry trong FreeCAD, không kích hoạt command chưa hỗ trợ, và không chứng minh toàn bộ feature đã được triển khai.

`FeatureContract` là chính sách của mẫu OpenMatrix9: vai trò đầu vào, loại dữ liệu option, số lượng tối thiểu/tối đa và loại tác động. Cardinality/required của mẫu không tự trở thành default hoặc giới hạn tương thích. Bảng tham số và phần thiết kế của feature ghi rõ thông tin đã xác định, ví dụ và chi tiết còn chưa xác định.

Một số mẫu chỉ khai báo vai trò cho nhánh đầu vào chính. Khi triển khai nhánh
khác, bổ sung role/schema và validator theo phase; giữ nhánh đó unsupported
trong mẫu hiện tại. Không bỏ kiểm tra unknown-role để đưa operand chưa được mô
hình hóa vào solver, và không coi request fixture trong bài test là dữ liệu
hình học hợp lệ cho mọi nhánh của feature.

Không tự điền default vào request. `Choice` kiểm tra kiểu và chuỗi không rỗng; adapter của feature phải kiểm tra tập giá trị thực sự hỗ trợ. `Number` chỉ kiểm tra hữu hạn; đơn vị, miền giá trị, độ chính xác và quan hệ giữa options phải được xử lý theo feature. Object key là tham chiếu opaque: host vẫn phải resolve đúng document/object/subelement và kiểm tra loại hình học.

`document_revision` là trường của thư viện mẫu, không phải tên property hay API
đã có của FreeCAD. Adapter phải có cơ chế nhận thay đổi phù hợp và kiểm tra cả
identity của document/session/input; hai document khác nhau có cùng số revision
không được coi là cùng snapshot. Guard số revision của mẫu chỉ minh họa một phần
của kiểm tra stale input.

## Các lớp xử lý

Trước khi chọn solver/adapter, đọc chương tương ứng trong
[nhóm 00 Rhino/FreeCAD](specs/00-rhino-core/README.md). Nhóm này phân biệt
primitive geometry có sẵn, UI của workbench khác và phần command/state phải
tích hợp vào OM9; các bảng là evidence từ source, chưa phải native validation.

| Lớp | Trách nhiệm |
|---|---|
| Rust | Đọc request có kiểu, xác thực options, duy trì session, tính toán thuần khi phù hợp, tạo plan bất biến và báo lỗi có cấu trúc. |
| C++/Qt | Nhận selection và input, chuyển hệ tọa độ, resolve tham chiếu, gọi Part/OpenCascade hoặc dịch vụ thích hợp, quản lý preview và transaction. |
| Python | Đăng ký workbench và hỗ trợ fixture. Command native dùng handler C++ đã đăng ký; không tạo một handler Python riêng dẫn tới hành vi lệch giữa menu/CMD/F6. |

Mỗi mẫu định nghĩa `plan(&Request) -> Result<OperationPlan, PlanError>`. API này thuộc thư viện **mẫu**, không phải tên API đã có của workbench.

## Trình tự gắn vào host

1. Kiểm tra capability hiện có của exact feature ID. Khi thiếu adapter hoặc option, giữ command/option vô hiệu hoặc trả lỗi unsupported cụ thể.
2. Thu thập input có kiểu theo đúng thứ tự. Với đối tượng trong hierarchy, dùng world placement thực tế; lưu identity/subelement và phiên bản input.
3. Đọc options, chuyển đơn vị ở một ranh giới rõ ràng và tạo request. Mẫu không chấp nhận tên role/option lạ và không bỏ qua giá trị không hợp lệ.
4. Chạy validator Rust rồi kiểm tra miền và quan hệ hình học đặc thù. Một request có cấu trúc hợp lệ chưa chứng minh curve/surface/mesh hợp lệ.
5. Tạo preview trong scene graph hoặc renderer tạm. Preview không thêm document object hoặc undo entry. Hủy, đổi document, đóng document hoặc deactivate workbench phải giải phóng preview.
6. Khi commit, xác nhận document/input vẫn là phiên bản đã dùng. Tính lại geometry cuối cùng; chỉ mở transaction sau khi kết quả đã qua kiểm tra phù hợp.
7. Ghi geometry và metadata cùng transaction, recompute và commit. Exception phải abort; không giữ kết quả từng phần. Kiểm tra Undo/Redo và FCStd save/reload bằng fixture native.

Với thao tác chỉ đo/đọc hoặc đổi camera, không tạo transaction geometry. Với export/render/project library, dùng luồng I/O riêng: đường dẫn, lỗi dịch vụ, tệp tạm, hủy và quyền ghi phải được xử lý ở adapter; không dùng một transaction document để giả lập tính nguyên tử của hệ thống tệp.

## Khung commit C++ cho adapter native

Đây là khung tích hợp dùng các phương thức transaction của `App::Document`.
Callback `apply` phải ghi toàn bộ output và metadata; solver đã tạo/kiểm tra
kết quả trước khi gọi. Callback kiểm tra revision phải xét cả input references.
Host chạy trên luồng GUI, giữ document hợp lệ và không lồng transaction.
Khung này cần biên dịch với SDK FreeCAD khi triển khai adapter; kết quả kiểm
thử Rust dưới đây không xác nhận nó đã chạy native.

```cpp
#include <App/Document.h>
#include <stdexcept>

template<class RevisionIsCurrent, class Apply>
void commit_feature(App::Document& doc, const char* feature_id,
                    RevisionIsCurrent revision_is_current, Apply apply)
{
    if (!revision_is_current())
        throw std::runtime_error("Feature inputs changed; rebuild the plan");

    doc.openTransaction(feature_id);
    try {
        apply(doc);
        doc.recompute();
        doc.commitTransaction();
    } catch (...) {
        doc.abortTransaction();
        throw;
    }
}
```

Kiểm tra kết quả recompute và trạng thái object theo API/SDK đang dùng; một
solver trả shape chưa đủ để bỏ qua kiểm tra solid count, orientation, volume,
mesh manifold hoặc lỗi dependency. Không dùng khung này cho camera/đo đọc/I/O.

## Dùng mã giả native để hoàn thiện hợp đồng

Mã giả Ghidra hỗ trợ xác định nhánh điều kiện, thứ tự gọi và dữ liệu đi qua
các bước. Trước khi chuyển thành behavior, kiểm tra hàm tương ứng với đúng
command/event, đối chiếu assembly, calling convention, kiểu dữ liệu và API
được gọi. Tên biến, struct và kiểu pointer suy ra chưa ổn định cần ghi là chưa
xác định. Đừng chuyển lỗi COM/runtime thành một phép toán hình học giả định.

Tách điều kiện vào/ra và lỗi thành hợp đồng riêng, viết code mới với API host
rõ ràng, rồi kiểm chứng bằng fixture. Không chép listing vào spec hoặc xem mã
giả như source C++ sẵn sàng build. Những điểm chưa có bằng chứng giữ dưới dạng
unknown/unsupported; đối sánh tên gần giống không xác nhận đúng implementation.

## Ví dụ sử dụng request

```rust
use om9_spec_examples::{Input, InputKind, Request, Value};

let mut request = Request {
    document_revision: Some(7),
    ..Request::default()
};
request.inputs.push(Input {
    role: "endpoints".into(),
    kind: InputKind::Point,
    key: "P0".into(),
    point: Some([0.0, 0.0, 0.0]),
});
request.options.insert("distance_mm".into(), Value::Number(1.0));
```

Đây là minh họa API. Khi dùng một feature cụ thể, lấy tên role/option và số lượng từ `CONTRACT` trong chính spec đó; không mặc định `endpoints` hoặc `distance_mm` có ở mọi feature.

## Kiểm tra bản mẫu

```text
cargo test --manifest-path OpenMatrix9_Codex_Spec_v1/examples/rust/Cargo.toml
```

Kiểm thử chung kiểm tra input thiếu, NaN, option lạ, revision thiếu hoặc cũ, tham chiếu lặp và snapshot độc lập. Kiểm thử sinh theo từng feature kiểm tra tất cả mẫu có thể biên dịch, request hợp lệ tạo plan mang đúng ID, option lạ bị từ chối, role bắt buộc không được thiếu và option số không được NaN. Các fixture hình học đặc thù trong feature là yêu cầu nghiệm thu tương lai, không được tính thành kết quả đã chạy của mẫu.

## Những gì được giữ riêng

Không nhúng raw listing, assembly, địa chỉ native, bitmap thương mại, đoạn văn trích nguyên văn hoặc thông báo nguồn vào feature spec. Nhật ký phân tích cục bộ được lưu ngoài package. Các file gốc và thông báo giấy phép của chúng không được sửa/xóa. Việc tổ chức lại tài liệu không xác nhận quyền công bố toàn bộ cây thư mục.
