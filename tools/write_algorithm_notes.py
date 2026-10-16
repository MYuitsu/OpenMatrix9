"""Authored algorithm notes for the rough-jewellery build chain.

Each entry is written from reading the Matrix decompile. It records:

  verified_facts   what was literally observed (structure, names, control flow shape,
                   numeric divisors, comparison thresholds) - NOT copied code
  algorithm        the operation decomposition in our own words, as ordered steps
  inputs/outputs   what the step consumes and produces
  om9_backend      which OM9 layer should carry the equivalent work
  boundaries       what reading the source does NOT establish
  source           file + the method names the notes came from

Hard rule: no line of decompiled source is reproduced here. Facts are stated, not
quoted. Numeric literals are recorded only when they are structural constants a
reimplementation must re-derive and test (e.g. a divisor of 2), and they are marked
as needing measurement against the real document.
"""
import json
import re
from pathlib import Path

ROOT = Path("H:/OpenMatrix9")
CORE = ROOT / "ref/matrix/decompiled-ui/Matrix"
OUT = ROOT / "docs" / "private-algorithm-notes.json"

NOTES = [
    {
        "id": "algo.builder.contract",
        "stage": "0. Hợp đồng builder (mọi builder đều theo)",
        "om9_backend": "pure Rust (service + trait), không phụ thuộc kernel",
        "source": {
            "file": "MatrixBuilder/MatrixBuilderBase.cs",
            "methods": ["MatrixBuilderBase (ctor)", "Cancel", "Close",
                        "ConduitCalcBoundingBox", "ConduitDrawForeground",
                        "ConduitDrawOverlay", "AddToDocument", "GetValues",
                        "Initialize", "ImagePicked", "SetClean",
                        "ReadUserData", "WriteUserData", "WriteId"],
            "also": ["MatrixBuilder/MatrixBuilderCore.cs:UpdateBuilders, PostDrawObjects",
                     "MatrixBuilder/MatrixGetPoint.cs:OnDynamicDraw"],
        },
        "verified_facts": [
            "MatrixBuilderBase là abstract class; ba member abstract: AddToDocument(), "
            "ConduitPostDrawObjects(pipeline), GetValues(List<MatrixDataBase>).",
            "Trạng thái builder là enum BuilderStatus, khởi tạo Unknown.",
            "Mỗi builder có BuilderName, EnglishName, và cờ HasGemListVb / HasMenuListVb.",
            "Có 3 hook conduit riêng: foreground, overlay, post-draw-objects; cộng thêm "
            "ConduitCalcBoundingBox cho hộp bao động.",
            "GetValues nhận List<MatrixDataBase> - tức giá trị tham số được truyền ra ngoài "
            "dưới dạng đối tượng dữ liệu, không phải field private.",
            "Có cặp ReadUserData/WriteUserData + WriteId: builder tự lưu/đọc trạng thái riêng.",
            "MatrixBuilderCore.UpdateBuilders điều phối nhiều builder cùng lúc.",
            "MatrixGetPoint kế thừa GetPoint và override OnDynamicDraw - chọn điểm có preview.",
        ],
        "algorithm": [
            "B1. Khởi tạo: builder nhận RunMode, dựng state, trả Result; thất bại thì dừng trước khi vẽ.",
            "B2. Chọn đầu vào: dùng GetPoint/GetObject của host, override OnDynamicDraw để preview "
            "ngay trong lúc chọn; mỗi lần chọn xong gọi UpdateBuilder để tính lại.",
            "B3. Preview: tính lại kết quả trong bộ nhớ rồi vẽ qua conduit, KHÔNG tạo đối tượng "
            "trong document; phân biệt foreground / overlay / post-draw-objects.",
            "B4. Tham số: GetValues đẩy các đối tượng dữ liệu ra cho UI; UI sửa rồi đẩy lại, "
            "kích hoạt B3.",
            "B5. Chốt: AddToDocument chuyển kết quả trong bộ nhớ thành đối tượng thật, kèm ID "
            "ổn định và user-data để lần sau đọc lại được.",
            "B6. Huỷ: Cancel/Close dọn state, document không đổi.",
            "B7. Bền vững: WriteUserData/WriteId khi chốt; ReadUserData khi mở lại để builder "
            "tái dựng tham số từ đối tượng đã tạo.",
        ],
        "inputs_outputs": {
            "inputs": "RunMode, lựa chọn hình học của người dùng, tham số từ UI",
            "outputs": "đối tượng document mới + user-data tham số + ID ổn định",
        },
        "boundaries": [
            "Đọc được hình dạng hợp đồng; KHÔNG chứng minh hành vi runtime của bất kỳ builder nào.",
            "Conduit là khái niệm của host (Rhino); OM9 phải tự ánh xạ sang lớp scene/preview của mình.",
            "Không có số liệu dung sai nào trong tệp này.",
        ],
    },
    {
        "id": "algo.gem.on.curve.multi",
        "stage": "3. Đặt đá theo curve (biến thể multi)",
        "om9_backend": "pure Rust (phân bố) + openNURBS/OCCT cho frame và biến đổi",
        "source": {
            "file": "MatrixBuilder/GemOnCurveMulti/GemOnCurveMultiBuilder.cs",
            "methods": ["Initialize", "OptionPicked", "UpdateBuilder", "UpdateHandles",
                        "GetValues", "AddToDocument"],
            "also": ["CS/CSGemOffsetCurves/GemOffsetCurve.cs:GemOffset, GemOffsetCurves, validGemShape",
                     "CS/CSUtils/CSUtilGeometry.cs:OrientToCurveNalp, OrientProfileToCurveNalp"],
        },
        "verified_facts": [
            "UpdateBuilder xoá danh sách kết quả trước, rồi thoát sớm nếu curve null hoặc IsValid false.",
            "Điểm dọc curve được xử lý bằng CHỈ SỐ RỜI RẠC trên một mảng thông tin curve "
            "(CrvInfoCount, GetFixedIndex, GetClosestIndexToPosition), không phải tham số liên tục.",
            "Vị trí đầu/cuối là tỉ lệ chuẩn hoá trong [0,1]; vị trí cuối được đặt bằng "
            "vị trí đầu trừ AbsoluteTolerance của document khi bật chế độ khép kín.",
            "Curve kín: khi tham số vượt qua đường may (ngưỡng so sánh 0.2 và 0.8 với giá trị "
            "lần trước) thì hệ số hướng _dirOver bị đảo dấu - tức có xử lý đường may riêng.",
            "Hướng duyệt suy ra từ so sánh chỉ số đầu và cuối, nhân với hệ số hướng.",
            "Có hai chế độ chia khoảng trống: hệ số 0.5 cho một phương pháp và 1.0 cho phương "
            "pháp còn lại, chia theo (số phần tử trừ một nửa) hoặc (số phần tử).",
            "Có tỉ lệ đường kính trên khoảng cách: (đường kính/2 + khoảng cách/2) / (đường kính/2) "
            "= 1 + spacing/diameter - dùng để hiệu chỉnh vị trí theo cỡ đá.",
            "Đường kính có thể thuôn (taper): nếu không bật taper thì đường kính cuối bị gán "
            "bằng đường kính đầu.",
        ],
        "algorithm": [
            "G1. Kiểm tra: curve phải tồn tại và hợp lệ; nếu không, trả về danh sách rỗng và không vẽ.",
            "G2. Chuẩn hoá tham số: kẹp start/end vào [0,1]; nếu khép kín thì end = start - dung sai "
            "tuyệt đối của document; nếu không bật taper thì endDiameter = startDiameter.",
            "G3. Phát hiện đường may: nếu curve kín và tham số nhảy qua mốc 0/1 giữa hai lần tính, "
            "đảo hệ số hướng để thứ tự đá không bị đảo.",
            "G4. Đổi tỉ lệ sang chỉ số: tìm chỉ số gần nhất cho start và end; suy hướng duyệt.",
            "G5. Quét: đi từng chỉ số theo hướng, thu thập vị trí ứng viên cho tới khi gặp chỉ số end.",
            "G6. Hiệu chỉnh khoảng trống còn dư ở đoạn cuối: chia đều phần dư cho các đá đã thu, "
            "với hệ số 0.5 hoặc 1.0 tuỳ phương pháp chia; tính lại frame cho từng đá sau khi dịch chỉ số.",
            "G7. Frame mỗi đá: lấy điểm + tiếp tuyến tại chỉ số, dựng transform định hướng theo curve.",
            "G8. Bán kính đặt đá: dùng hệ số 1 + spacing/diameter để bù cỡ đá và khoảng cách.",
            "G9. Chốt: AddToDocument tạo instance đá thật với transform đã tính, giữ ID ổn định.",
        ],
        "inputs_outputs": {
            "inputs": "curve chính, start/end (tỉ lệ), spacing (mm), start/end diameter, "
                      "phương pháp chia khoảng, cờ khép kín, cờ taper, cỡ và hình dạng đá",
            "outputs": "danh sách dòng instance đá (vị trí + transform + đường kính)",
        },
        "boundaries": [
            "KHÔNG đọc được giá trị mặc định của spacing/diameter từ tệp này.",
            "KHÔNG chứng minh công thức frame khớp với Rhino ở mọi trường hợp suy biến.",
            "Ngưỡng 0.2/0.8 và hệ số 0.5/1.0 là hằng số cấu trúc: phải đo lại trên document thật "
            "trước khi coi là đúng, không được chép nguyên làm đặc tả.",
            "Cách Matrix biểu diễn 'chỉ số rời rạc' phụ thuộc lớp curve riêng của họ; OM9 dùng "
            "arc-length của openNURBS/OCCT thì phải tự định nghĩa lại phép ánh xạ.",
        ],
    },
    {
        "id": "algo.gemshape.offset",
        "stage": "3. Đặt đá theo curve - offset curve theo hình dạng đá",
        "om9_backend": "OCCT (offset curve) + pure Rust cho bảng hình dạng và kiểm tra hợp lệ",
        "source": {
            "file": "CS/CSGemOffsetCurves/GemOffsetCurve.cs",
            "methods": ["GemOffset", "GemOffsetCurves", "validGemShape"],
        },
        "verified_facts": [
            "Lớp chỉ có 4 thành viên, trong đó validGemShape là phép kiểm tra hình dạng hợp lệ.",
            "GemOffset nhận (shape, gem, distance, out curve) theo chữ ký đã ghi ở inventory: "
            "đường offset phụ thuộc hình dạng đá chứ không chỉ khoảng cách.",
            "GemOffsetCurves là dạng số nhiều - sinh nhiều đường offset cho một hình dạng.",
        ],
        "algorithm": [
            "O1. Kiểm tra hình dạng đá có nằm trong tập hợp lệ không; nếu không, từ chối.",
            "O2. Từ hình dạng + cỡ đá, suy bán kính/đường bao tham chiếu.",
            "O3. Offset đường bao theo khoảng cách đã cho, ra một hoặc nhiều đường.",
            "O4. Trả đường kết quả cho bước dựng prong/bezel hoặc cho phân bố đá.",
        ],
        "inputs_outputs": {
            "inputs": "hình dạng đá (enum), tham số đá, khoảng cách offset",
            "outputs": "curve/curves offset",
        },
        "boundaries": [
            "KHÔNG đọc được bảng hình dạng hợp lệ gồm những giá trị nào (chỉ biết có hàm kiểm tra).",
            "KHÔNG chứng minh offset của họ tương đương offset của OCCT ở chỗ tự giao.",
        ],
    },
    {
        "id": "algo.geometry.kernel-calls",
        "stage": "Dùng chung cho bước 1, 2, 8 (thân, vành, cutter)",
        "om9_backend": "OCCT cho sweep/loft/boolean/offset; openNURBS cho curve và 3DM",
        "source": {
            "file": "CS/CSUtils/CSUtilGeometry.cs",
            "methods": ["CreateProng", "CreateProngDome", "CreateMeshProng", "CreateDome",
                        "MakeProngProfileStraight", "BlendCurve", "BlendCurveCleaned",
                        "BlendCurveCorners", "OffsetNew", "OriginOffset", "OriginOffsetXY",
                        "Trim2dLineCurveToSurface", "TrimStartOfCrv", "TrimEndOfCrv",
                        "SplitProfileInHalf", "IntersectionSurfaces", "GetClosestPointToBrep",
                        "GetClosestNormalToBrep", "NakedEdges", "CleanBrepArray",
                        "FixBrepSolidOrientation", "ExtrudeSurface",
                        "OrientProfileToCurveNalp", "OrientProfileToRailNalp",
                        "OrientToCurveNalp", "RingRailGetTopBottomRight"],
            "also": ["CS/CSUtils/CSUtilSweep.cs:Sweep1, Sweep1RC, Sweep1C4Simple, Sweep2, "
                     "Sweep2C4, Sweep2NoParam, Loft, LoftMatchCurves, MeshPipe",
                     "CS/CSUtils/CSUtilRailRevolve.cs:RailRevolve, RailRevolveGen, RailRevolveRhino4",
                     "CS/CSUtils/CSUtilCurves.cs:BlendCurve, TrimCurve"],
        },
        "verified_facts": [
            "Tên hàm cho thấy phân rã theo thao tác kernel: prong/dome, sweep 1 và 2 rail, "
            "rail revolve, loft, blend, offset (có cả biến thể New/Old), trim, split profile, "
            "giao surface, closest point/normal trên brep, naked edges, clean brep, "
            "fix hướng solid, extrude surface, và các hàm định hướng profile (Nalp).",
            "Có hậu tố _Old_V1 và Rhino4 trong tên hàm: tồn tại nhánh hành vi cũ được giữ lại.",
            "Có biến thể C4 riêng cho sweep (Sweep1C4Simple, Sweep2C4).",
            "Lớp CSUtilGeometry là static utility (48 method) - nơi tập trung lời gọi kernel.",
        ],
        "algorithm": [
            "K1. Định hướng: từ curve/rail, dựng frame và định hướng profile trước khi sweep "
            "(nhóm hàm Orient*Nalp, OrientProfileToRailNalp).",
            "K2. Dựng mặt: chọn sweep 1 rail, sweep 2 rail, rail revolve, loft hoặc extrude tuỳ số "
            "rail và kiểu profile; rail revolve có cả biến thể gen và biến thể tương thích Rhino4.",
            "K3. Làm sạch: sau khi dựng, kiểm tra naked edges, clean brep, sửa hướng solid; nếu "
            "không sạch thì không được coi là khối kín.",
            "K4. Cắt/giao: trim curve theo surface, giao surface-surface, split profile, offset.",
            "K5. Prong/dome: dựng prong từ profile thẳng hoặc dome; có cả biến thể mesh thuần.",
        ],
        "inputs_outputs": {
            "inputs": "curve/rail, profile, tham số kích thước, dung sai document",
            "outputs": "surface/brep/mesh; trạng thái sạch hay không",
        },
        "boundaries": [
            "Đây là BẢN ĐỒ THAO TÁC, không phải thuật toán số: chưa biết dung sai, bậc, "
            "hay cách chọn hướng nào cho trường hợp suy biến.",
            "Tên hàm không nói thứ tự gọi; thứ tự ở trên là suy luận từ tên nhóm và từ quy trình "
            "gia công, cần đối chiếu sheet manual trước khi dùng làm đặc tả.",
            "Biến thể _Old_V1/Rhino4 nghĩa là hành vi đã từng thay đổi - không được giả định "
            "chỉ có một hành vi đúng.",
        ],
    },
]


def main():
    payload = {
        "schema": "om9.private_algorithm_notes.v1",
        "policy": ("Ghi chú thuật toán do tác giả viết từ việc ĐỌC nguồn Matrix9. Không sao chép "
                   "mã, không trích nguyên văn. Dùng để viết implementation sample và đặc tả; "
                   "không phải bằng chứng runtime."),
        "rough_chain_order_evidence": {
            "source": "thứ tự trang trong sheet manual đã có trong repo",
            "sequence": [
                "m8book2 p134 ring-rail / m8book1 p87-88 / p139-140 outside-ring-rail / "
                "p140-141 custom-rail / p141 four-rail-profile / p163-164 custom-rail-advanced",
                "m8book2 p177-181 gem-on-curve / p181-187 gem-on-curve-advanced / "
                "p195-197 gem-on-curve-multi / p197-200 gem-profile-curve / p208-209 gem-on-surface",
                "m8book2 p203-207 halo-builder / p220-224 cluster-builder",
                "m8book2 p225-230 auto-pav / p230-231 pav-builder / p231-232 pav-prong / "
                "p231 pav-azure / p238-240 pav-sphere",
                "m8book2 p243-249 head-builder / p255-257 pull-object-to-rail / p257-260 prong-builder",
                "m8book2 p272-288 cutters (creating, gem-cutter, micro-prong, bright-cut-channel, "
                "bright-cutter, bezel-cutter, plane-cube) / p290-292 boolean-builder",
            ],
        },
        "notes": NOTES,
    }
    OUT.write_text(json.dumps(payload, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"notes written: {len(NOTES)}")
    for n in NOTES:
        print(f"  {n['id']:34s} {n['stage'][:50]}")
        print(f"      source: {n['source']['file']}")
        print(f"      facts={len(n['verified_facts'])} steps={len(n['algorithm'])} "
              f"boundaries={len(n['boundaries'])}")
    print()
    print("written:", OUT.relative_to(ROOT))


if __name__ == "__main__":
    main()
