"""Route authored screenshot requirements without promoting implementation.

Only textual observations/hash metadata enter public specs. Original commercial
reference images remain outside the repository. V1/status/ID baselines survive.
"""
import hashlib
import json
from pathlib import Path
import re
import yaml

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT / "OpenMatrix9_Codex_Spec_v2"
REFERENCE = "specs/00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md"
TITLE = "## Bổ sung ảnh Matrix9 — 2026-10-14"
FEATURES = {
    "OM9-IFACE-001": ("sidebar", ["S1", "S2", "S3", "S4", "S5", "S6"],
        "Giữ cột sidebar compact khoảng 300 pixel trong các crop, header xám/viền mảnh/nút X và đúng thứ tự sections. CMD nằm bên phải sidebar; toolbar tabs → icon row → viewport tabs nằm dưới input. Kích thước crop không xác lập DPI/logical pixel. Active title/tab xanh theo view identity, không gắn cứng Perspective.", "M9IMG-01, M9IMG-03, M9IMG-09"),
    "OM9-MAIN-001": ("sidebar", ["S1", "S5"],
        "Hai hàng category giữ File/View/Utilities/Measure/Custom/Reset và Curve/Surface/Solid/Transform/SubD/Art, Builder/Tools/Gems/Setting/Cutters/Render. S1 Curve có glyph vàng; S5 Solid có glyph cam; hàng chung xanh phía dưới vẫn giữ. Main category độc lập toolbar Standard/CPlanes. Glyph khó đọc chưa có tooltip/handler thì không gán chức năng gần giống.", "M9IMG-01, M9IMG-02"),
    "OM9-LAYER-001": ("layers", ["S2"],
        "Hai cột cơ bản/mở rộng và khoảng cách nhóm bốn slot đúng bảng tham chiếu. Metal01 có highlight tên xanh; tên/current, swatch, Assign triangle, lock và visibility state là các tín hiệu riêng. Lights/Hide/Show ở hàng đầu. Advanced đang mở là trạng thái ảnh, không đổi default thu. Giữ footer C A L V C đã được người dùng yêu cầu bỏ và mọi permission/Hidden rule đã duyệt.", "M9IMG-04"),
    "OM9-PROJECT-001": ("projects", ["S2", "S3"],
        "Project rows bên trái và Job Bag shelf bên phải có scrollbar riêng. Default được chọn; shelf hiện 1–3 hoặc cuộn tới 13–14, không khóa capacity ở ba/mười. Giữ card index/thumbnail/actions và snapshot shelf Master/Creation/Parts/Render/Output có offset riêng. Stable bag identity không phải index/offset; Load bag/import khác Load snapshot toàn document và vẫn có Save/Discard/Cancel.", "M9IMG-05, M9IMG-06"),
    "OM9-PROJECTDB-001": ("projects", ["S2", "S3"],
        "Mngr hiện dưới danh sách project, tách khỏi action từng Job Bag và snapshot shelf. Ảnh chỉ xác nhận entry/button/layout; không xác nhận database dialog, codec mpj, storage root, search hay concurrent writers. Không báo opened/saved/restored chỉ từ click hoặc thumbnail.", "M9IMG-05, M9IMG-06"),
    "OM9-VIEWPORT-001": ("viewport-layout", ["S4", "S6"],
        "Layout TL Looking Down/TR Perspective/BL Through Finger/BR Side View; tab order Perspective/Through Finger/Side View/Looking Down/+ là thứ tự riêng. Looking Down active trong ảnh có title/tab xanh; startup Perspective vẫn là default đã duyệt. Một CAD master có ba projection wireframe và Perspective shaded, camera/grid/mode mỗi view riêng. Menu tiêu đề đúng S6, gắn ViewportId/session và reject stale view.", "M9IMG-08, M9IMG-09, M9IMG-10, M9IMG-11"),
    "OM9-INFO-008": ("viewport-layout", ["S4", "S6"],
        "Thanh viewport tabs nằm ngay trên canvas, đúng thứ tự Perspective/Through Finger/Side View/Looking Down/+; active green state đồng bộ title. Không sắp tabs theo vị trí 2×2 hoặc đổi startup từ ảnh sau Box. Dấu + quan sát được không tự xác lập action khi chưa có contract/interaction evidence.", "M9IMG-09"),
    "OM9-INFO-010": ("viewport-menu", ["S1", "S6"],
        "Phân biệt Display panel và menu tiêu đề viewport. Menu S6 gồm Maximize, radio Wireframe/presets, display options, navigation, camera/CPlane, layout, bitmap/grid/capture, Close Viewport và các mục cuối đúng thứ tự/separators. Submenu chưa mở vẫn unknown. Mode action scope theo view được menu capture, không đổi geometry hoặc tạo material ngầm.", "M9IMG-10, M9IMG-11, M9IMG-12"),
    "OM9-DISPLAY-008": ("viewport-menu", ["S1", "S4", "S6"],
        "Menu tiêu đề có Wireframe cùng radio và 26 preset khác theo danh sách S6; khác sidebar Shade Mode dropdown/five-slot filtering đã đặc tả. Giữ spelling Detect Backfaces/ClayooSculpt, map stable IDs mà không đổi diagnostics color policy hoặc chứng nhận plugin. Per-view Wireframe/Shaded cùng tồn tại; mode unavailable visible-disabled và không fallback thành pass.", "M9IMG-08, M9IMG-10, M9IMG-12"),
    "OM9-DISPLAY-001": ("viewport-layout", ["S4"],
        "Perspective có mặt box shaded trong khi các view khác wireframe. Preview/display cache không tạo master hoặc thay topology CAD; màu xanh ảnh chưa xác lập selection/current layer/material. So geometry/IDs/History trước/sau thay preview mode và revalidate session.", "M9IMG-08, M9IMG-12"),
    "OM9-DISPLAY-002": ("viewport-menu", ["S6"],
        "Entry Shade Selected Objects Only nằm sau Print Preview/Flat Shade và trước nhóm navigation. Label nhìn thấy không chứng minh đang bật hoặc phạm vi selection. Giữ selected-only scope/IDs/permissions theo feature; không shade mọi object hay làm selection/history thay đổi.", "M9IMG-10, M9IMG-12"),
    "OM9-DISPLAY-004": ("viewport-layout", ["S4", "S6"],
        "Canvas đen với finite world grid, vùng ngoài grid đen, grid thay theo camera/pan từng view. Glyph trục và title không phải geometry. Giữ XY/XZ/YZ và trục dương màu đã duyệt; Grid Options... trong menu là dialog entry, không suy số cells/spacing/default từ crop.", "M9IMG-08, M9IMG-11, M9IMG-12"),
    "OM9-DISPLAY-006": ("sidebar", ["S1", "S4"],
        "Combo Coarse Mesh nằm cạnh Shaded nhưng có state chất lượng tessellation riêng. Creating meshes là feedback cache sau Box, không chuyển polysurface master thành mesh hoặc thêm object. Cache density/budget/Cancel và per-view mode phải nghiệm thu riêng.", "M9IMG-03, M9IMG-07, M9IMG-12"),
    "OM9-INFO-004": ("cmd-box", ["S4", "S5"],
        "Transcript nền xanh và input Command: ghim bên dưới với scrollbar history riêng. Box prompts/feedback giữ thứ tự; hai selection/Gumball hint lines có thể lặp theo events, không deduplicate toàn bộ history hoặc tự báo Alt duplicate. Replay phải phân biệt event hợp lệ với callback trùng; failed/Cancelled không là successful command history.", "M9IMG-02, M9IMG-07"),
    "OM9-SOLID-012": ("cmd-box", ["S4", "S5"],
        "Ảnh xác nhận spelling Command: _Box; first base corner với Diagonal/3Point/Vertical/Center, other corner or length với 3Point, rồi Height. Press Enter to use width. Giữ default hai góc rồi Height, không hỏi mode bắt buộc. Một polysurface master hiện ở bốn view; Creating meshes là cache feedback. Green appearance/Gumball hint không chứng minh current layer, selected color hoặc duplicate. Không thay contracts Cube/Vertical/Center chỉ từ ảnh.", "M9IMG-07, M9IMG-08"),
}


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def replace_block(text, heading, body):
    if heading in text:
        start = text.index(heading)
        end = text.find("\n## ", start + len(heading))
        return text[:start] + body + (text[end + 1:] if end >= 0 else "")
    return text + "\n\n" + body


def main():
    assert (PACKAGE / REFERENCE).is_file()
    features = read_json(PACKAGE / "FEATURES.json")
    before = [(f["id"], f["command"], f["implementation_status"], f["implementation_validation_status"]) for f in features]
    by_id = {f["id"]: f for f in features}
    assert set(FEATURES) <= set(by_id) and len(features) == 607
    lookup = read_json(PACKAGE / "FEATURE_LOOKUP.json")
    yaml_path = PACKAGE / "FEATURES.yaml"
    yaml_text = yaml_path.read_text(encoding="utf-8")
    for fid, (anchor, sources, detail, cases) in FEATURES.items():
        feature = by_id[fid]
        link = f"{REFERENCE}#{anchor}"
        metadata = {"spec_path": link, "source_ids": sources,
                    "received_on": "2026-10-14", "evidence_kind": "user_screenshot_observations",
                    "validation_status": "requirements_only_not_runtime_evidence"}
        feature["ui_reference_supplements"] = [x for x in feature.get("ui_reference_supplements", [])
                                               if x.get("spec_path") != link] + [metadata]
        lookup[fid]["ui_reference_supplements"] = feature["ui_reference_supplements"]
        spec = PACKAGE / feature["spec_path"]
        content = spec.read_text(encoding="utf-8")
        body = (f"{TITLE}\n\nNguồn {', '.join(sources)}; [phân tích và trace](../00-rhino-core/MATRIX9_SCREENSHOT_DETAILS_2026-10-14.md#{anchor}). "
                "Yêu cầu bổ sung, chưa là nghiệm thu hoặc backend capability.\n\n"
                f"{detail}\n\n- [ ] Kiểm các ca `{cases}` trên executable OM9; không chuyển ảnh thành pass.\n\n")
        if TITLE in content:
            content = replace_block(content, TITLE, body)
        else:
            content = content.replace("## Hợp đồng thiết kế\n", body + "## Hợp đồng thiết kế\n", 1)
        assert TITLE in content
        spec.write_text(content, encoding="utf-8")
        pattern = rf"(?ms)^- id: {re.escape(fid)}\n.*?(?=^- id: |\Z)"
        match = re.search(pattern, yaml_text)
        assert match
        block = re.sub(r"(?ms)^  ui_reference_supplements:\n.*\Z", "", match.group())
        field = yaml.safe_dump({"ui_reference_supplements": feature["ui_reference_supplements"]}, allow_unicode=True,
                               sort_keys=False, width=120)
        block = block.rstrip() + "\n" + "\n".join("  " + line for line in field.rstrip().splitlines()) + "\n"
        yaml_text = yaml_text[:match.start()] + block + yaml_text[match.end():]
    assert yaml.safe_load(yaml_text) == features, "JSON/YAML catalog divergence"
    assert before == [(f["id"], f["command"], f["implementation_status"], f["implementation_validation_status"]) for f in features]
    write_json(PACKAGE / "FEATURES.json", features)
    yaml_path.write_text(yaml_text, encoding="utf-8")
    write_json(PACKAGE / "FEATURE_LOOKUP.json", lookup)
    ledger = read_json(PACKAGE / "MIGRATION_LEDGER.json")
    entry = {"date": "2026-10-14", "spec_path": REFERENCE, "source_ids": [f"S{i}" for i in range(1, 7)],
             "affected_feature_ids": list(FEATURES), "acceptance_cases": [f"M9IMG-{i:02}" for i in range(1, 13)],
             "status": "requirements_only_not_runtime_evidence", "images_copied_to_repository": False,
             "implementation_status_promoted": False}
    ledger["ui_reference_supplements"] = [x for x in ledger.get("ui_reference_supplements", [])
                                         if x.get("spec_path") != REFERENCE] + [entry]
    assert ledger["target_implementation_count"] == 0
    write_json(PACKAGE / "MIGRATION_LEDGER.json", ledger)
    capability_path = PACKAGE / "specs/00-rhino-core/CAPABILITY_LOOKUP.json"
    capabilities = read_json(capability_path)
    assert len(capabilities["capabilities"]) == 148
    capabilities["ui_reference_supplement_path"] = Path(REFERENCE).name
    capabilities["ui_reference_fixture_ids"] = entry["acceptance_cases"]
    write_json(capability_path, capabilities)
    requirements_path = ROOT / "docs/core-requirements.json"
    requirements = read_json(requirements_path)
    requirements["ui_reference_supplements"] = [x for x in requirements.get("ui_reference_supplements", [])
                                               if x.get("spec_path") != REFERENCE] + [entry]
    write_json(requirements_path, requirements)
    baseline = read_json(PACKAGE / "SOURCE_READ_AUDIT.json")
    for row in baseline["files"]:
        path = ROOT / "OpenMatrix9_Codex_Spec_v1" / row["path"]
        with path.open("rb") as stream:
            sha = hashlib.file_digest(stream, "sha256").hexdigest()
        assert path.stat().st_size == row["bytes"] and sha == row["sha256"], row["path"]
    print(json.dumps({"features_linked": len(FEATURES), "catalog_count": len(features),
                      "v1_files_unchanged": len(baseline["files"]), "images_copied": 0,
                      "runtime_status_promoted": False}))


if __name__ == "__main__":
    main()
