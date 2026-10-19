# W6 — port schema option + completion (thuộc M01 `OM9-L2-01-02`)

**Vị trí milestone:** M01 – CAD Shell, Viewport, L2 `OM9-L2-01-02` (Be Command Dispatch Undo Redo).

| Phần W6 | L3 trong M01 | Trạng thái |
| --- | --- | --- |
| Schema option có kiểu (domain/default/required) | `OM9-L3-01-02-005` Command Option Parser | L3 đã nghiệm thu trước ở mức *parser*; schema là phần **mở rộng phạm vi**, không tự đổi trạng thái L3 |
| Schema gắn với descriptor | `OM9-L3-01-02-002` Typed Command Descriptor | `Operation.options` hiện chỉ là `Vec<String>`; kernel này là nửa dữ liệu để nối |
| Completion tên lệnh / tên option / value choice | `OM9-L3-01-02-009` Command Repeat And Dispatch | kernel xếp hạng + cắt theo limit |

## Đã port

| Legacy | Hành vi giữ lại | Target |
| --- | --- | --- |
| `command_option_schemas.rs` (`OptionSpec`/family schema) | mỗi option có **domain có kiểu**, default, required; schema phải tự nhất quán trước khi nhận giá trị | `command_schema::{OptionSpec, CommandSchema}` |
| `command_options.rs` (fill/validate) | defaults nằm dưới giá trị người dùng nhập; tên được chuẩn hoá về cách viết khai báo; từ chối option lạ/thiếu required; mọi giá trị cuối phải qua domain | `CommandSchema::apply` |
| `command_completion.rs` (`rank`/`names`) | xếp hạng exact → prefix → prefix không phân biệt hoa thường → substring, cắt theo limit, thứ tự ổn định | `command_schema::{rank, complete, CommandSchema::complete, value_candidates}` |

Ba quyết định thiết kế:

1. **Schema tự nhất quán trước, giá trị sau.** `OptionSpec::validate` kiểm domain, default phải thoả
   chính domain của nó, và **required + default** bị từ chối vì khi đó "thiếu" là bất khả thi — đó là lỗi
   schema, không phải lỗi nhập liệu.
2. **Không tự bịa giá trị mặc định.** Schema chỉ chứa default do người khai báo đặt; `apply` merge default
   dưới giá trị nhập, không suy diễn.
3. **Chuẩn hoá tên, không chuẩn hoá giá trị.** `apply` trả option theo đúng cách viết khai báo (nhập
   `mode=smooth` → khoá `Mode`), nhưng **giá trị** vẫn bị kiểm bởi domain (nhập `Smoo` vẫn là
   `Invalid("option value/type")`).

## Test (7)

`crates/om9-core/tests/command_schema.rs`:

- schema hợp lệ; từ chối option trùng tên (không phân biệt hoa thường), default nằm ngoài domain,
  required kèm default, alias trùng, tên lệnh rỗng;
- `apply` điền default dưới giá trị nhập, giá trị nhập thắng; chuẩn hoá cách viết tên khoá; từ chối option
  lạ, required thiếu, và giá trị sai chiều (`Invalid("missing unit")`);
- `rank`/`complete`: exact → prefix → prefix không phân biệt hoa thường → substring; limit; input rỗng trả
  toàn bộ theo tên; ứng viên không khớp bị loại;
- `CommandSchema::complete` trả tên option đã khai báo; `value_candidates` trả values của domain Choice
  theo prefix, domain khác trả rỗng.

## Còn lại của W6

- **Nối vào descriptor**: `Operation.options: Vec<String>` → mang `CommandSchema` (hoặc tham chiếu schema
  theo `feature` id). Cần quyết định: schema đi kèm descriptor hay là catalog riêng? (legacy dùng catalog
  theo family số; target nên theo `feature` string.)
- **Nhập option trong commandline**: `om9-app/src/option_parser.rs` hiện parse `name=value` với bảng đơn vị
  chung; khi có schema thì parser nên lấy domain từ `CommandSchema` thay vì suy từ descriptor để lỗi
  khớp (`Invalid("unknown option")` / `Missing("required option")`).
- **`cage_commands.rs`** (16 KB, trong W6 nhưng nội dung là cage/SubD): không thuộc command framework —
  xếp lại vào W9 cage/subd.
- **`command_options_ffi.rs`/`menu.rs`/`commands.rs`**: `LEGACY_HOST`, không port.

## Ghi chú phạm vi

W6 phục vụ M01 `OM9-L2-01-02`; các L3 ở đó đã được nghiệm thu ở mức hiện tại, nên W6 **không** đổi trạng
thái L3 nào — nó bổ sung nửa dữ liệu (schema + completion) để lần mở rộng L3 sau có sẵn.
