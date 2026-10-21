# 3DM — kế hoạch import đủ object (Rhino 5 jewelry)

Ngày: 2026-10-20 · Trạng thái: **R5-B CORE E2E GREEN — MANUAL UI VÀ EXTENDED V5 CÒN MỞ** · Owner nghiệp vụ: `OM9-L2-01-13` (M03)

Tài liệu này là kế hoạch thi công cho yêu cầu "import 3DM này cho đủ object". Nó dựa trên chẩn đoán
thực tế file `bong 5X7.3dm`, không dựa trên kế hoạch cũ hay giả định.

Phần còn thiếu về tables/attributes/classes V5 được chốt riêng tại
[`THREEDM-V5-EXCHANGE-COMPLETION.md`](THREEDM-V5-EXCHANGE-COMPLETION.md).

## 0a. Kết quả triển khai Rhino 5 (2026-10-20)

Đã chạy thật bằng build release `--features occt-brep` + native mới:

- Import file thật: **44/44 native object bind**, 34 layer; 29 root `Geometry::Brep` + 13 `Curve` +
  1 `Instance`, và **1 definition member BRep**; tổng **30/30 BRep thành `Geometry::Brep`**,
  `breps_opaque=0` (strict mode; nếu một BRep lỗi, import abort).
- 29/29 root BRep có display mesh khác rỗng; definition member cũng convert thành công.
- `.om9` 8.566.153 byte; release import **5,37 s**.
- Export release: 2.182.344 byte 3DM; native `--inspect`: **44 object, 34 layer, 1 definition,
  30 ON_Brep, 0 diagnostic**; export **4,83 s**.
- Reimport file export: **43 root object** (29 `Brep` + 13 `Curve` + 1 `Instance`), 34 layer, 0 opaque.
- Test: `om9-exchange`, `om9-archive`, `om9-kernel --features 3dm-brep` xanh; app lib **117/117**.
- Public audit git-public scope: **0 errors**; `--strict` vẫn báo private material.
- Native artifact: `build/native-exchange/Release/om9-exchange-native.exe`,
  SHA-256 `923caba857b4f738b6641f3dfe1abd68705c33e32c5aab0de3f21e9069b1d1be`.
- `occt-brep` là **default feature** của `om9-app`; build release không cần `--features`.
  `--no-default-features` chỉ là fallback compile, không display BRep.
- UI: four-view smoke và face/edge pick smoke tự động pass; manual visual checklist còn mở.
- Annotation/hatch/light host types **chưa implement**; vẫn retained + count/summary.

Đã đóng: layer fingerprint, V5-only gate, node/output budget, userdata opaque projection, BRep
ON_Brep ↔ OCCT, editable display/topology, export `Brep` → `NativeBrep`, archive string admission
cho BRep source >16 KiB.

Chưa đóng:

- R5-A fallback: public openNURBS không có implementation `ON_Brep::CreateMesh`; feature-off
  `NativeBrep` không có cached mesh phát `BREP_DISPLAY_UNAVAILABLE`, không còn claim editable/visible.
- Manual UI 4 view/select chưa chạy; extended V5 classes (annotation/hatch/light) mới ở mức retained.
- V5 import hiện **strict**: file có BRep không convert exact sẽ abort; best-effort chưa bật.
- V6–V8 trả deferred; V4 legacy chưa làm.

## 0b. Định nghĩa "đủ object"


Có hai mức, cần chốt trước khi code:

| Mức | Kết quả | Chi phí | Khuyến nghị |
|---|---|---|---|
| **A. Đủ object nhìn thấy/select/export được** | 44/44 native object có host entity; BRep vẫn là `NativeBrep` opaque, có display mesh để vẽ; sửa topology BRep vẫn chưa có | Trung bình, không cần OCCT | Fallback feature-off; không còn claim hiển thị BRep |
| **B. Đủ object chỉnh sửa hình học** | BRep thành `Geometry::Brep` thật trong OCCT, trim/surface/boolean dùng được | Lớn, phụ thuộc `OM9-L2-01-05` + M00-OCCT chưa duyệt | **Đã chọn cho release Rhino 5** |

Kế hoạch dưới đây là ladder A → B. R5-B đã được triển khai và đo bằng file thật; R5-A vẫn là fallback
cho build không có OCCT, nhưng hiện không tự tessellate được BRep không có cached mesh.

## 1. Bằng chứng đo được (chẩn đoán 2026-10-20)

File thử:

- đường dẫn: `C:\Users\nguye\Downloads\bong 5X7.3dm` (không commit file người dùng)
- SHA-256: `573bb8b0b1523424c95cf0dfa2b26500c71e5add06b57462421a2ffc697f1218`
- 2.643.916 byte; archive version **50** (Rhino 5); openNURBS version `201705225`
- 34 layer, 44 object, 35 material, 5 line pattern, 8 group, 5 user table
- object: 30 `ON_Brep`, 4 `ON_NurbsCurve`, 4 `ON_LineCurve`, 3 `ON_PolyCurve`, 2 `ON_ArcCurve`,
  1 `ON_InstanceRef`

Các kết quả đã đo:

1. `--inventory-source` đọc **toàn bộ** file thành công, 0 CRC/recoverable/critical error.
2. `--project-source` (đường import thật) **abort ở bảng layer** với:
   `source classification serialization diagnostics: ON_Layer`.
3. Nguyên nhân: mọi layer có render material index `0..33`; `layer_preservation` chuẩn hóa layer
   nhưng không reset material/linetype index; scratch archive không có manifest nên openNURBS báo
   `unable to convert model index to archive index`; `record_fingerprint` coi mọi error mới là fatal.
4. Khi vá 2 dòng reset `native_material_index`/`native_line_pattern_index`, projection đọc đủ
   **34 layer / 44 object / 89 component**, `complete_read=true`, 0 diagnostic.
5. 29/44 object bị `retained_only` **chỉ vì** `unsupported_fields = ["Geometry.userdata"]`:
   19 BRep (1 là definition member), 4 NURBS, 2 polycurve, 2 arc, 1 line, 1 instance ref.
6. Khi cho phép project các object chỉ có userdata opaque:
   - projection đạt 44/44 object;
   - full import CLI exit 0, `.om9` 7.888.263 byte, 34 layer;
   - host document: **43 root entity** (29 `NativeBrep` + 13 `Curve` + 1 `Instance`) và **1 definition
     member** nằm trong block definition; tổng 44 native object đều có host binding;
   - native projection output 5.378.318 byte và **1.705.752 JSON node** (vượt native limit 262.144,
     nhưng dưới Rust `Limits::max_nodes = 2.000.000`).
7. **29/29 `NativeBrep` có display mesh rỗng** trong file này. Nghĩa là kể cả khi host entity tồn tại,
   BRep sẽ không vẽ được nếu không bổ sung tessellation.
8. Export document đủ 44 object hiện fail `invalid native geometry correspondence`; document 15 object
   (không có block) export được 136.356 byte. Vì vậy export là gate riêng, không được bỏ qua khi báo xong.

## 2. Phạm vi và không phạm vi

### Làm

- Sửa blocker layer fingerprint.
- Phân loại `Geometry.userdata` là **metadata opaque**, không phải lý do chặn projection hình học.
- Cho 44/44 object vào host document ở `ImportMode::Geometry`, vẫn giữ nguyên source bytes.
- Nâng budget/transport cho inventory 44 object một cách có đo đạc, không mở vô hạn.
- Sinh display mesh cho `NativeBrep` từ openNURBS để 29 BRep hiện được trong 4 view.
- Xử lý block definition/instance và round-trip export của document đủ object.
- Test tự động + smoke UI + ghi evidence/ledger.

### Không làm

- Không chuyển `NativeBrep` thành OCCT `Geometry::Brep` editable; không làm trim/boolean/solid edit.
- Không khôi phục userdata proprietary của Matrix/Clayoo vào host entity; chỉ giữ trong retained source
  và báo loss rõ ràng (đúng exclusion của `OM9-L2-01-13`).
- Không material/linetype/dimstyle table vào host document.
- Không commit file `bong 5X7.3dm` vào repo; chỉ dùng làm fixture local có SHA/provenance.

## 3. Thứ tự lát cắt

Mỗi slice phải có test RED trước, GREEN sau, và không slice sau được che lỗi slice trước.

### S0 — Fixture và test tái lập (RED)

**Mục tiêu:** biến các lỗi đã đo thành test thất bại có chủ đích, không phụ thuộc file người dùng.

Việc làm:

1. Tạo fixture tổng hợp bằng openNURBS (trong `native/`, không commit file riêng):
   - `layers_with_material.3dm`: 2 layer, mỗi layer có render material index;
   - `brep_with_userdata.3dm`: 1 BRep có userdata, không có cached render mesh;
   - `block_instance.3dm`: 1 definition + 1 instance, member có transform;
   - `mixed_jewelry.3dm`: nhiều BRep/curve + 1 block, đủ để vượt budget cũ.
2. Thêm test native-level (qua `om9-exchange-native --project-source`) và Rust e2e:
   - `crates/om9-app/tests/three_dm_full_object.rs` (ignored, nhận `OM9_THREE_DM_FIXTURE`);
   - test DTO/budget ở `crates/om9-exchange/tests/` nếu cần.
3. Ghi rõ test nào đang RED và lỗi mong đợi.

**Exit:** có ít nhất 4 test RED tương ứng: layer material, userdata object, node budget, empty BRep display.

### S1 — Vá blocker layer (P0, bắt buộc trước mọi thứ)

**File:** `native/native_exchange.cpp`, `layer_preservation` (`#L795-L803`).

Thay đổi tối thiểu:

```cpp
auto normalized = layer;
normalized.SetId(defaults.Id());
normalized.SetIndex(defaults.Index());
normalized.SetName(defaults.Name());
normalized.SetParentLayerId(defaults.ParentLayerId());
normalized.SetVisible(defaults.IsVisible());
normalized.SetLocked(defaults.IsLocked());
normalized.SetColor(defaults.Color());
// Fingerprint is written to a scratch archive with no component tables/manifest.
normalized.SetRenderMaterialIndex(ON_UNSET_INT_INDEX);
normalized.SetLinetypeIndex(ON_UNSET_INT_INDEX);
```

Không đổi `layer_values`: material/linetype index vẫn được lưu riêng, nên preservation không mất dữ liệu.

**Test:** fixture `layers_with_material.3dm` import thành công; file thật `--project-source` qua bảng layer.

**Exit:** không còn `source classification serialization diagnostics: ON_Layer`; các test layer cũ vẫn xanh.

### S2 — Phân loại userdata và project đủ object (P1)

**Vấn đề:** native đang dùng một điều kiện duy nhất `item["unsupported_fields"].empty()` để quyết định
project hình học (`native_exchange.cpp#L987`, `#L995`). Điều này trộn hai loại khác nhau:

- **metadata opaque**: `Geometry.userdata`, `Attributes.userdata` — không làm thay đổi hình học;
- **geometric capability bị thiếu**: `Attributes.page_space`, `Mesh.ngons`, `Mesh.face_normals`,
  `Mesh.texture_coordinates`, `Mesh.surface_parameters`, `Mesh.curvatures`, `Cloud.scalar_values`...

Thiết kế:

1. Tách predicate trong native, ví dụ `opaque_fields_only(item)`:
   - trả true nếu mọi unsupported field thuộc allowlist `{Geometry.userdata}`;
   - `Attributes.page_space` **không** thuộc allowlist (page-space object không phải model geometry);
   - bất kỳ mesh channel/cloud scalar/unknown field nào cũng **không** thuộc allowlist.
2. Điều kiện project thành:
   `g && type==ModelGeometry && (unsupported_fields.empty() || opaque_fields_only(item))`.
3. Vẫn giữ `Geometry.userdata` trong `unsupported_fields` để:
   - `ImportMode::Preserve` tiếp tục giữ object ở `retained_only` (đúng policy);
   - `ImportMode::Geometry` được phép tạo host entity nhưng phải phát loss diagnostic.
4. Rust `eligible_object` (`crates/om9-exchange/src/imported.rs#L639-L654`) hiện đã cho Geometry mode
   bỏ qua `unsupported_fields`; cần siết lại:
   - Geometry mode chỉ bỏ qua **opaque allowlist**;
   - nếu còn unsupported field hình học thì vẫn không tạo entity, hoặc tạo nhưng phải có diagnostic
     `unsupported_geometry_channel` — chọn "không tạo" để tránh tam giác thiếu normal/UV/ngon.
5. Thêm diagnostic loss cho `unsupported_fields` (hiện `loss()` chỉ đọc `preservation.nondefault_fields`,
   nên `Geometry.userdata` không xuất hiện trong báo cáo):
   `"{id} (object): geometry import omits native fields Geometry.userdata; original bytes remain retained."`

**Kết quả mong đợi:** 44/44 native object được bind; 43 root entity (29 `NativeBrep` + 13 `Curve` +
1 `Instance`) và 1 definition member trong block; 0 object `retained_only` vì hình học; mỗi object có
userdata có diagnostic loss.

**Test:** fixture `brep_with_userdata.3dm`; file thật; test Preserve mode vẫn giữ object đó retained.

**Exit:** native projection 44/44; Rust import không còn `retained-only` cho geometry; diagnostics nêu rõ
userdata bị bỏ; không silent loss.

### S3 — Budget/transport cho inventory nhiều object (P1)

**Vấn đề:** `max_nodes = 262144` (`native_exchange.cpp#L28`) dùng chung cho packet đầu vào và output tree.
File này cần 1.705.752 node. Output 5.38 MB, vẫn dưới stdout budget 8 MB nhưng margin mỏng.

Thiết kế:

1. Tách constant:
   - `max_packet_nodes = 262144` (input JSON từ Rust, giữ nguyên hardening);
   - `max_inventory_nodes = 4'000'000` (output tree, có đo đạc);
   - `max_inventory_bytes = 16 * 1024 * 1024` (stdout; hiện 8 MB).
2. `source_output_tree` dùng `max_inventory_nodes`; `packet()` vẫn dùng `max_packet_nodes`.
3. Ghi số node thực vào response (`"inventory_nodes"`) để log/ledger có số đo, không đoán.
4. Rust:
   - `Limits::max_nodes` mặc định 2.000.000 hiện đủ cho 1.705.752; **không nâng toàn cục** nếu không cần;
   - nếu file thật vượt 2M, trả `Budget` rõ ràng thay vì cắt object;
   - kiểm tra `response_limits` cho 5.4 MB.
5. Đo peak memory của native + Rust trong lúc decode; ghi vào evidence.

**Exit:** file thật `--project-source` trả 44 object, 1.7M node, không budget error; file lớn hơn trả lỗi
budget có thông điệp; input packet vẫn bị giới hạn 262k.

**Backlog (chưa làm trong lát cắt này):** nếu cần file >2M node, chuyển `NativeBrep.payload` từ mảng byte
JSON sang base64/hex hoặc binary sidecar; khi đó giảm node ~50× và không phải nâng limit. Không trộn vào S3
vì sẽ đổi DTO/serde và test round-trip.

### S4 — Display mesh cho BRep opaque (P1, điều kiện để "thấy" object)

**Vấn đề:** `native_brep_json` (`native_exchange.cpp#L491-L513`) chỉ lấy cached mesh của **một face đầu**;
file này không có cached mesh nên 29/29 display rỗng.

Thiết kế:

1. Hàm `brep_display_mesh(const ON_Brep&, ...)`:
   - thu tất cả mesh cached của các face (`ON::any_mesh`/`render_mesh`/`analysis_mesh`);
   - nếu không có, gọi `brep.CreateMesh(mp, meshes)` với `ON_MeshParameters` kiểu fast render,
     tolerance lấy từ source absolute tolerance và clamp theo budget;
   - merge nhiều `ON_Mesh` thành một `Mesh` duy nhất: vertices/faces offset, normals/colors theo đúng
     channel; bỏ quad thành 2 tam giác hoặc giữ polygon như schema hiện tại;
   - giới hạn tổng vertex/face theo `max_points`/`max_array`; lỗi/timeout → display rỗng + diagnostic
     `"BRep display tessellation unavailable"`, không làm fail import.
2. `native_brep_json` dùng hàm này; payload/CRC không đổi.
3. Unit: display phải được scale về canonical mm giống curve/mesh. Hiện `geometry_to_mm`
   (`native_exchange.cpp#L831-L837`) không xử lý `NativeBrep`; thêm nhánh chỉ scale `display`,
   **không** scale payload (payload là archive gốc, dùng cho export). Với file đơn vị mm, scale = 1;
   với file inch, đây là bug cần test riêng.
4. Rust không đổi schema; `NativeBrep.display` đã có trong `om9-core/src/geometry.rs#L528-L571`.

**Test:**
- fixture `brep_with_userdata.3dm` không cached mesh → display vertices/faces > 0;
- file thật → 29/29 `NativeBrep.display` khác rỗng;
- test scene batch có vertex count > 0;
- test negative: BRep hỏng/không tessellate được → entity vẫn vào, diagnostic có, không panic.

**Exit:** 4 view hiển thị đủ 29 BRep + 13 curve + block instance; selection/hit-test hoạt động; payload vẫn round-trip.

### S5 — Definition/instance và export round-trip (P2)

**Vấn đề:** full import đủ object tạo 1 block definition + 1 instance; export hiện fail
`invalid native geometry correspondence`. Document 15 object không block export được, nên nghi vấn
chính là definition/instance hoặc một trong 29 geometry mới.

Việc làm:

1. Tạo fixture tối thiểu 1 definition + 1 instance; chạy export → native `--inspect` → reimport.
2. Khoanh vùng lỗi:
   - `inspection.rs#L489-L490` so `defs.len()` với `document.definitions.len()`;
   - `inspection.rs#L256-L278` so `NativeBrep` payload length/CRC;
   - `retained_packet.rs`/`current_overlay.rs` đường `PatchDefinition`/`DefinitionMember`.
3. Nếu writer thiếu definition/member: bổ sung packet/merge; nếu inspection kỳ vọng sai: sửa oracle,
   nhưng phải chứng minh bằng native inspect, không nới kiểm tra để test xanh.
4. Xác nhận userdata của object không được ghi vào host/export (loss diagnostic), nhưng payload BRep
   và layer/transform vẫn đúng.

**Exit:** full document export thành công; native inspect thấy đủ object/definition; reimport ổn định;
không mất BRep payload; test partial 15 object vẫn xanh.

Nếu S5 vượt quá phạm vi import, tách thành gate riêng và **không báo "đủ object"** khi export còn fail;
khi đó ghi rõ "import đủ để xem, export block instance chưa xong".

### S6 — Acceptance, UI smoke, ledger

1. Automated:
   - `cargo test -p om9-exchange`, `-p om9-app`, `-p om9-core`, `-p om9-3dm` (theo phạm vi);
   - `cargo clippy --workspace --all-targets -j 2 -- -D warnings`;
   - build native bằng `tools/build-native-exchange.ps1`, cập nhật SHA/manifest;
   - chạy e2e ignored với `OM9_THREE_DM_FIXTURE` trỏ file thật.
2. UI smoke (Windows, app thật):
   - File > Import 3DM chọn file;
   - Layers panel: 34 layer, tên/màu/visible/locked đúng;
   - scene: 43 root object (29 BRep + 13 curve + 1 instance) và block definition/member; 29 BRep hiện;
   - 4 view, zoom/pan/orbit, select BRep/curve/instance, object info;
   - không treo, không mất document cũ, Undo một entry.
3. Evidence/ledger:
   - lưu log lệnh, SHA file, native SHA, số object/layer/node/display;
   - cập nhật `OM9-L2-01-13` runtime_implementation và `docs/openmatrix9-progress.json`;
   - không commit file người dùng; ghi provenance/SHA theo `docs/private-reference-policy.md`.

## 4. Test matrix

| # | Case | Kỳ vọng | Slice |
|---|---|---|---|
| 1 | Layer có material index | import qua layer, preservation đủ | S1 |
| 2 | BRep có userdata | host `NativeBrep`, diagnostic userdata loss | S2 |
| 3 | File thật 44 object | 44/44 bind; 43 root + 1 definition member | S2/S3 |
| 4 | 1.7M node output | decode OK; không budget error | S3 |
| 5 | BRep không cached mesh | display > 0; 29/29 hiện | S4 |
| 6 | Block definition + instance | 1 definition, transform đúng | S5 |
| 7 | Export full document | native inspect đủ object/def; reimport ổn định | S5 |
| 8 | Preserve mode + userdata | object vẫn retained, không host giả | S2 |
| 9 | Mesh ngon/UV/normal thiếu | không tạo host entity thiếu channel; diagnostic | S2 |
| 10 | Page-space object | không đưa vào model scene; diagnostic | S2 |
| 11 | File hỏng/truncated | atomic rollback, document/Undo không đổi | S1/S2 |
| 12 | Budget vượt | lỗi Budget rõ, không cắt object im lặng | S3 |

## 5. Rủi ro và cách chặn

| Rủi ro | Mức | Giảm thiểu |
|---|---|---|
| `ON_Brep::CreateMesh` chậm/không thread-safe | Trung bình | single-thread native; fast mesh params; budget; fallback display rỗng + diagnostic |
| Tăng node/output budget làm tăng RAM/DoS | Trung bình | tách input/output limit; cap 4M/16MB; đo peak; Rust admission vẫn 2M |
| Userdata chứa ngữ nghĩa plugin, bỏ đi làm sai hành vi | Cao | chỉ bỏ cho host Geometry; giữ source bytes; loss diagnostic; Preserve mode giữ retained |
| Export block instance là lỗi sâu trong retained merge | Cao | fixture tối thiểu; tách gate; không tuyên bố hoàn tất nếu export fail |
| BRep payload không round-trip byte-for-byte | Trung bình | so CRC + native inspect + reimport; không chỉ so length |
| Đơn vị source khác mm | Trung bình | scale display; payload giữ đơn vị gốc; test file inch riêng |
| File lớn hơn 2M node | Thấp (file này) | backlog base64/binary payload; lỗi Budget rõ |

## 6. Thứ tự thực hiện đề nghị

1. **S0** — fixture + RED tests.
2. **S1** — layer fix; đây là blocker tuyệt đối.
3. **S2** — userdata classification + loss diagnostics.
4. **S3** — tách budget, đo 44 object.
5. **S4** — display tessellation (để thấy BRep).
6. **S5** — definition/instance + export.
7. **S6** — acceptance/UI/ledger.

S1+S2+S3 là điều kiện tối thiểu để **host document có đủ 44 object**. S4 là điều kiện để **nhìn thấy**
29 BRep. S5 là điều kiện để **lưu/export lại** document đủ object.

## 7. Câu hỏi cần chốt

1. Chọn mức **A** (đủ object nhìn thấy, BRep opaque) hay mức **B** (BRep editable, cần OCCT)?
   Kế hoạch này giả định **A**.
2. Export full document có phải điều kiện hoàn tất của đợt này, hay chỉ cần import đủ để xem?
3. Có chấp nhận bỏ userdata proprietary khỏi host entity (giữ trong retained source + báo loss) không?
   Đây là exclusion đã ghi trong `OM9-L2-01-13`.

## 8. Tài liệu tham chiếu

- `OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L2.json`
- `.../L3-006 - 3dm NURBS Surface And Trimmed B-Rep/l3.json`
- `.../L3-007 - 3dm Mesh And Instance Blocks/l3.json`
- `.../L3-008 - 3dm Unsupported Proprietary Object Policy/l3.json`
- `implementation/design/M01/THREEDM-READINESS.md`
- `implementation/design/M01/THREE-DM-HOST-ADAPTER.md`
- `docs/openmatrix9-progress.json` — record `three-dm-native-host-2026-10-19`

## 9. Trạng thái bằng chứng

- Đã đo: inventory/projection/import chẩn đoán, số layer/object/node/display, lỗi export.
- Chưa chạy: bất kỳ test tự động mới nào, UI smoke, build native chính thức sau khi sửa.
- Không được coi tài liệu này là acceptance; chỉ là kế hoạch thi công.




































