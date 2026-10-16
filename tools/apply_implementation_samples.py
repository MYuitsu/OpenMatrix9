"""Attach authored Rust code/algorithm sketches to M01 and M02 L4 records.

For every L4 in M01 and M02 this adds an ``implementation_samples`` block: a
skeleton shaped by the task's own role, plus the algorithm notes, the positive and
negative oracles and the reference targets that are already on the record.

The sketches are authored scaffolding, not verified code:

- they never name a project symbol that is not already in the task record;
- the runbook section says plainly that a sketch is not evidence;
- ``validate_implementation_samples.py`` fails closed if the honesty flags are
  missing, if a block claims a runtime status, or if the sketch drifts from what
  the template produces today.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import implementation_sample_templates as tpl  # noqa: E402
import apply_matrix_ref_supplements as refs  # noqa: E402

MILESTONES = {
    "M00": "M00 - Feasibility, QymCAD Source, OCCT, Legal and Environment",
    "M01": "M01 - CAD Shell, Viewport",
    "M02": "M02 - Viewport, Gumball",
    "M03": "M03 - Curve, Surface, Solid and 3DM Exchange",
    "M04": "M04 - Builder and History",
    "M05": "M05 - Gems and Layout",
    "M06": "M06 - Settings and Prongs",
    "M07": "M07 - Cutters",
    "M08": "M08 - Jewelry Modeling",
}

# The M03/M04/M05/M06/M07 records label their tasks with a different role
# vocabulary from M01/M02. Each label maps onto the template that fits it; labels
# with no template are skipped rather than guessed.
ROLE_ALIASES = {
    "SOURCE_MAP": "SOURCE_AUDIT",
    "SOURCE_AUDIT": "SOURCE_AUDIT",
    "CONTRACT_FIXTURE": "CONTRACT_TEST",
    "API_FIXTURE": "CONTRACT_TEST",
    "CONTRACT_TEST": "CONTRACT_TEST",
    "IMPLEMENT": "IMPLEMENT",
    "INTEGRATE": "INTEGRATE",
    "INTEGRATION_STRESS": "REGRESSION_STRESS",
    "REGRESSION": "REGRESSION_STRESS",
    "REGRESSION_STRESS": "REGRESSION_STRESS",
    "ACCEPT": "ACCEPTANCE",
    "ACCEPTANCE": "ACCEPTANCE",
    "ADVERSARIAL": "REGRESSION_STRESS",
    "DISPATCH_CONTRACT": "CONTRACT_TEST",
    "WIRE_UI": "INTEGRATE",
    # M03 dispatch/adapter tasks: a command entry point that forwards to the owning
    # domain service. INTEGRATE is the template that states that boundary.
    "DISPATCH": "INTEGRATE",
    # M08 (E13) task roles. MENU_BINDING_ACCEPTANCE is the M08 form of a command/
    # menu adapter that must dispatch to the owning service, so it uses INTEGRATE.
    "MENU_BINDING": "INTEGRATE",
}

SAMPLES_KEY = "implementation_samples"
GUIDE_KEY = "implementation_sample_guide"
RUNBOOK_HEADING = "## Code mẫu / thuật toán mẫu (sketch, không phải bằng chứng)"
SCHEMA = "om9.l4.implementation_samples.v1"
NOTES_DOC = ROOT / "docs" / "private-algorithm-notes.json"

# ---------------------------------------------------------------------------
# Domain-algorithm references.
#
# The rough-jewellery extraction produced authored algorithm notes carrying real
# numbers. A note is attached to an L3 only where that L3 genuinely owns the same
# kind of work - an architecture, an option pipeline, a parameter graph, a
# persistence boundary, a kernel call map, or the actual jewellery operation.
# No note is attached just because it exists.
#
# `steps` cites the note's own ordered step numbers so a sketch can point at the
# exact behaviour without restating it. Every L3 id here was verified against
# build/matrix-ref-inventory.json.
# ---------------------------------------------------------------------------
DOMAIN_NOTE_MAP = {
    # ---- OM9-M00- ----
    "OM9-M00-L3-01-03": {
        "note": "algo.option.data-object",
        "steps": [1, 2, 3, 4],
        "why": "Mẫu đối tượng tuỳ chọn là ví dụ cụ thể nhất về một abstraction đáng REUSE/ADAPT: tên, giá trị mặc định riêng, khoá lưu, cờ dirty, hai hệ lưu trữ.",
    },
    "OM9-M00-L3-02-01": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 2, 3],
        "why": "Bản đồ thao tác kernel là danh sách việc lớp FFI phải bao phủ, kèm thao tác nào thuộc OCCT và thao tác nào chỉ là số học thuần.",
    },
    "OM9-M00-L3-02-03": {
        "note": "algo.gem.block.data-model",
        "steps": [5, 6],
        "why": "Hai phát hiện số học phải vào ràng buộc nền: dung sai duy nhất thuộc document, và bảng tỉ trọng KHÔNG có trong mã nên OM9 phải tự chọn nguồn.",
    },
    "OM9-M00-L3-03-01": {
        "note": "algo.builder.contract",
        "steps": [1, 2, 3, 4],
        "why": "Hợp đồng builder là kiến trúc nguồn phải tái tạo: ba hook conduit, một hàm lấy tham số, một hàm chốt, user-data riêng.",
    },
    "OM9-M00-L3-05-01": {
        "note": "algo.csblock.channel-model",
        "steps": [1, 2, 3],
        "why": "Mô hình nút theo cổng định vị là quy ước đặt tên và kiến trúc tham chiếu cần kiểm.",
    },
    "OM9-M00-L3-05-02": {
        "note": "algo.sort.gems.and.graph",
        "steps": [1, 4, 5],
        "why": "Mẫu này cho thấy vì sao KHÔNG thể dựa vào thứ tự kết quả: đầu ra gom vào túi nên thứ tự không bảo đảm, và lịch đánh giá không nằm trong cây.",
    },
    # ---- OM9-L3-0 ----
    "OM9-L3-01-01-008": {
        "note": "algo.builder.contract",
        "steps": [1, 2, 3, 4, 5],
        "why": "Panel builder chính là chỗ neo của hợp đồng builder: chọn theo bước, preview, lấy tham số, chốt.",
    },
    "OM9-L3-01-02-004": {
        "note": "algo.builder.contract",
        "steps": [1, 2, 3, 4, 6],
        "why": "Máy trạng thái lệnh tương tác tương ứng vòng khởi tạo, chọn, tính lại, chốt, kèm đường huỷ không đổi document.",
    },
    "OM9-L3-01-02-005": {
        "note": "algo.option.data-object",
        "steps": [1, 2, 3, 5],
        "why": "Mẫu option cho biết một tuỳ chọn phải mang gì: tên, khoá lưu, mặc định riêng, cờ dùng được ở command line, và nhãn hai trạng thái.",
    },
    "OM9-L3-01-03-008": {
        "note": "algo.csblock.channel-model",
        "steps": [1, 2, 3, 4],
        "why": "Đồ thị block với cổng định vị và giá trị mồi là tiền lệ trực tiếp cho đồ thị định nghĩa/instance trong document.",
    },
    "OM9-L3-01-05-007": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 4],
        "why": "Ánh xạ thao tác kernel cho biết thao tác giao thuộc OCCT, và thao tác nào chỉ là số học thuần nên không được gọi xuống kernel.",
    },
    "OM9-L3-01-08-003": {
        "note": "algo.prong.placer",
        "steps": [1, 9],
        "why": "Bằng chứng rằng dung sai document là thứ duy nhất được truyền xuống hình học, còn dung sai 0.01/0.001 là hằng số hard-code phải thay.",
    },
    "OM9-L3-01-08-004": {
        "note": "algo.option.data-object",
        "steps": [1, 4, 5],
        "why": "Giá trị mặc định được chụp riêng khỏi giá trị hiện tại, để nút Reset và việc lưu không lẫn nhau.",
    },
    "OM9-L3-01-08-005": {
        "note": "algo.builder.single-parameter",
        "steps": [1, 2, 3],
        "why": "Builder một tham số cho thấy kẹp ràng buộc thật nằm ở bước tính lại (chiều cao không âm) chứ không ở tầng UI.",
    },
    "OM9-L3-01-08-006": {
        "note": "algo.option.data-object",
        "steps": [4, 5],
        "why": "Tuỳ chọn có hai dạng ghi và hai dạng đọc, nghĩa là việc lưu tham số phải định nghĩa đơn vị và khoá rõ ràng trước khi có hai hệ lưu trữ.",
    },
    "OM9-L3-01-12-002": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2, 3],
        "why": "Mô hình dữ liệu đá cho thấy round-trip chỉ có hai đường: dữ liệu đối tượng theo khoá, và tên instance. Không có hàm ghi XML trong lớp mô tả.",
    },
    "OM9-L3-01-12-004": {
        "note": "algo.builder.contract",
        "steps": [5, 7],
        "why": "Builder tự lưu và đọc trạng thái riêng để lần sau tái dựng được tham số từ đối tượng đã tạo - đây chính là hình dạng của một history stub.",
    },
    "OM9-L3-03-07-001": {
        "note": "algo.option.data-object",
        "steps": [1, 2],
        "why": "Mẫu tuỳ chọn có tên và nhãn hai trạng thái là hợp đồng cho việc hiển thị và đặt tên lệnh trên sidebar.",
    },
    "OM9-L3-03-10-001": {
        "note": "algo.option.data-object",
        "steps": [2, 3],
        "why": "Đăng ký command line trả chỉ số và đọc kết quả từ đối tượng tuỳ chọn, không đọc từ UI.",
    },
    "OM9-L3-03-10-002": {
        "note": "algo.builder.contract",
        "steps": [2, 3, 6],
        "why": "Chuỗi chọn theo bước và preview trong lúc chọn là tiền lệ cho prompt tương tác.",
    },
    "OM9-L3-03-10-003": {
        "note": "algo.option.data-object",
        "steps": [1, 2, 5],
        "why": "Một tuỳ chọn vừa lên command line vừa lên UI, có nhãn cho từng trạng thái - đúng hợp đồng của token bấm được.",
    },
    "OM9-L3-03-14-005": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 3, 4],
        "why": "Chuỗi mô tả có ba dạng theo họ hình dạng - overlay phải dùng đúng dạng, không tự phát minh định dạng.",
    },
    "OM9-L3-03-14-007": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2, 4],
        "why": "Bề mặt thuộc tính của đối tượng (cỡ, hình dạng, profile, pháp tuyến, cờ gương) là danh sách trường mà property editor phải đọc được.",
    },
    "OM9-L3-01-07-003": {
        "note": "algo.rail.profile.body",
        "steps": [9],
        "why": "Rail revolve scale từng vòng rail theo bán kính profile - tiền lệ cho việc truy cập và biến đổi subobject face/edge/vertex.",
    },
    "OM9-L3-01-07-007": {
        "note": "algo.builder.single-parameter",
        "steps": [1, 6],
        "why": "Handle của builder một tham số cho thấy mỗi tham số là một cặp điểm neo, và đổi handle thì quay lại bước tính tham số.",
    },
    "OM9-L3-02-02-001": {
        "note": "algo.gem.on.curve.family",
        "steps": [1, 2],
        "why": "Phép lấy mẫu theo độ phân giải cố định và việc đi trên mảng chỉ số là ràng buộc cho việc ánh xạ toạ độ trong ngữ cảnh mặt phẳng dựng.",
    },
    "OM9-L3-02-03-004": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 4],
        "why": "Snap trên surface và polysurface cần đúng thao tác kernel: closest point và giao thuộc OCCT, không được tự viết lại.",
    },
    "OM9-L3-03-03-004": {
        "note": "algo.builder.single-parameter",
        "steps": [1, 2],
        "why": "Kéo trục gumball tương ứng bước chốt delta vào giá trị thực rồi reset delta - đúng mẫu PostUpdate của builder một tham số.",
    },
    "OM9-L3-03-03-006": {
        "note": "algo.option.data-object",
        "steps": [1, 3],
        "why": "Sửa số bằng gumball dùng lại mẫu tuỳ chọn có tên, có mặc định riêng và bật cờ dirty khi đổi giá trị.",
    },
    "OM9-L3-03-09-001": {
        "note": "algo.sort.gems.and.graph",
        "steps": [1, 4],
        "why": "F6 phân nhánh theo lớp đối tượng giống cách một block nhận dữ liệu từ cổng: phân loại trước, rồi mới chọn hành vi.",
    },
    "OM9-L3-03-09-006": {
        "note": "algo.csblock.channel-model",
        "steps": [1, 2],
        "why": "Điền trước đầu vào cho builder tương ứng việc nối cổng: một cổng hoặc chưa gán (giữ mặc định) hoặc trỏ vào nguồn cụ thể.",
    },
    "OM9-L3-03-09-008": {
        "note": "algo.builder.contract",
        "steps": [1, 2],
        "why": "Hành động bị vô hiệu tương ứng trạng thái chưa khởi tạo của builder: không có đầu vào hợp lệ thì không vào được vòng chọn.",
    },
    "OM9-L3-01-09-003": {
        "note": "algo.rail.profile.body",
        "steps": [9],
        "why": "Rail revolve là phép dựng mặt bằng cách scale từng vòng rail theo bán kính profile - khác hẳn phép quay cứng, và có ba biến thể khác nhau về mốc 1:1.",
    },
    "OM9-L3-01-09-007": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 2, 3],
        "why": "Sweep một và hai rail nằm trong nhóm thao tác kernel; kèm yêu cầu kiểm sạch sau khi dựng.",
    },
    "OM9-L3-02-05-003": {
        "note": "algo.rail.profile.body",
        "steps": [9, 10],
        "why": "Nhóm workflow revolve/rail-revolve/extrude: rail revolve không phải một lệnh revolve thường.",
    },
    "OM9-L3-01-10-001": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "Sau boolean phải kiểm naked edge, clean brep và sửa hướng solid - boolean không tự bảo đảm khối kín.",
    },
    "OM9-L3-01-10-002": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3, 4],
        "why": "Phép trừ cũng nằm trong nhóm cần kiểm sạch và cắt/giao lại sau khi áp.",
    },
    "OM9-L3-01-10-003": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3, 4],
        "why": "Phép giao dùng chung nhóm thao tác cắt/giao và cùng yêu cầu kiểm sạch.",
    },
    "OM9-L3-01-10-004": {
        "note": "algo.geometry.kernel-calls",
        "steps": [4],
        "why": "Cắt curve theo surface và giao surface-surface là thao tác kernel, không phải số học thuần.",
    },
    "OM9-L3-01-10-005": {
        "note": "algo.cluster.halo.bezel",
        "steps": [3],
        "why": "Việc đậy lỗ phẳng và cắt theo profile trong dựng kênh cho thấy mẫu cap/trim phải kiểm giao được trước khi cắt, nếu không thì báo lỗi.",
    },
    "OM9-L3-01-10-007": {
        "note": "algo.geometry.kernel-calls",
        "steps": [4],
        "why": "Offset và trim mặt dùng nhóm thao tác kernel; offset có cả biến thể mới và cũ trong tiền lệ.",
    },
    "OM9-L3-01-10-009": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "Naked edge là phép kiểm sức khoẻ hình học trung tâm: dựng xong phải kiểm, không được coi là kín nếu còn cạnh hở.",
    },
    "OM9-L3-01-11-004": {
        "note": "algo.gem.on.curve.family",
        "steps": [6],
        "why": "Pattern theo cực và theo đường thẳng cần cùng cách chia khoảng dư: tiền lệ có HAI mẫu số khác nhau cho cùng thao tác chia lại.",
    },
    "OM9-L3-01-11-005": {
        "note": "algo.gem.on.curve.family",
        "steps": [5, 6],
        "why": "Frame đặt theo mẫu curve: đi trên mảng chỉ số, rồi hiệu chỉnh khoảng dư ở đoạn cuối.",
    },
    "OM9-L3-01-11-006": {
        "note": "algo.gem.on.curve.family",
        "steps": [4],
        "why": "Frame theo UV surface cần đúng thứ tự transform: dịch offset trước, dựng frame sau, rồi mới nhân nghiêng/xoay/scale.",
    },
    "OM9-L3-01-11-007": {
        "note": "algo.gem.on.curve.family",
        "steps": [5],
        "why": "Bộ giải định hướng hai/ba điểm chính là bước dựng frame ba điểm trong tiền lệ đặt đá.",
    },
    "OM9-L3-01-11-008": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 6],
        "why": "Chính sách bake hay instance: tiền lệ dùng instance kèm dữ liệu đối tượng, và bỏ qua khối instance khi tính thể tích là hành vi phải quyết định.",
    },
    "OM9-L3-01-13-002": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 4],
        "why": "Hợp đồng cầu đọc openNURBS phải nằm trong bản đồ thao tác kernel, không gọi thẳng tuỳ tiện.",
    },
    "OM9-L3-01-13-006": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "NURBS surface và B-rep có trim: sau khi dựng phải kiểm sức khoẻ hình học.",
    },
    "OM9-L3-02-04-005": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Nối, nổ, đóng và cắt curve dùng chung nhóm tiện ích curve với trim và merge, trong đó dung sai merge là hằng số hard-code 0.01.",
    },
    "OM9-L3-02-04-006": {
        "note": "algo.geometry.kernel-calls",
        "steps": [4],
        "why": "Offset, fillet và chamfer curve là thao tác kernel theo bản đồ.",
    },
    "OM9-L3-02-04-007": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Blend của tiền lệ chỉ đạt liên tục bậc MỘT (G1): không có vector cong nào được tính, nên không được quảng cáo là G2.",
    },
    "OM9-L3-02-04-011": {
        "note": "algo.gem.on.curve.family",
        "steps": [2, 3],
        "why": "Đường may và hướng curve là thứ tiền lệ xử lý riêng: có ngưỡng phát hiện vượt đường may và có đảo hướng khi cần.",
    },
    "OM9-L3-02-04-012": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Sketch trên hai view dùng cách dịch profile hai bên rồi dựng mặt ruled và giao với mặt đích - con số dịch chỉ để đủ cao, không phải tham số hình học.",
    },
    "OM9-L3-02-05-001": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 2],
        "why": "Workflow sweep một/hai rail là bước dựng mặt theo bản đồ kernel.",
    },
    "OM9-L3-02-05-002": {
        "note": "algo.geometry.kernel-calls",
        "steps": [2, 3],
        "why": "Loft và curve network dùng nhóm dựng mặt, sau đó phải kiểm sạch.",
    },
    "OM9-L3-02-05-005": {
        "note": "algo.geometry.kernel-calls",
        "steps": [4],
        "why": "Trim, split và untrim surface là nhóm cắt/giao trong bản đồ thao tác.",
    },
    "OM9-L3-02-05-007": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Match, blend và join surface phải khai đúng bậc liên tục đạt được; tiền lệ chỉ đạt G1.",
    },
    "OM9-L3-02-05-008": {
        "note": "algo.geometry.kernel-calls",
        "steps": [4],
        "why": "Offset, fillet và chamfer surface thuộc nhóm kernel.",
    },
    "OM9-L3-02-05-010": {
        "note": "algo.gem.on.curve.family",
        "steps": [4],
        "why": "Pháp tuyến surface là dữ liệu frame; đổi pháp tuyến thì phải tính lại frame, không dùng lại cache.",
    },
    "OM9-L3-01-04-003": {
        "note": "algo.sort.gems.and.graph",
        "steps": [4, 5],
        "why": "Lan truyền dirty và thứ tự đánh giá: tiền lệ cho thấy LỊCH ĐÁNH GIÁ KHÔNG nằm trong cây mã, nên OM9 phải tự định nghĩa thứ tự tô-pô của mình.",
    },
    "OM9-L3-01-04-004": {
        "note": "algo.csblock.channel-model",
        "steps": [2, 3],
        "why": "Liên kết tham chiếu ổn định tương ứng việc một cổng trỏ vào (nguồn, chỉ số ra) chứ không sao chép giá trị.",
    },
    "OM9-L3-01-04-005": {
        "note": "algo.prong.placer",
        "steps": [1],
        "why": "Tính lại tăng dần có cache: tiền lệ dùng cache theo khoá tham số với dung sai so khớp 0.001, và xoá cache đầu mỗi lần tính.",
    },
    "OM9-L3-01-04-006": {
        "note": "algo.builder.contract",
        "steps": [3, 5],
        "why": "Preview so với đã chốt: preview nằm trong bộ nhớ và document không đổi cho tới bước chốt.",
    },
    "OM9-L3-01-04-007": {
        "note": "algo.builder.contract",
        "steps": [6],
        "why": "Rollback khi tính lại thất bại tương ứng đường huỷ của builder: dọn state và document không đổi.",
    },
    "OM9-L3-03-11-004": {
        "note": "algo.builder.contract",
        "steps": [2, 3, 4],
        "why": "Vòng đời preview trực tiếp của builder: chọn, tính lại, vẽ lại, chưa tạo đối tượng.",
    },
    "OM9-L3-03-11-005": {
        "note": "algo.builder.single-parameter",
        "steps": [6],
        "why": "Handle điều khiển trong viewport: mỗi tham số là một cặp điểm neo, đổi handle thì tính lại.",
    },
    "OM9-L3-03-11-006": {
        "note": "algo.option.data-object",
        "steps": [1, 3],
        "why": "Đồng bộ hai chiều giữa handle và panel dùng đúng mẫu tuỳ chọn: cùng một đối tượng mang giá trị, và đổi giá trị bật cờ dirty.",
    },
    "OM9-L3-03-11-008": {
        "note": "algo.option.data-object",
        "steps": [4, 5],
        "why": "Accept, Cancel, Reset và Undo: mặc định được chụp riêng nên Reset quay về được, và lưu chỉ xảy ra khi chốt.",
    },
    "OM9-L3-03-12-001": {
        "note": "algo.option.data-object",
        "steps": [4, 5],
        "why": "Schema snapshot tham số style là danh sách các đối tượng tuỳ chọn kèm khoá lưu của chúng.",
    },
    "OM9-L3-03-13-005": {
        "note": "algo.builder.contract",
        "steps": [7],
        "why": "Sửa lại builder cũ qua history dựa vào việc builder tự đọc được trạng thái đã lưu.",
    },
    "OM9-L3-04-02-001": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Cỡ vành từ đường kính rail theo công thức hệ US, bị lặp ở bốn chỗ trong tiền lệ - OM9 phải tập trung nó vào một chỗ và ghi rõ nguồn chuẩn.",
    },
    "OM9-L3-04-02-005": {
        "note": "algo.rail.profile.body",
        "steps": [1, 2, 3, 7, 8],
        "why": "Công cụ bốn rail: giao rail với mặt phẳng profile, đặt gốc tại trung bình bốn điểm giao, rồi sắp rail theo góc trong mặt phẳng đó (chú ý trục X bị đảo dấu). Pha sweep thực chất là BỐN lần sweep hai rail, trong đó hai lần dùng cạnh sinh ra từ hai lần trước, rồi join với dung sai 0.001 và yêu cầu đúng một brep.",
    },
    "OM9-L3-04-02-008": {
        "note": "algo.rail.profile.body",
        "steps": [7, 8],
        "why": "Đầu ra sweep phải là vật thể liên kết được: tiền lệ join bốn mảnh và yêu cầu đúng một brep, nếu không thì thất bại.",
    },
    "OM9-L3-04-02-009": {
        "note": "algo.builder.single-parameter",
        "steps": [6],
        "why": "Handle cho rail và profile theo đúng mẫu cặp điểm neo của builder một tham số.",
    },
    "OM9-L3-04-19-001": {
        "note": "algo.gem.block.data-model",
        "steps": [5],
        "why": "Thể tích chỉ tính được trên brep solid, manifold, hợp lệ và thể tích dương; nếu không thì báo lỗi riêng chứ không im lặng trả 0.",
    },
    "OM9-L3-04-19-002": {
        "note": "algo.gem.block.data-model",
        "steps": [5, 6],
        "why": "Sổ đăng ký tỉ trọng vật liệu: tiền lệ KHÔNG có bảng trong mã, tỉ trọng là dữ liệu người dùng sửa được parse từ file vật liệu - OM9 phải tự chọn nguồn.",
    },
    "OM9-L3-04-19-003": {
        "note": "algo.gem.block.data-model",
        "steps": [5],
        "why": "Công thức khối lượng: thể tích nhân 0.001 nhân tỉ trọng, làm tròn hai chữ số; bản pennyweight chia thêm 1.55517.",
    },
    "OM9-L3-04-19-005": {
        "note": "algo.gem.block.data-model",
        "steps": [5],
        "why": "Chính sách khi brep hở: tiền lệ từ chối tính, chọn lại đối tượng vi phạm và báo ba mã lỗi riêng biệt.",
    },
    "OM9-L3-04-09-001": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2, 3],
        "why": "Schema cỡ và hình dạng đá đọc từ khoá dữ liệu đối tượng, sáu điểm min/max, làm tròn theo số chữ số cấu hình; profile là chuỗi điểm riêng.",
    },
    "OM9-L3-04-09-002": {
        "note": "algo.gem.block.data-model",
        "steps": [1],
        "why": "Tài sản mesh và B-rep của đá gắn với cùng mô tả: cỡ suy từ sáu điểm min/max đã biến đổi bởi xform của instance.",
    },
    "OM9-L3-04-09-003": {
        "note": "algo.cluster.halo.bezel",
        "steps": [2],
        "why": "Chọn cỡ đá bằng cách duyệt BẢNG CỠ của định nghĩa và giữ mục có độ trải Y gần đích nhất - không phải nội suy tuỳ ý.",
    },
    "OM9-L3-04-09-004": {
        "note": "algo.gemshape.offset",
        "steps": [1, 2, 3, 4],
        "why": "Hình dạng đá tuỳ biến: kiểm hợp lệ trước, rồi suy đường bao tham chiếu, rồi offset - offset phụ thuộc hình dạng chứ không chỉ khoảng cách.",
    },
    "OM9-L3-04-09-005": {
        "note": "algo.cluster.halo.bezel",
        "steps": [4],
        "why": "Họ emerald và baguette có số curve offset riêng (chín curve cho họ emerald, năm cho họ baguette) và vòng lặp khớp góc riêng.",
    },
    "OM9-L3-04-09-008": {
        "note": "algo.gemshape.offset",
        "steps": [2, 3],
        "why": "Tham chiếu facet, outline và girdle: offset lấy từ đường bao đá và bán kính, có hàm kiểm hình dạng hợp lệ.",
    },
    "OM9-L3-04-10-001": {
        "note": "algo.gem.on.curve.family",
        "steps": [1, 2, 3, 4, 5, 6, 7, 8],
        "why": "Gem On Curve Basic dùng chung cơ chế bốn biến thể: lấy mẫu 1/50, xử lý đường may, dựng frame ba điểm, và trật tự transform cố định.",
    },
    "OM9-L3-04-10-002": {
        "note": "algo.gem.on.curve.family",
        "steps": [1, 2, 3, 4, 5, 6, 7, 8, 9],
        "why": "Gem On Curve Advanced: mặc định start 0.2 end 0.8, đường kính 2.0, spacing 0.2; có máy trạng thái tìm đá và HAI mẫu số khác nhau cho cùng thao tác chia lại.",
    },
    "OM9-L3-04-10-003": {
        "note": "algo.gem.on.curve.family",
        "steps": [1, 2, 3, 4, 5, 6],
        "why": "Gem Count On Curve điều khiển bằng SỐ LƯỢNG (mặc định 5, khoảng 2..15), nội suy theo phân số k chia (count trừ 1). Lưu ý cặp nửa rộng của biến thể này NGƯỢC với Advanced ở chiều tiến - nếu dùng một công thức chung thì sẽ lệch chỗ chèn đá.",
    },
    "OM9-L3-04-10-004": {
        "note": "algo.gem.on.curve.family",
        "steps": [6],
        "why": "Gem List On Curve đặt theo THỨ TỰ danh sách, không chia đều khoảng cách và không gương; mỗi viên dùng cỡ riêng của nó.",
    },
    "OM9-L3-04-10-005": {
        "note": "algo.gem.on.curve.multi",
        "steps": [1, 2, 3],
        "why": "Gem On Curve Multi thêm phân bố nhiều hàng, pattern, và split-between-stones; hard-code Diamond/Round nên không có tuỳ chọn gem block.",
    },
    "OM9-L3-04-10-008": {
        "note": "algo.gem.on.curve.family",
        "steps": [4, 5],
        "why": "Đi theo curve bên dưới: điểm đến là điểm trên curve dịch theo trục y cục bộ của frame, và pháp tuyến được đo LẠI từ điểm đã dịch.",
    },
    "OM9-L3-04-10-011": {
        "note": "algo.gem.on.curve.family",
        "steps": [6],
        "why": "Liên kết history giữa đá và curve/surface dựa vào việc lưu chỉ số, đường kính và offset trên từng viên để dựng lại transform mà không cần đoán.",
    },
    "OM9-L3-04-11-001": {
        "note": "algo.cluster.halo.bezel",
        "steps": [9],
        "why": "Halo là ĐỒ THỊ BA BLOCK: cluster đá, một dải kim loại, và bộ đặt chấu; chỉ lấy phần tử đầu tiên của dải kim loại.",
    },
    "OM9-L3-04-11-002": {
        "note": "algo.cluster.halo.bezel",
        "steps": [1, 2, 3, 4, 5],
        "why": "Cluster: chọn cỡ melee theo bề rộng lòng kênh, dựng rail bằng offset đường bao viên giữa, chia theo họ hình dạng, và KHÔNG có phép kiểm chồng lấn.",
    },
    "OM9-L3-04-11-003": {
        "note": "algo.cluster.halo.bezel",
        "steps": [7, 8],
        "why": "Bố trí channel baguette dùng chính mô hình kênh: profile được chuẩn hoá vào hộp đơn vị rồi scale theo bề rộng, sàn nằm dưới đỉnh đúng GemSizeZ.",
    },
    "OM9-L3-04-11-007": {
        "note": "algo.cluster.halo.bezel",
        "steps": [1, 9],
        "why": "Sửa tham số halo/cluster: bề rộng kênh và dày tường là handle DÙNG CHUNG, nên đổi bề rộng kênh là đổi số viên đá.",
    },
    "OM9-L3-04-12-001": {
        "note": "algo.prong.placer",
        "steps": [1, 2, 3],
        "why": "Auto Pave nhận ý tưởng từ bộ đặt chấu: vị trí đá đến từ thượng nguồn, còn bộ đặt chấu chỉ dựng chấu - đừng đi tìm thuật toán rải đá trong đó.",
    },
    "OM9-L3-04-12-004": {
        "note": "algo.prong.placer",
        "steps": [3, 4, 5, 7, 8, 10],
        "why": "Bố trí chấu pave: mỗi cặp đá liền kề dựng hai đường tròn biên, giải đường tròn tiếp xúc bộ ba, rồi chỉ phát chấu khi đường kính nằm trong khoảng min/max.",
    },
    "OM9-L3-04-12-006": {
        "note": "algo.prong.placer",
        "steps": [6],
        "why": "Lò xo và giãn đá: tiền lệ KHÔNG có bước giải lặp nào - khoảng cách được bảo đảm bằng cấu trúc tiếp xúc, không bằng tối ưu.",
    },
    "OM9-L3-04-12-008": {
        "note": "algo.sort.gems.and.graph",
        "steps": [4],
        "why": "Làm mới hình học theo history: chuỗi đá được sắp lại SAU MỖI viên thêm vào, nên thứ tự là một phần của kết quả chứ không phải chi tiết phụ.",
    },
    "OM9-L3-04-13-002": {
        "note": "algo.option.data-object",
        "steps": [1, 3],
        "why": "Sửa cỡ và điều khiển đá dùng lại mẫu tuỳ chọn: giá trị có tên, có mặc định riêng, và đổi thì bật cờ dirty.",
    },
    "OM9-L3-04-13-008": {
        "note": "algo.gem.block.data-model",
        "steps": [5],
        "why": "Báo cáo số lượng và khối lượng đá: khối lượng đá tròn dùng công thức riêng theo đường kính; hình dạng khác dùng công thức có hệ số F (thân hàm chưa đọc).",
    },
    "OM9-L3-04-13-009": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 3, 4],
        "why": "Giữ ID và thuộc tính qua history: định danh đá nằm trong tên instance và khoá dữ liệu đối tượng, nên đổi tên là mất định danh.",
    },
    "OM9-L3-04-14-001": {
        "note": "algo.cluster.halo.bezel",
        "steps": [2, 4, 5],
        "why": "Head builder dạng basket: dựng rail bằng offset đường bao viên giữa, rồi chia theo họ hình dạng và đặt chấu quanh viên - đúng cơ chế cluster đã trích.",
    },
    "OM9-L3-04-14-002": {
        "note": "algo.cluster.halo.bezel",
        "steps": [3, 5],
        "why": "Chế độ half bezel: rail và góc đặt chấu lấy từ cùng công thức offset phụ thuộc bề rộng lòng kênh có bằng 0 hay không.",
    },
    "OM9-L3-04-14-003": {
        "note": "algo.prong.placer",
        "steps": [3, 4, 5],
        "why": "Sửa mức rail chấu: mỗi cặp đá liền kề dựng hai đường tròn biên rồi giải đường tròn tiếp xúc, nên đổi mức rail là đổi đường kính chấu chứ không phải dịch chấu.",
    },
    "OM9-L3-04-14-004": {
        "note": "algo.cluster.halo.bezel",
        "steps": [10, 11],
        "why": "Bezel đầy đủ: bốn rail offset từ đường bao đá, thành ngoài dựng bằng cách quay và scale profile theo chiều cao, join phải cho đúng một brep.",
    },
    "OM9-L3-04-14-005": {
        "note": "algo.cluster.halo.bezel",
        "steps": [7, 8],
        "why": "Under bezel và gallery dùng mô hình kênh: profile chuẩn hoá vào hộp đơn vị rồi scale theo bề rộng, sàn nằm dưới đỉnh đúng GemSizeZ.",
    },
    "OM9-L3-04-14-006": {
        "note": "algo.cluster.halo.bezel",
        "steps": [5],
        "why": "Scallop bezel: góc và số viên theo họ hình dạng, có công thức nhận biết góc riêng cho style 1.",
    },
    "OM9-L3-04-14-007": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2, 3],
        "why": "Duyệt profile head/bezel: mô tả đá và profile đọc từ khoá dữ liệu đối tượng, profile là chuỗi điểm cần ít nhất bảy điểm cho các độ dài góc/đáy.",
    },
    "OM9-L3-04-14-008": {
        "note": "algo.cluster.halo.bezel",
        "steps": [4],
        "why": "Gắn seat ở girdle: rail girdle/seat dựng bằng offset theo GirdleSpacing tại cao độ seat, và seat chỉ dựng khi SeatLength lớn hơn 0.",
    },
    "OM9-L3-04-14-009": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "Kiểm hợp lệ và cập nhật hình học head/bezel: sau khi dựng phải kiểm naked edge, clean brep và sửa hướng solid.",
    },
    "OM9-L3-04-15-001": {
        "note": "algo.prong.placer",
        "steps": [2, 3, 5, 7, 10],
        "why": "Prong builder theo đường đá: nhận chuỗi đá đã xếp thứ tự, mỗi cặp dựng hai đường tròn biên, giải đường tròn tiếp xúc bộ ba, chỉ phát khi đường kính trong khoảng min/max.",
    },
    "OM9-L3-04-15-002": {
        "note": "algo.prong.placer",
        "steps": [7, 8, 9],
        "why": "Sửa chấu đã chọn: đường may của chấu được canh theo hướng tiếp xúc cộng một phần tư vòng, và đặt dọc theo trục trung bình của hai viên.",
    },
    "OM9-L3-04-15-003": {
        "note": "algo.geometry.kernel-calls",
        "steps": [1, 4],
        "why": "Chấu trên surface cần closest point và giao thuộc OCCT; không được tự viết lại phép chiếu.",
    },
    "OM9-L3-04-15-004": {
        "note": "algo.cluster.halo.bezel",
        "steps": [8],
        "why": "Mảnh kim loại từ đá: dải kim loại là mặt mà các cột điểm điều khiển bám theo offset đường bao viên giữa, không phải vòng đá.",
    },
    "OM9-L3-04-15-005": {
        "note": "algo.gemshape.offset",
        "steps": [1, 2, 3],
        "why": "Đường biên kênh suy từ đường bao đá qua offset, và offset phụ thuộc hình dạng đá chứ không chỉ khoảng cách.",
    },
    "OM9-L3-04-15-006": {
        "note": "algo.gemshape.offset",
        "steps": [2, 3],
        "why": "Bead và milgrain trên curve: bán kính hạt suy từ đường bao tham chiếu rồi offset dọc curve.",
    },
    "OM9-L3-04-15-007": {
        "note": "algo.cluster.halo.bezel",
        "steps": [6],
        "why": "Bead trên surface: định hướng theo pháp tuyến surface, và phép đo lại pháp tuyến từ điểm đã dịch là bắt buộc khi có offset.",
    },
    "OM9-L3-04-15-008": {
        "note": "algo.rail.profile.body",
        "steps": [9],
        "why": "Kéo đối tượng về rail: rail revolve cho thấy rail cho đường đi còn profile cho bán kính, nên kéo về rail là đổi tham số bán kính chứ không phải quay cứng.",
    },
    "OM9-L3-04-15-009": {
        "note": "algo.gem.on.curve.family",
        "steps": [2, 3, 6],
        "why": "Chế độ đầu hàng và khoảng cách chấu: xử lý đường may, hướng duyệt, và khoảng dư được chia lại bằng một trong hai mẫu số khác nhau.",
    },
    "OM9-L3-04-15-010": {
        "note": "algo.gem.block.data-model",
        "steps": [5],
        "why": "Khối lượng của setting dùng cùng công thức: thể tích nhân 0.001 nhân tỉ trọng, làm tròn hai chữ số; brep hở bị từ chối chứ không trả 0.",
    },
    "OM9-L3-01-12-008": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2],
        "why": "Codec STL tessellation: hình học đi ra từ cùng mô hình dữ liệu đá, nên đơn vị và hướng mặt phải khớp với mô tả chứ không tự đặt.",
    },
    "OM9-L3-04-16-001": {
        "note": "algo.cutter.is-library-data",
        "steps": [1, 2, 3, 4, 5],
        "why": "Cutter azure cho một viên: KHÔNG có thuật toán cutter trong mã managed ở tiền lệ - cutter là dữ liệu thư viện, định vị bằng hàm chung, rồi trừ bằng boolean của nhân hình học và kiểm sạch.",
    },
    "OM9-L3-04-16-002": {
        "note": "algo.cutter.is-library-data",
        "steps": [1, 3, 4],
        "why": "Cutter nhiều viên: cùng đường đi dữ liệu-thư-viện rồi boolean, chỉ khác số đối tượng bị trừ và việc kiểm sạch phải chạy trên kết quả gộp.",
    },
    "OM9-L3-04-16-003": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2],
        "why": "Thư viện profile cutter là DỮ LIỆU, không phải mã: định dạng và trường của cutter do OM9 tự định nghĩa, không chép được từ tiền lệ.",
    },
    "OM9-L3-04-16-005": {
        "note": "algo.gem.block.data-model",
        "steps": [1, 2],
        "why": "Kiểm soát độ sâu và khe hở pavilion cần cỡ đá có dấu: mô tả đá đọc min/max theo Z và làm tròn, nên cao độ pavilion suy được từ đó.",
    },
    "OM9-L3-04-16-006": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3, 4],
        "why": "Boolean tương tác trên kim loại: sau phép boolean phải kiểm naked edge, clean brep và sửa hướng solid - boolean không tự bảo đảm khối kín.",
    },
    "OM9-L3-04-16-007": {
        "note": "algo.cutter.is-library-data",
        "steps": [4],
        "why": "Tuỳ chọn giữ hay bỏ thân cutter: quyết định này thuộc tầng ứng dụng, không phải thuật toán - tiền lệ không có mã cho nó.",
    },
    "OM9-L3-04-17-001": {
        "note": "algo.cluster.halo.bezel",
        "steps": [7, 8],
        "why": "Channel builder cutter dùng mô hình kênh: profile chuẩn hoá vào hộp đơn vị, cắt hai tường, suy chiều cao mái rồi đặt sàn dưới đỉnh đúng GemSizeZ.",
    },
    "OM9-L3-04-17-002": {
        "note": "algo.cutter.is-library-data",
        "steps": [1, 2, 3],
        "why": "Microprong cutter: cùng đường đi dữ liệu-thư-viện; không có thuật toán microprong trong mã managed ở tiền lệ.",
    },
    "OM9-L3-04-17-003": {
        "note": "algo.cluster.halo.bezel",
        "steps": [7, 8],
        "why": "Bright cut channel builder dùng lại mô hình kênh với bo góc: sàn được hạ bằng hai curve blend khi độ sâu lớn hơn hai lần bán kính bo.",
    },
    "OM9-L3-04-17-004": {
        "note": "algo.cutter.is-library-data",
        "steps": [1, 3, 4],
        "why": "Bright cutter một viên: dữ liệu thư viện cộng boolean; phần duy nhất thuộc OM9 là kiểm sạch sau khi trừ.",
    },
    "OM9-L3-04-17-005": {
        "note": "algo.cluster.halo.bezel",
        "steps": [7],
        "why": "Profile và điều khiển đầu của channel cutter: chuẩn hoá profile vào hộp đơn vị rồi scale theo bề rộng, nên kích thước vẽ của profile không quan trọng.",
    },
    "OM9-L3-04-17-006": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3, 4],
        "why": "Boolean tương tác nhiều viên: giao và trừ thuộc OCCT, và kết quả phải qua phép kiểm sức khoẻ hình học.",
    },
    "OM9-L3-04-18-001": {
        "note": "algo.rail.profile.body",
        "steps": [11],
        "why": "Cắt theo finger rail: cỡ vành suy từ đường kính rail theo công thức hệ US, và rail được tìm theo tên đối tượng khớp mẫu Ring Rail.",
    },
    "OM9-L3-04-18-002": {
        "note": "algo.cutter.is-library-data",
        "steps": [1, 2, 3],
        "why": "Cutter mặt phẳng và khối: dụng cụ cắt ở đây là hình học sơ cấp chứ không phải profile thư viện, nhưng phép áp vẫn là boolean của nhân hình học.",
    },
    "OM9-L3-04-18-003": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "Routine lật quad: đổi hướng mặt phải kèm phép sửa hướng solid, nếu không thì khối mất tính kín.",
    },
    "OM9-L3-04-18-004": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3, 4],
        "why": "Boolean builder: hợp, trừ, giao đều là thao tác kernel và cùng chịu phép kiểm sạch sau đó.",
    },
    "OM9-L3-04-18-005": {
        "note": "algo.builder.contract",
        "steps": [3, 5],
        "why": "History của mặt phẳng cắt: mặt phẳng cắt là một tham số của builder, nên preview nằm trong bộ nhớ và document chỉ đổi ở bước chốt.",
    },
    "OM9-L3-04-18-006": {
        "note": "algo.geometry.kernel-calls",
        "steps": [3],
        "why": "Kiểm kín nước cho cutter: naked edge là phép kiểm trung tâm, và dựng xong phải kiểm trước khi coi là thành công.",
    },
}


RESULT_RE = re.compile(r"->\s*(Result\s*<[^>]*>|[A-Za-z_][A-Za-z0-9_:<>]*)")
ERROR_RE = re.compile(r"Result\s*<[^,]*,\s*([A-Za-z_][A-Za-z0-9_:]*)>")


def load_domain_notes() -> dict:
    """Load the authored algorithm notes, keyed by note id."""
    if not NOTES_DOC.is_file():
        return {}
    try:
        payload = json.loads(NOTES_DOC.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    return {n["id"]: n for n in payload.get("notes", [])}


DOMAIN_NOTES = load_domain_notes()


def domain_reference(l3_id: str) -> dict | None:
    """The domain-algorithm reference for an L3, resolved against the notes document.

    Returns None when the L3 is not mapped, so every other milestone is unaffected.
    """
    mapping = DOMAIN_NOTE_MAP.get(l3_id)
    if not mapping:
        return None
    note = DOMAIN_NOTES.get(mapping["note"])
    if note is None:
        return None
    steps = note["algorithm"]
    cited = [s for s in mapping["steps"] if 1 <= s <= len(steps)]
    if not cited:
        return None
    return {
        "note_id": note["id"],
        "stage": note["stage"],
        "why_for_this_task": mapping["why"],
        "om9_backend": note["om9_backend"],
        "steps_cited": cited,
        "algorithm_steps": [steps[i - 1] for i in cited],
        "source_file": note["source"]["file"],
        "source_methods": note["source"]["methods"],
        "boundaries_carried": note["boundaries"],
        "note_status": ("authored description derived from reading the private Matrix9 source; "
                        "not runtime evidence, and every constant must be re-measured"),
    }


def task_role(task):
    """The task's role label, whichever key this milestone's records use."""
    return (task.get("role") or task.get("task_role") or "").strip()


def template_role(role):
    """Map a milestone-specific role label onto a template, or None."""
    return ROLE_ALIASES.get(role)


# M00 is a feasibility/architecture milestone: its records carry no role field, so the
# only honest option is to infer one from the record's own title and to say so. These
# patterns are matched against the title text.
#
# Only two templates make sense for a feasibility record: an inventory (SOURCE_AUDIT)
# and a constraint test (CONTRACT_TEST). A decision, licence check or release-notice
# record is a constraint to verify, not an acceptance matrix over measured geometry,
# so it maps to CONTRACT_TEST rather than ACCEPTANCE.
TITLE_ROLE_RULES = (
    ("CONTRACT_TEST", ("proof", "verify", "check", "test", "validation", "gate", "baseline",
                       "stress", "invariant", "continuous", "decision", "strategy", "policy",
                       "license", "trademark", "release", "notices", "security", "audit")),
    ("SOURCE_AUDIT", ()),
)


def role_from_title(title):
    """Infer a template role from an M00 record title, with the rule that matched."""
    low = (title or "").lower()
    for role, keys in TITLE_ROLE_RULES:
        for key in keys:
            if key in low:
                return role, f"title contains {key!r}"
    return "SOURCE_AUDIT", "default for a feasibility record (no role field on the record)"


def resolve_role(task, milestone, title):
    """(label, template_role, provenance) for a task.

    Uses the record's own role when it has one, and only for M00 - a milestone whose
    records genuinely have no role field - falls back to the title. The provenance is
    stored on the block so an inferred role is never mistaken for a recorded one.
    """
    label = task_role(task)
    if label:
        return label, template_role(label), "recorded_on_task"
    if milestone == "M00":
        role, why = role_from_title(title)
        return f"INFERRED_FROM_TITLE:{role}", role, f"inferred ({why})"
    return "", None, "absent"


def fn_name(task_id):
    """Stable snake_case entry point name derived from the task id."""
    slug = re.sub(r"[^a-z0-9]+", "_", (task_id or "").lower()).strip("_")
    slug = re.sub(r"^om9_", "", slug)          # avoid om9_om9_... prefixes
    return "om9_" + slug


def error_type(api):
    match = ERROR_RE.search(api or "")
    if match:
        return match.group(1)
    match = RESULT_RE.search(api or "")
    return "Error" if not match else "Error"


def sources_for(task):
    """Reference paths already recorded on the task, used for the source list.

    M01/M02 record each source as a dict with path/purpose; the later milestones
    record plain path strings, so both shapes are accepted.
    """
    paths = []
    for key in ("source_paths", "source_paths_verified_locally", "source_anchors"):
        for entry in (task.get(key) or []):
            if isinstance(entry, str):
                path, purpose = entry, "reference"
            else:
                path = entry.get("path")
                purpose = entry.get("purpose") or "reference"
            if not path:
                continue
            line = f"{path} — {purpose}"
            if line not in paths:
                paths.append(line)
    return paths


def proposed_api(task):
    """The task's proposed API, whichever key this milestone's records use."""
    api = task.get("proposed_api_not_verified")
    if isinstance(api, str) and api.strip():
        return api.strip()
    contract = task.get("proposed_contract")
    if isinstance(contract, dict):
        parts = []
        for key in ("input", "output", "error_type"):
            val = contract.get(key)
            if isinstance(val, str) and val.strip():
                parts.append(f"{key}: {val.strip()}")
        return "; ".join(parts)
    if isinstance(contract, str) and contract.strip():
        return contract.strip()
    return ""


GOVERNANCE_MARKERS = (
    "git rev-parse", "cargo metadata", "agents/claude", "license scope", "reuse",
    "reflect", "blocked", "do not invent", "pass criteria", "evidence", "signoff",
    "review", "audit trail", "commit", "push",
)


def algorithm_text(task):
    """Algorithm notes already on the record, in a stable order.

    M01/M02 carry a single implementation_algorithm string. The later milestones
    instead carry geometry invariants, a note about how the manual was interpreted,
    and ordered work instructions. Both of the latter mix engineering governance
    ("reinspect the PDF", "record evidence", "check the pinned SHA") with real
    behaviour, so items that are *purely* governance are dropped. Items are never
    split: the record's own punctuation is meaningful ("...; never hard-code ..."),
    and splitting it produces broken fragments in the sketch.
    """
    algo = task.get("implementation_algorithm")
    if isinstance(algo, str) and algo.strip():
        return algo.strip()
    chunks = []

    def add(value):
        for item in (value if isinstance(value, list) else [value]):
            text = str(item).strip()
            if not text or text in chunks:
                continue
            if _is_governance_only(text):
                continue
            chunks.append(text)

    add(task.get("geometry_invariants"))
    add(task.get("fixtures_design_note"))
    add(task.get("manual_interpretation_note"))
    add(task.get("work_instructions"))
    # items already end in a period; joining with one would double it
    return re.sub(r"\.{2,}", ".", ". ".join(chunks))


def _is_governance_only(text):
    """True when an item is process instruction with no behaviour content.

    Such an item both starts and ends in governance vocabulary; an item that merely
    mentions evidence while stating geometry rules is kept.
    """
    low = text.lower()
    return any(marker in low for marker in GOVERNANCE_MARKERS) and not any(
        keep in low for keep in
        ("mm", "tolerance", "brep", "geometry", "gem", "stone", "preview", "transaction",
         "undo", "redo", "id", "spacing", "curve", "surface", "offset", "prong", "cut",
         "instance", "transform", "layer", "fixture identifier"))


def oracles(task):
    """(positive, negative) oracle text, from whichever fields exist."""
    positives = task.get("automated_tests") or []
    pass_criteria = task.get("pass_criteria") or []
    errors = task.get("error_cases") or []
    positive = ""
    if isinstance(positives, list) and positives:
        positive = str(positives[0])
    elif isinstance(pass_criteria, list) and pass_criteria:
        positive = str(pass_criteria[0])
    negative = ""
    if isinstance(errors, list) and errors:
        negative = str(errors[0])
    elif isinstance(pass_criteria, list) and len(pass_criteria) > 1:
        negative = str(pass_criteria[1])
    return positive, negative


def negative_case_text(task):
    """The negative case a test must reject.

    Distinct from the negative oracle: `failure_cases` describes the input that must
    be refused, while `error_cases` describes the typed error the refusal must raise
    and is therefore used as the oracle.
    """
    for key in ("negative_case", "failure_cases"):
        value = task.get(key)
        if isinstance(value, str) and value.strip():
            return value.strip()
        if isinstance(value, list) and value:
            return str(value[0])
    return "invalid, empty or degenerate input"


def build_context(record, task, milestone):
    api = proposed_api(task)
    algorithm = algorithm_text(task)
    positive, negative = oracles(task)
    api = api or record.get("proposed_api_not_verified") or ""
    return {
        "task_id": task.get("task_id") or record.get("id"),
        "title": task.get("title") or record.get("title"),
        "role": template_role(task_role(task)),
        "role_label": task_role(task),
        "purpose": task.get("purpose") or task.get("goal") or "implement the capability",
        "api": api,
        "algorithm": algorithm,
        "owner": task.get("owner_module_id") or record.get("owner_l2_id") or "the owning module",
        "positive": positive or task.get("positive_case") or "valid input from the fixture",
        "negative": (task.get("negative_case") or negative_case_text(task)),
        "oracle": task.get("test_oracle") or positive or "measured positive oracle",
        "negative_oracle": (task.get("negative_oracle") or negative
                            or "typed error with unchanged state"),
        "fixture": ((task.get("fixture_ids") or [None])[0]
                    or task.get("fixture_id")
                    or (task.get("outputs") or ["the L3 fixture"])[0]),
        "sources": sources_for(task),
        "fn_name": fn_name(task.get("task_id") or record.get("id")),
        "error_type": error_type(api),
        "milestone": milestone,
    }


def build_block(record, task, milestone):
    title = task.get("title") or record.get("title") or ""
    label, role, provenance = resolve_role(task, milestone, title)
    role = role or "IMPLEMENT"
    template = tpl.ROLE_TEMPLATES.get(role)
    if template is None:
        return None
    ctx = build_context(record, task, milestone)
    ctx["role"] = role
    l3_id = record.get("parent_l3_id") or task.get("parent_l3_id") or ""
    reference = domain_reference(l3_id)
    ctx["domain_steps"] = (reference or {}).get("algorithm_steps", [])
    sketch = template(ctx)
    block = {
        "schema": SCHEMA,
        "policy_id": "om9.implementation_sample.skeleton.v1",
        "status": "author_skeleton_untrusted",
        "language": "rust",
        "role": role,
        "role_label_on_record": label or task_role(task),
        "role_provenance": provenance,
        "owner_boundary": ctx["owner"],
        "goal": ctx["purpose"],
        "algorithm_on_record": ctx["algorithm"],
        "proposed_signature_on_record": ctx["api"],
        "sketch_is_untested": True,
        "not_runtime_evidence": True,
        "must_not_be_reported_as_implemented": True,
        "reference_note": tpl.ROLE_REFERENCE_NOTE.get(role, "read the pinned source before reuse"),
        "sketch": sketch,
    }
    # Only present where this L3 genuinely owns the same work as an authored note.
    if reference:
        block["domain_algorithm_reference"] = reference
    return block


def runbook_block(blocks_summary):
    roles = ", ".join(f"`{role}` x{count}" for role, count in sorted(blocks_summary.items()))
    return "\n".join([
        RUNBOOK_HEADING,
        "",
        "**Trạng thái:** `author_skeleton_untrusted` — code mẫu để bắt đầu, KHÔNG phải "
        "bằng chứng và không được báo cáo là đã triển khai.",
        "",
        f"**Vai trò được sinh mẫu:** {roles}.",
        "",
        "### Quy tắc dùng code mẫu",
        "",
        "- Mẫu chỉ nêu tên những thứ đã có trong chính bản ghi này (API đề xuất / "
        "`implementation_algorithm` / `source_paths`) hoặc thư viện chuẩn Rust — không có symbol bịa.",
        "- `unimplemented!()` là cố ý: mẫu là khung, không chạy được và không chứng minh hành vi nào.",
        "- Code tham chiếu từ QymCAD (revision đã khóa) và từ Matrix9 phải **đọc rồi ADAPT**, "
        "không copy nguyên văn; phạm vi license lấy từ M00.",
        "- Oracle dương/âm trong mẫu lấy từ bản ghi; điều kiện PASS vẫn là command receipt exit 0 "
        "cộng oracle đo được từ lần chạy thật.",
        "",
        "> Không thay placeholder bằng số liệu suy đoán. Không xóa cảnh báo này để biến mẫu "
        "thành bằng chứng.",
        "",
    ])


def inject_markdown(text, block):
    if RUNBOOK_HEADING in text:
        return text
    value = json.loads(text)
    current = value.get("agent_runbook_markdown")
    if not isinstance(current, str) or not current:
        return text
    combined = current.rstrip() + "\n\n" + block
    if not combined.endswith("\n"):
        combined += "\n"
    rendered = json.dumps(combined, ensure_ascii=False)
    marker = '"agent_runbook_markdown": '
    start = text.index(marker) + len(marker)
    end = start
    in_string = False
    escaped = False
    for char in text[start:]:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                break
        elif char == '"':
            in_string = True
        end += 1
    end += 1
    return text[:start] + rendered + text[end:]


def strip_applied(text):
    data = json.loads(text)
    data.pop(SAMPLES_KEY, None)
    data.pop(GUIDE_KEY, None)
    runbook = data.get("agent_runbook_markdown")
    if isinstance(runbook, str) and RUNBOOK_HEADING in runbook:
        data["agent_runbook_markdown"] = runbook.split(RUNBOOK_HEADING)[0].rstrip() + "\n"
    return json.dumps(data, ensure_ascii=False, indent=2) + "\n"


def process(path, milestone, apply, refresh):
    text = path.read_text(encoding="utf-8")
    record = json.loads(text)
    # M01/M02 nest the task under "task"; M04-M07 put the same fields at the record
    # root, so fall back to the record itself rather than building an empty context.
    task = record.get("task") or record
    block = build_block(record, task, milestone)
    if block is None:
        return "skipped_unknown_role", None
    had = SAMPLES_KEY in text
    if had and not refresh:
        return "already_applied", None
    if had:
        text = strip_applied(text)
    updated = refs.inject_object(text, SAMPLES_KEY, block)
    guide = runbook_block({block["role"]: 1})
    if isinstance(record.get("agent_runbook_markdown"), str):
        # M01/M02 already carry a runbook string; append the warning section to it
        updated = inject_markdown(updated, guide)
        where = "runbook"
    else:
        # The overlay records (M04-M07) have no runbook field, so the warning gets
        # its own member rather than being silently dropped.
        updated = refs.inject_object(updated, GUIDE_KEY, guide)
        where = "guide_field"
    json.loads(updated)
    if apply:
        path.write_text(updated, encoding="utf-8")
    return ("refreshed" if had else "applied"), block["role"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--refresh", action="store_true")
    parser.add_argument("--milestone", action="append", default=None)
    args = parser.parse_args()

    counts, roles = {}, {}
    for milestone, dirname in MILESTONES.items():
        if args.milestone and milestone not in args.milestone:
            continue
        base = ROOT / "OM9_Milestones" / dirname
        for path in sorted(base.rglob("l4.json")):
            result, role = process(path, milestone, args.apply, args.refresh)
            counts[result] = counts.get(result, 0) + 1
            if role:
                roles[role] = roles.get(role, 0) + 1
    print(json.dumps({"apply": args.apply, "refresh": args.refresh,
                      "results": counts, "roles": roles}, indent=1))


if __name__ == "__main__":
    main()
