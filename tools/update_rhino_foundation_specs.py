"""Normalize user-supplied Rhino 5 guide into a separate compatibility layer."""
from pathlib import Path
import hashlib
import json
import re
import yaml
from pypdf import PdfReader

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / 'ref/matrix9/OpenMatrix9_Codex_Spec_v1'
PDF = Path('C:/Users/Admin/Downloads/windows_pdf_user_s_guide (1).pdf')
SOURCE = 'rhino5-windows-user-guide-2016'

# Requirements are paraphrases. Ranges are printed pages; PDF page = printed + 8.
DATA = [
('CMD', 'Command dispatch and lifecycle', 3, 15,
 'Menu, toolbar và CMD khởi chạy cùng named command. Prompt nhận tên option, chữ gạch chân hoặc click option; nhận giá trị khoảng cách/góc. Esc hủy command. Enter/Space/right-click viewport xác nhận; khi idle thì repeat, ngoại trừ Undo/Delete và danh sách không repeat. Command transcript và prompt là hai vai trò riêng.',
 'Chạy cùng fixture qua menu và CMD, so sánh hình học và option; kiểm tra Esc giữa các bước, repeat sau Undo/Delete và Ctrl+Z/Ctrl+Y.'),
('OBJECT', 'Object semantics', 17, 23,
 'Phân biệt point, curve/polycurve, surface, polysurface, solid, lightweight extrusion và polygon mesh. Solid phải bao kín thể tích. Trimmed surface giữ underlying surface và trim loops; isocurve chỉ hỗ trợ hiển thị, edge là biên hình học. Mesh không tương đương NURBS.',
 'Kiểm tra loại hình học, open/closed, seam, trim loop và thể tích; không dùng shaded preview làm bằng chứng solid; kiểm tra output sau save/reload.'),
('SELECT', 'Selection and subobjects', 25, 30,
 'Click chọn; Shift thêm, Ctrl bỏ; click nền/Esc bỏ chọn. Window kéo phải chọn đối tượng nằm hoàn toàn trong cửa sổ; crossing kéo trái nhận cả đối tượng giao cửa sổ. Modifier áp dụng cả window/crossing. Ctrl+Shift chọn face/edge/control point/mesh component hoặc thành phần group.',
 'Fixture có đối tượng nằm trong, giao và ngoài window; kiểm tra additive/subtractive, face và edge đầu vào cho Loft/Extrude; không dùng bounding box thay thế bằng chứng giao hình học chính xác.'),
('VIEW', 'Viewport and navigation', 31, 35,
 'Bốn view thông thường gồm ba parallel và một perspective; có thêm two-point perspective. Right-drag pan parallel, rotate perspective; Shift+right-drag pan perspective; Ctrl+Shift+right-drag rotate parallel; Ctrl+right-drag hoặc wheel zoom. Có thể điều hướng trong command. Home/End lùi/tiến view, Plan nhìn vuông góc CPlane, Zoom Extents bao đối tượng. Double-click title maximize/restore.',
 'Kiểm tra từng projection và modifier bằng native viewport; camera, grid và object phải cùng phản ánh thay đổi; điều hướng giữa point picks không commit/cancel command.'),
('DISPLAY', 'Display modes', 32, 35,
 'Wireframe dùng edge/isocurve; Shaded hiển thị surface/solid theo màu layer/object/custom; Rendered dùng lighting/material. Display không đổi topology. Viewport title menu cung cấp view, camera/target, CPlane, grid và shading.',
 'Đổi mode trên từng view, giữ nguyên document geometry; kiểm tra màu, visibility và selection bằng native render.'),
('PICK', 'Grid, Ortho and object snaps', 37, 39,
 'Marker là điểm sẽ commit, có thể lệch cursor. Grid snap dùng spacing cấu hình được trên grid vô hạn. Ortho có tác dụng sau first point, theo góc cấu hình; giữ Shift tạm đảo trạng thái. Osnap có persistent và one-pick; thông thường ưu tiên hơn grid/constraint nhưng có trường hợp phối hợp. Right-click một Osnap bật riêng nó; Disable tạm ngưng, right-click Disable xóa các persistent modes.',
 'So sánh marker preview với tọa độ commit; thử nhiều Osnap đồng thời, one-pick, Disable và Shift; ghi quy tắc phối hợp cho command, không suy ra một thứ tự ưu tiên tuyệt đối.'),
('COORD', 'Coordinates and constraints', 39, 43,
 'CMD nhận x,y và x,y,z theo CPlane active; world frame cố định, CPlane độc lập từng viewport. r x,y (không có khoảng trắng khi nhập, ví dụ r2,3) tương đối với last point. Nhập distance rồi <angle kết hợp hai constraint. Elevator: Ctrl+pick base, tiếp tục chọn hoặc nhập cao độ theo CPlane Z; SmartTrack dùng temporary points/lines trong thời gian command.',
 'Cùng input trong Top/Front/Right và CPlane xoay phải ra đúng world position; thử r2,3, distance + <angle, height âm/dương và snap vào tracking intersections. Không tự suy ra cú pháp world-coordinate chưa được PDF mô tả.'),
('SURFACE', 'Surfaces from curves', 45, 59,
 'EdgeSrf nhận ba/bốn edge curves. ExtrudeCrv quét profile theo đường thẳng. Loft tạo mặt qua section curves không cần rail. Revolve nhận profile và hai điểm axis, có FullCircle. RailRevolve nhận profile, rail và axis. Sweep1 nhận rail trước rồi sections; Sweep2 nhận hai rails trước rồi sections.',
 'Kiểm tra thứ tự selection, cross sections, orientation, seam và output surface; dùng fixture mở/đóng và preview option có căn cứ. Không biến lựa chọn trong tutorial thành default toàn cục.'),
('EDIT', 'Join, Explode, Trim and Split', 61, 61,
 'Join nối curve segments thành polycurve hoặc surfaces kề nhau thành polysurface. Explode tách phần đã join. Trim xóa phần chọn bỏ; Split giữ tất cả phần. Split surface có thể dùng curve/surface/polysurface/isocurve; Untrim khôi phục underlying surface, có option giữ trimming curve.',
 'So sánh số parts, phần giữ/xóa, topology và Undo cho Join/Explode/Trim/Split; giữ riêng unsupported input và tolerance chưa có bằng chứng.'),
('POINTS', 'Control point editing', 61, 63,
 'PointsOn bật control points của curve/surface; PointsOff hoặc Esc tắt. NURBS được kéo về control points, không nhất thiết đi qua chúng. Không bật control points thông thường của polysurface; SolidPtOn là grip khác (printed p.22). Rebuild/ChangeDegree/Smooth thay cấu trúc control points.',
 'Phân biệt control point, edit point và solid grip; thử single surface so với polysurface, thao tác transform trên points và cập nhật hình học.'),
('TRANSFORM', 'Transforms', 65, 83,
 'Move nhận from/to; Copy giữ source và có nhiều destination trước Enter. Rotate theo CPlane quanh center, nhận angle hoặc reference points. Scale nhận origin với factor hoặc reference lengths; có scale theo chiều. Mirror tạo ảnh đối xứng theo mirror plane. Array tạo copies theo hàng/cột hoặc vòng; Orient kết hợp đặt vị trí, xoay và scale.',
 'Kiểm tra tọa độ, copy ownership, source giữ/bỏ theo option có căn cứ, base point, angle và kích thước; không thay transform bằng thay camera.'),
('ANALYSIS', 'Measurement and diagnostics', 85, 88,
 'Distance giữa hai points, Angle giữa hai lines, Radius tại vị trí curve, Length của curve. Dir hiển thị/đảo direction. Curvature và surface analysis giúp đánh giá hình dạng; edge evaluation/diagnostics kiểm tra edges và bad geometry.',
 'Dùng geometry có đáp án giải tích, kiểm tra units và direction flip; xác minh naked edges/validity riêng với vẻ ngoài shaded.'),
('ORGANIZE', 'Layers, groups and blocks', 89, 90,
 'Layers tổ chức object và properties; groups thao tác nhiều object cùng nhau; blocks dùng definition và instances; worksessions là cơ chế riêng. Không đồng nhất group với joined topology hoặc block với copied geometry.',
 'Kiểm tra current layer, visibility/lock, group membership và block instance/definition khi command thực sự dùng chúng; kiểm tra save/reload.'),
('ANNOTATE', 'Dimensions, text and notes', 90, 92,
 'Dimensions dùng current dimension style; không tự associative trừ khi được tạo với History enabled, sửa dimension không sửa geometry. Text/Leader/Dot khác nhau; Dot song song view và kích thước màn hình không đổi khi zoom. Notes được lưu trong model file.',
 'Kiểm tra text/style, kích thước đo, History on/off, Dot khi zoom và Notes sau reopen; không nhầm annotation Text với TextObject solid.'),
('MAKE2D', 'Make2D and layouts', 92, 92,
 'Make2D tạo silhouette curves chiếu phẳng lên world XY; có current view/CPlane, four-view US/European, hidden-line layers và tangent edges. Layout/page space và detail viewport được minh họa ở printed p.273-276.',
 'Kiểm tra output 2D geometry, projection, hidden/tangent layer, page/detail scale và print width khi áp dụng; không coi screenshot là Make2D geometry.'),
('RENDER', 'Rendering', 93, 96,
 'Rendering gồm lights, materials, environment, ground plane và lưu image. Materials có màu, finish, transparency, texture/bump, gán cho object hoặc layer. Rendered viewport khác final rendered image. PDF này không xác định engine hay jewelry presets của Matrix.',
 'Kiểm tra assignment object/layer và output image; đối chiếu riêng engine, gem/metal settings và scene presets theo Matrix specs.'),
('SOLID', 'Solid tutorial workflow', 99, 123,
 'Pull Toy minh họa solid primitives, ExtrudeCrv, Pipe, ArrayPolar và Mirror. Mechanical Part printed p.261-272 minh họa BooleanDifference theo thứ tự subtract-from rồi subtract-with, holes, closed-solid inspection và Make2D.',
 'Fixture extrusion kín/hở và boolean có cutter; kiểm tra volume, holes, input retention theo option tường minh; giá trị tutorial chỉ là fixture.'),
('PICTURE', 'PictureFrame tracing', 233, 238,
 'Dragonfly dùng PictureFrame ở Top/Front, reference line 50 mm để đặt kích thước; Hide ảnh view phụ, Curve trace, Mirror/Bend, CSec và Loft. 50 mm là dữ liệu bài tập, không phải default PictureFrame.',
 'Kiểm tra image plane/orientation, scale theo reference, visibility và curve trace trong đúng CPlane.'),
('FLOW', 'UV mapping and FlowAlongSrf', 255, 259,
 'Wrap Text dùng TextObject dạng Solids, CreateUVCrv để lấy surface border sang plane, đặt objects trong UV rectangle, tạo base surface rồi FlowAlongSrf với Rigid=No. Pick gần corner của base và seam của target ảnh hưởng placement.',
 'Kiểm tra corner/seam orientation, base/target surface và deformed output; height 1.5, thickness .1 và Rigid=No là fixture bài tập, không suy ra mọi command mặc định.'),
]

def write_json(path, data):
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

def main():
    pages = PdfReader(PDF).pages
    contracts = []
    for key, title, start, end, behavior, acceptance in DATA:
        contracts.append(dict(id='RH5-'+key, title=title, source_id=SOURCE,
            printed_pages=[start,end], pdf_pages=[start+8,end+8],
            source_behavior=behavior, acceptance=acceptance,
            implementation_status='unverified'))
    common = ['CMD','OBJECT','SELECT']
    domain = {
        '01-core':['VIEW','DISPLAY','PICK','COORD','ORGANIZE','ANNOTATE','ANALYSIS'],
        '02-curve':['PICK','COORD','EDIT','POINTS'],
        '03-surface':['PICK','COORD','SURFACE','EDIT','POINTS','ANALYSIS'],
        '04-solid':['PICK','COORD','SOLID','EDIT','ANALYSIS'],
        '05-transform':['PICK','COORD','TRANSFORM'],
        '06-tsplines':['PICK','COORD','TRANSFORM'],
        '07-matrix-art':['PICK','COORD'], '08-builders':['PICK','COORD','SURFACE','TRANSFORM'],
        '09-tools':['PICK','COORD','EDIT','TRANSFORM','ANALYSIS'],
        '10-gems':['PICK','COORD','TRANSFORM','ORGANIZE'],
        '11-settings':['PICK','COORD','SURFACE','SOLID'],
        '12-cutters':['PICK','COORD','SOLID','TRANSFORM'],
        '13-render':['DISPLAY','RENDER','ORGANIZE'],
        '14-subd':['PICK','COORD','TRANSFORM'],
        '15-emboss':['PICK','COORD'],
        '16-matrix-tools':['PICK','COORD','TRANSFORM','ORGANIZE'],
    }
    commands = {'pictureframe':'PICTURE','flowalongsrf':'FLOW','createuvcrv':'FLOW',
        'make2d':'MAKE2D','pointson':'POINTS','pointsoff':'POINTS','solidpton':'POINTS',
        'join':'EDIT','explode':'EDIT','trim':'EDIT','split':'EDIT','untrim':'EDIT',
        'move':'TRANSFORM','copy':'TRANSFORM','rotate':'TRANSFORM','scale':'TRANSFORM',
        'mirror':'TRANSFORM','array':'TRANSFORM','arraypolar':'TRANSFORM','orient':'TRANSFORM',
        'edgesrf':'SURFACE','extrudecrv':'SURFACE','loft':'SURFACE','revolve':'SURFACE',
        'railrevolve':'SURFACE','sweep1':'SURFACE','sweep2':'SURFACE',
        'distance':'ANALYSIS','angle':'ANALYSIS','radius':'ANALYSIS','length':'ANALYSIS','dir':'ANALYSIS',
        'notes':'ANNOTATE','text':'ANNOTATE','leader':'ANNOTATE','dot':'ANNOTATE',
        'group':'ORGANIZE','ungroup':'ORGANIZE'}
    features = json.loads((PACK/'FEATURES.json').read_text(encoding='utf-8'))
    mappings = []
    marker = '\n## Rhino 5 foundation compatibility\n'
    for feature in features:
        keys = list(dict.fromkeys(common + domain[feature['domain']]))
        command = (feature.get('command') or '').lower()
        if command in commands and commands[command] not in keys:
            keys.append(commands[command])
        direct = commands.get(command)
        refs = ['RH5-'+k for k in keys]
        mapping = dict(feature_id=feature['id'],spec_path=feature['spec_path'],
            contracts=refs, applicability='conditional_on_actual_inputs_and_workflow',
            mapping_basis='OpenMatrix9 domain baseline; exact command match where listed',
            direct_command_contract=('RH5-'+direct if direct else None),
            command_coverage=('guide_workflow' if direct else 'foundation_only_command_semantics_not_established'),
            acceptance_status='unverified')
        mappings.append(mapping)
        feature['rhino_foundation'] = mapping.copy()
        spec = PACK/feature['spec_path']
        body = spec.read_text(encoding='utf-8').split(marker)[0].rstrip()
        link = '../../RHINO5_FOUNDATION.md'
        section = marker + '\n'
        section += 'Supplemental source: Rhino 5 User\'s Guide (Windows), Robert McNeel & Associates, 2016-11-30.\n\n'
        section += 'This is a separate compatibility contract; the Matrix Source behavior above remains authoritative for Matrix-specific behavior. Applicability below is an OpenMatrix9 specification decision, conditional on the feature actually using that operation. It is not a recovered dependency graph.\n\n'
        for key in keys:
            item = next(c for c in contracts if c['id']=='RH5-'+key)
            section += f"### {item['id']} - {item['title']}\n\nSource: printed pp. {item['printed_pages'][0]}-{item['printed_pages'][1]}; PDF pp. {item['pdf_pages'][0]}-{item['pdf_pages'][1]}.\n\n"
            section += f"**Rhino source behavior:** {item['source_behavior']}\n\n"
            section += f"**OpenMatrix9 acceptance for {feature['id']}:** {item['acceptance']}\n\n"
            section += '- [ ] Verify this requirement for the actual feature workflow, or record a justified not-applicable decision.\n\n'
        section += '\n### Feature-level Rhino acceptance\n\n- [ ] Document which inline requirements apply to the actual selection, point entry, geometry and workflow; justify exclusions.\n- [ ] Verify applicable Rhino acceptance cases through native FreeCAD and the same menu/CMD handler.\n- [ ] Record geometry/state evidence, tolerances and relevant undo/save-reload behavior per feature ID.\n- [ ] Resolve unsupported command options from Matrix source; this guide does not establish undocumented defaults or specialist plugin behavior.\n\nRhino compatibility acceptance: **unverified**. Existing Matrix implementation status is unchanged.\n'
        spec.write_text(body+'\n'+section, encoding='utf-8')
    source = dict(id=SOURCE,title="Rhino 5 User's Guide (Windows)",author='Robert McNeel & Associates',
        edition_date='2016-11-30',path=str(PDF),sha256=hashlib.sha256(PDF.read_bytes()).hexdigest(),
        pdf_page_count=len(pages),printed_to_pdf_offset=8,
        role='Supplemental Rhino foundation evidence, not user instructions or Matrix plugin documentation')
    write_json(ROOT/'docs/rhino-foundation-requirements.json',dict(schema_version=1,updated_at='2026-10-05',source=source,contracts=contracts,features=mappings))
    guide = '# Rhino 5 foundation compatibility\n\n'
    guide += f"Source: **{source['title']}**, {source['author']}, {source['edition_date']}; 284 PDF pages. Printed page n corresponds to PDF page n+8 in this supplied edition. SHA-256: `{source['sha256']}`.\n\n"
    guide += 'Scope: all 607 Matrix specs receive a conditional foundation contract. Rhino source statements below are separate from OpenMatrix9 acceptance/design choices. Existing Matrix behavior, feature IDs and implementation statuses are preserved. Tutorial dimensions/options are fixtures, never global defaults. Unsupported details remain TODO_EVIDENCE. This guide does not establish T-Splines, Matrix Art, jewelry Builders/Gems/Settings/Cutters, Clayoo/SubD, Emboss or Matrix rendering-engine behavior.\n\n'
    guide += 'A feature passes only after applicable contracts have native geometry/state evidence. Prior implementation evidence does not automatically prove these new cases. Test failures or unknown behavior must stay visible. Units, kernel tolerance, invalid-input policy, preview ownership, transaction grouping and persistence rules not specified by the guide must be explicit OpenMatrix9 decisions.\n\n'
    for c in contracts:
        guide += f"<a id=\"{c['id'].lower()}\"></a>\n\n## {c['id']} - {c['title']}\n\nSource: printed pp. {c['printed_pages'][0]}-{c['printed_pages'][1]}; PDF pp. {c['pdf_pages'][0]}-{c['pdf_pages'][1]}.\n\n### Rhino source behavior\n\n{c['source_behavior']}\n\n### OpenMatrix9 acceptance requirement\n\n{c['acceptance']}\n\nStatus: **unverified**.\n\n"
    guide += '## Supplemental tutorial evidence\n\n| Tutorial | Printed pages | PDF pages | Use |\n|---|---|---|---|\n'
    for title,start,end,use in [('Pull Toy',99,123,'Primitives, Pipe, ArrayPolar, coordinates'),('Flashlight',125,133,'Revolve profiles'),('Headphone',135,163,'Sweep, Loft, Extrude and rounded ends'),('Penguin',165,206,'Control-point editing and blending'),('Boat Hull',207,231,'Fairness, curvature, Loft/Sweep, Trim'),('Dragonfly',233,254,'PictureFrame, trace, CSec, Loft, BlendSrf, Pipe'),('Wrap Text',255,259,'TextObject, UV, FlowAlongSrf'),('Mechanical Part',261,272,'BooleanDifference, holes, Make2D, dimensions'),('Layouts',273,276,'Page/detail view, title block, print widths')]:
        guide += f'| {title} | {start}-{end} | {start+8}-{end+8} | {use} |\n'
    guide += '\n## Conflicts and limits\n\n- Matrix mouse/keyboard profiles remain distinct from the Rhino default navigation table. A deliberate Matrix override requires per-feature evidence; do not silently rewrite one profile using the other.\n- Ortho is described by Rhino independently of the Osnap master switch; the Matrix Ortho spec has an additional master-switch cue. Preserve both evidence statements and test/document the selected Matrix behavior.\n- Ctrl means different things in selection, navigation and Elevator; route by command/input context and active gesture.\n- Polysurface control points and SolidPtOn grips are distinct. Clayoo/SubD editing must use its own Matrix evidence.\n- Rhino dimensions are non-associative unless created with History; do not assume all dimensions or builders have live dependencies.\n- Snap type-specific algorithms, all command options/defaults, file-format fidelity, mesh-to-NURBS quality and specialist plugin operations are not fully specified by this user guide.\n'
    (PACK/'RHINO5_FOUNDATION.md').write_text(guide,encoding='utf-8')
    write_json(PACK/'FEATURES.json',features)
    (PACK/'FEATURES.yaml').write_text(yaml.safe_dump(features,allow_unicode=True,sort_keys=False),encoding='utf-8')
    for name in ['INDEX.md','README.md','SOURCES.md','IMPLEMENTATION_RULES.md','ALL_FEATURES.md']:
        path=PACK/name
        body=path.read_text(encoding='utf-8').split('\n## Supplemental Rhino 5 foundation')[0].rstrip()
        path.write_text(body+'\n\n## Supplemental Rhino 5 foundation\n\nAll 607 specs link to [RHINO5_FOUNDATION.md](RHINO5_FOUNDATION.md). The supplied Rhino 5 Windows guide (2016-11-30) adds conditional foundation acceptance requirements; it does not replace Matrix source behavior or establish specialist plugin semantics. Machine-readable contracts and per-feature mapping: `../../../docs/rhino-foundation-requirements.json` (from package root). Rhino acceptance remains unverified pending native evidence.\n',encoding='utf-8')
    for path in (PACK/'specs').glob('*/README.md'):
        body=path.read_text(encoding='utf-8').split('\n## Rhino foundation')[0].rstrip()
        path.write_text(body+'\n\n## Rhino foundation\n\nEach feature includes conditional Rhino compatibility requirements. Read [the shared contracts](../../RHINO5_FOUNDATION.md) and record applicability and native acceptance per feature. Specialist Matrix behavior continues to use its original source.\n',encoding='utf-8')
    locations=ROOT/'docs/openmatrix9-reference-locations.json'
    obj=json.loads(locations.read_text(encoding='utf-8'))
    obj.setdefault('manuals',{})[SOURCE]=str(PDF).replace('\\','/')
    obj['rhino_foundation_spec']='ref/matrix9/OpenMatrix9_Codex_Spec_v1/RHINO5_FOUNDATION.md'
    write_json(locations,obj)
    corepath=ROOT/'docs/core-requirements.json'
    core=json.loads(corepath.read_text(encoding='utf-8'))
    byid={m['feature_id']:m for m in mappings}
    for item in core['items']:
        item['rhino_foundation']=byid[item['id']]
    write_json(corepath,core)
    manifest=[]
    for path in sorted(PACK.rglob('*')):
        if path.is_file() and path.name!='MANIFEST.json':
            data=path.read_bytes()
            manifest.append(dict(path=path.relative_to(PACK).as_posix(),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
    write_json(PACK/'MANIFEST.json',manifest)
    print(f'Updated {len(features)} specs, {len(contracts)} contracts, {len(core["items"])} core records; manifest {len(manifest)} files.')

if __name__=='__main__':
    main()
