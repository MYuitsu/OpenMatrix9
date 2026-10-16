# M06/E11 — Contract notes for GPT-6 Luna (proposed, not source-verified)

**Nhãn pháp lý/kỹ thuật:** Tài liệu này là **design proposal** để triển khai Rust application trên OCCT native kernel. Không khẳng định type, module, file hoặc ABI nào sau đây đã tồn tại trong QymCAD. Thực tế phải đọc source checkout, đối chiếu giấy phép và `planning/M00` trước khi áp dụng.

## 1. Ownership và module seam

| Đơn vị | Chủ dữ liệu | Được làm | Không được làm |
|---|---|---|---|
| E10 gem placement, `OM9-L2-04-09`/`04-10` | E10 | `GemId`, geometry, ordered gem rows, transformations | M06 không sao chép gem loader/row layout |
| `OM9-L2-04-14` | M06 Head/Bezel | Head/Bezel settings parameters, girdle/seat attachment, geometry validity & update | Không reimplement kernel/feature graph, không lấy Clayoo/TSpline |
| `OM9-L2-04-15` | M06 Prong/Metal/Bead | Prong settings, guide curves, metal piece, bead placement, setting mass request | Không copy head/bezels sang module này |
| Kernel/OCCT bridge từ E01–E08 | Existing shared services | Solid/curve creation, boolean, topology validity, serialized authoritative BRep | Không tính hình học kernel trong UI |
| E09 history/transactions | Existing shared services | Feature dependency, undo/redo, deterministic recompute | Không tạo transaction engine song song |
| E12 cutters | E12 | Manufacturing cutters, STL, production validation | M06 `Metal Piece > Top` chỉ tạo kiểu output trung gian theo hợp đồng khi chưa có E12 |

**Rust requirement:** toàn bộ logic ứng dụng, state machine, builder orchestration và test chủ yếu bằng Rust. OCCT là thư viện C++ native được gọi qua *FFI/C ABI tối thiểu* đã được G00 xác nhận. Không tuyên bố mọi dòng bao gồm OCCT đều viết Rust.

## 2. Typed interface — illustrative pseudocode only

Các tên sau **không phải API có thật**. Agent phải map sang types tương đương của checkout; nếu mapping không có thì đề xuất dưới `CONTRACT` trước khi viết code.

```rust
// PSEUDOCODE ONLY — not drop-in buildable; select actual types after source audit.
struct SettingContext {
    source_gem_ids: Vec<GemId>,
    source_row_id: Option<FeatureId>,
    unit_system: UnitSystem,
    tolerance_profile_id: ToleranceProfileId,  // from G00 evidence
}

enum SettingRequest {
    CreateBasketHead { context: SettingContext, params: HeadParams },
    CreateBezel { context: SettingContext, mode: BezelMode, params: BezelParams },
    CreateProngs { context: SettingContext, params: ProngParams },
    CreateMetalPiece { context: SettingContext, params: MetalPieceParams },
    CreateChannelBorder { context: SettingContext, params: BorderParams },
    PlaceBeads { context: SettingContext, placement: BeadPlacement },
    EditSetting { feature_id: FeatureId, patch: ValidatedSettingPatch },
}

enum SettingGeometry {
    ClosedSolids(Vec<ShapeHandle>),
    GuideCurves(Vec<CurveHandle>),
    Instances(Vec<PlacedInstance>),
    CutterProfilePreview(Vec<ShapeHandle>), // not E12 production cutter
}

struct SettingPreview {
    proposed: SettingGeometry,
    source_fingerprint: SourceFingerprint,
    diagnostic: GeometryDiagnostics,
    // Not yet persisted, NO document ID allocation on pointer movement.
}

// All operations: Result<SettingPreview, SettingError>.
// Commit: one validated transaction, existing history API, E10 references preserved.
// Cancel: zero persistent changes, preview disposed, no stale GPU scene handles.
```

## 3. BRep / curve / instances are different

- Head/Bezel/Prong/Metal: BRep must be mathematically valid, have correct units and orientation, and be solid if that feature promises a solid.
- Channel Border and profile library: **curves/data**, not expected to have BRep volume or closed-solid certification. Check curve continuity, self-intersection, topology and serialized references.
- Bead pattern and oriented objects: distribution may be **instances** referencing source solids. Verify geometry only for the *prototype*, and transforms/IDs/spacing for each instance. Materialized BRep assembly may be optional and must not be assumed.
- Output type must remain observable after save/open and recompute. Rendering tessellation is not authoritative geometry. Golden geometry must compare numeric quantities under approved G00 tolerance.
- `Top` mode in Metal Piece references cutter-like geometry in the manual, but implementing E12 cutter subsystem here is out of scope; if required for end-to-end, define a typed handoff contract and keep actual Boolean ownership with shared kernel/E12.

## 4. Atomicity / recompute rules

1. Input references resolved by stable document IDs + topology handles; no direct use of viewport screen coordinates as persistent geometry IDs.
2. Parameter validation happens before destructive OCCT operations. OCCT error/exception is converted into a typed failure; no dangling native handle.
3. Preview builds in transient space; **Escape/Cancel ≡ no document change**. Commit is exactly one Undo entry; Undo/Redo restore parameters and geometry.
4. Gem resize/move/rotation triggers dependent rebuild *or* a visible `CONFLICT`/`UNSUPPORTED` error with the prior valid document intact. Never silently detach.
5. A save/open cycle preserves ids, graph, params, material assignments and unit system. Numeric `volume`, `mass` and BRep health metrics must be invalidated/recomputed when dependencies change.
6. Density source should be versioned and reviewed; when model units are mm and density is g/cm³, mass in g = closed metal solid volume in mm³ × density / 1000. This is dimensional conversion, **not** a numeric density or production weight guarantee. Exclude gems, guide curves, cutter previews and voids from metal volume.
7. Open/invalid solids must not silently produce a trusted weight. The UI must explain why the weight cannot be calculated.

## 5. Fixture planning, no fabricated goldens

Every L3 has placeholders `OM9-M06-FX-14/15-NNN-{VALID,INVALID,UPDATE}` recorded in each `l4.json`. CONTRACT phase must materialize them into real input data with provenance and expected geometry/callback behavior. Suggested coverage (examples, not a test report):

- Round gem, rotated rectangular/girdle gem, non-round profile with corners; mm and foreign units if G00 supports conversion.
- Open gem row (two, three, five gems), closed row, reverse row order, gem reposition, invalid/degenerated gem or row.
- Planar/curved target surface, curve with discontinuities, high instance count, zero/negative thickness, self-intersecting offsets.
- Expected snapshots: BRep validity, face/edge/solid counts if stable, section clearance, frame normal, bounding box and exact volume where appropriate; curve length/orientation and instance transforms for other outputs.

Do not hardcode tolerances in L4. Obtain approved `tolerance_profile_id` and numeric goldens from G00/E10 fixtures, with independent numerical validation.

## 6. Agent stop rules

- If source workspace, AGENTS.md, active Rust toolchain, license decision, OCCT binding, E10 gem contract, or numeric tolerances are unavailable: **STOP** the runtime part, record one precise blocker and return next unblock action.
- Do not mark status `DONE` from doc generation or from tests that were never executed.
- If a BE task requires major unrelated kernel changes, propose a new kernel-owner issue; don't introduce duplicate code into M06.
- FEBE/QA `DONE` requires human manual acceptance of actual executable. Planning-only validator verifies **schema and DAG**, not application behavior.
