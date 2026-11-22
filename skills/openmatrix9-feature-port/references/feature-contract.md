# Normalize the selected feature

Use the exact L1/L2/L3/L4 ID from `OM9_Milestones`. Resolve it with the workflow router, read its parent owner and exact milestone contract, then implementation/runtime evidence. Source/provenance guidance comes from root AGENTS, the selected source anchors and `docs/private-reference-policy.md`; proposed APIs in imported documents are not implemented APIs.

Record the following in `docs/features/<ID>.md`, derived separately from the original package:

| Contract field | Required evidence/decision |
|---|---|
| Identity | Exact ID/name/command/domain/spec path; source manual and printed/PDF page range, including command continuation pages up to the next feature heading |
| Evidence | Quote or summarize narrowly with file/page/line; separate inference and design choice |
| Inputs | Supported object types, selection order, preconditions, empty/invalid selection |
| Parameters | Meaning, units, ranges, default values and why supported; explicit `TODO_EVIDENCE` for missing facts |
| Outputs | Geometry/object type, topology, ownership, placement, orientation and dependencies |
| Lifecycle | Invocation and applicable preview/update/commit/cancel; do not assume every command is a builder |
| History | Applicable dependency recomputation, stable IDs and update propagation |
| Errors | Failed operations, no-op/cancel, cleanup and useful user feedback |
| Undo/save | Transaction boundary, undo/redo, document serialization and reload |
| Validation | Geometry/state invariants, fixture/manual comparison, tolerances and live standalone OM9/native adapter checks |
| Continuation | Current verified status, unresolved question and smallest next document/function |

Builder preview geometry must be separated from committed document geometry. Cancellation must leave the document in the intended pre-command state. If the original lifecycle is unproven, distinguish a chosen host interaction design from source evidence.

Source dependency cues such as “History”, “Mesh” and “Builder” are detected terms. Verify actual dependencies before choosing architecture or marking a framework applicable.

## Conditional reference reads

| Condition | Read next |
|---|---|
| Interactive builder confirmed | `specs/01-core/om9-buildercore-001-builder-framework.md` |
| Source or behavior uses History | `specs/01-core/om9-history-001-history-workflow.md` |
| Styles presets are part of selected feature | `specs/01-core/om9-style-001-styles-framework.md` |
| Clayoo feature | `specs/14-subd/README.md`, then exact ID spec |
| T-Splines feature | `specs/06-tsplines/README.md`, then exact ID spec |
| Original code resolves a missing behavior | Native mapping skill, selected container/event/VA |

## Example: Gem Loader

Helper `feature OM9-GEM-001` resolves `gvLoader` and its v2 spec. Historical source mapping places Gem Loader at Matrix8 Book2 printed402/PDF176; this manual/page is provenance, not a required field of the v2 catalog.

The supported workflow cues select the loader, shape and size. Cut Type, Keep Original Size, Gem Shape, Gem Size, On Surface and Custom Gem Sizes are option names. They do not supply values, units, placement rules or cancellation behavior. The catalog marks it `kind: command`; a detected Builder mention does not change that fact.

The historical Gem Loader section spans PDF176–177/printed402–403. For an unresolved parameter/UI question read that bounded range; v2 normalized semantics do not require a PDF reread. If implementing On Surface, resolve the separate OM9-GEM-011 spec rather than treating the branch as fully described here.

The package intentionally excludes its PDFs. Local files discovered during retrieval testing are registered in `docs/openmatrix9-reference-locations.json`; validate paths when used and update the project registry if files move. Resolve the current private paths from the registry; old Downloads/ref observations are not defaults. If a cited file is absent, record the exact missing page; do not invent values to claim fidelity. Menu wiring, feature identity and independent infrastructure may proceed without that page.

### Gem layer palette

Quy tắc OM9 được duyệt ngày 2026-11-22. Khi tạo gem theo mặc định, tra nhóm đá trong tài liệu hiện tại theo hình dạng, kiểu cắt, vật liệu và kích thước dài × rộng danh nghĩa (làm tròn tới 0,001 mm). Gem cùng nhóm dùng lại layer và màu của nhóm, kể cả sau khi mở lại tài liệu.

Bốn nhóm đầu lần lượt dùng Gem 01 `#003CA0`, Gem 02 `#4178CD`, Gem 03 `#82B4FA` và Gem 04 `#AFD0FF`. Nhóm mới tiếp theo dùng User 01, rồi User 02 và các layer User tiếp theo. Ví dụ, khi tạo Round Diamond cỡ 2, 3, 4, 5, 6, 2 mm trong tài liệu mới, hai viên 2 mm đều thuộc Gem 01 và viên 6 mm thuộc User 01.

Khi kiểm tra nét vẽ của viên đá, xem [Wireframe Round Diamond](../../openmatrix9-workspace-contract/references/grid-and-viewports.md#round-diamond-wireframe). Bằng chứng trong checkout OM9 nằm ở `docs/validation/2026-11-22-round-wire-palette.md`.

### Gem Loader size catalog

Quy tắc người dùng duyệt ngày 2026-11-22 cho `OM9-GEM-001` và `OM9-L3-04-09-003`.

- Dùng bảng cỡ Matrix9 gốc của từng cặp **Cut–Shape** và chế độ đo. Giữ nguyên từng số đo theo mm và thứ tự gốc, kể cả các cỡ cùng chiều dài với chiều rộng khác nhau. Triangle và Trillion dùng bảng riêng khi bật chế độ đo đỉnh–đỉnh ở những biến thể có hỗ trợ.
- Với bảng có `N` cỡ, dùng 1 tab khi `N ≤ 22`, 2 tab khi `23 ≤ N ≤ 44`, và 3 tab khi `N ≥ 45`. Chỉ số đầu là `0`, chỉ số cuối là `N − 1`. Với `P` tab, mỗi mốc trong bảng là `k × N / P` với `1 ≤ k < P`, làm tròn đến số nguyên gần nhất và làm tròn về số chẵn ở đúng điểm giữa như VB6. Mỗi tab lấy cả hai đầu của đoạn chỉ số. Cỡ cuối tab trước lặp lại thành cỡ đầu tab sau.
- Tính **Depth %** bằng `100 × Depth / số đo trục quy chiếu`. Trục quy chiếu là đường kính của Round, Width trong chế độ dài–rộng, hoặc số đo đỉnh–đỉnh trong chế độ corner-to-corner. Khi nhập tỷ lệ, tính ngược `Depth = số đo trục quy chiếu × Depth % / 100`. Giữ độ chính xác của preset khi áp dụng và làm tròn theo định dạng hiển thị.
- Kiểm tra số cỡ từng tab, thứ tự và cỡ chung ở ranh giới. Diamond Round có `23 / 22 / 22` cỡ với hai ranh giới Ø 3,40 và Ø 6,80 mm. Diamond Oval có `13 / 11` cỡ với ranh giới 7,20 × 5,20 mm. Oval 6,00 × 4,00 mm dùng Width 4,00 mm để tính độ sâu khoảng 2,4492 mm.

Tra cứu nguồn và phạm vi đã kiểm tại `docs/validation/matrix9-gem-loader-size-evidence.md` và `docs/validation/2026-11-22-dock-popup-theme.md#matrix9-size-catalog-and-tabs` trong checkout.

## Distinct domains

Clay Edit is `OM9-SUBD-001`/`ClayEdit`, not a T-Splines HUD feature. Conversion from T-Splines to Clayoo is a separate command (`OM9-SUBD-043`). Emboss and Matrix Art likewise use separate domain catalogs. Resolve IDs through the helper rather than guessing a filename from a menu label.
