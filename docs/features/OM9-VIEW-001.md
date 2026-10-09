# OM9-VIEW-001 — Picture Frame

<!-- rhino-core-extended-input:start -->
## Extended numeric input — validated slice

Extended typed point input through existing native geometry; no all-option or all-P0 completion.

[Runtime/source evidence](../validation/2026-10-09-rhino-core-extended-input.md). Native geometry is reused; live constraints, locale and other missing contracts remain open.

<!-- rhino-core-extended-input:end -->

Status: partially_implemented; validated_supported_slice (2026-10-09).

**Supported slice:** decoded image placement as a native `Image::ImagePlane`, with aspect-preserving dimensions, transient preview and owned coordinate/unit provenance. Explicit embedding/external-link and alpha-display modes remain open.

### Inputs

Select a readable image file, then enter/pick a first corner and either a width-reference point or scalar width. Existing geometry selection is not required. The shared Rust parser supports CPlane/world/relative Cartesian points, explicit coordinate/length units and the captured document input scale. A scalar is accepted only in the width phase and remains a tagged length in provenance. Typed `Length=` is not registered as an option; width is entered directly. Native picks/snaps/F4 are world-coordinate inputs.

### Parameters and defaults

Each session starts with `Vertical=No`, `SelfIllumination=Yes` and `Autoname=Yes`; these options accept case-insensitive Yes/No or 1/0. Vertical uses CPlane Z for image height. SelfIllumination selects the native No shading/Shading display mode. Autoname changes the visible Label to the source filename stem; the internal object Name remains a unique PictureFrame name, so the full filename-as-object-name contract is not completed. Width and derived height must be in `[1e-7, 1e9]` mm, with finite resulting corners within ±`1e9` mm. Scalar width follows captured CPlane X; a reference point determines width/direction in the captured plane. Mouse Shift/F8 uses the existing Ortho path. The prompt does not yet present all current option values.

`EmbedBitmap` mode selection, external-path missing status, and `AlphaTrasnparency` Object-color/Transparent selection are not implemented. Native image storage or an RGBA texture alone does not establish those option contracts. Preserve the exact compatibility spelling `AlphaTrasnparency`.

### Output

Creation produces one document-owned `Image::ImagePlane` with image file, X/Y sizes and centered Placement. It is a textured reference object, not traced geometry or a Part BRep. The same creation transaction stores `OM9CoordinateReport` and `OM9UnitContext`. Native placement and size properties remain editable after creation. Separate-process reload after removal of the external fixture verifies actual native image storage; it does not establish an EmbedBitmap Yes/No control.

### Preview / commit / cancel

The document owner, units and CPlane are captured before the image dialog and checked again after it. Placement uses that command-start frame. Hover preview is transient and does not append accepted inputs or consume Once; rejected width/reference values retain the accepted corner and options. The final candidate report is validated before mutation. A rejected transaction start fails before adding an image; successful creation places the image and provenance in one transaction. Cancel clears preview and pending inputs without creating an object. Document switch/close, workbench cancellation or stale units invalidate placement.

### History / dependency model

Native Undo/Redo governs image creation and its provenance together. The persisted report is an owned record of the accepted point/point or point/length inputs and their captured context. It does not establish associative History links to snapped objects or a feature replay graph. Undo/Redo, same-process restore and separate-process reload passed against the final tested module.

### Error / invalid-input behavior

Unreadable images, malformed/non-finite/overflowing input, degenerate width, unsupported dimensions and out-of-range corners fail without accepting the candidate. Native mutation/recompute failures abort an opened transaction and do not record command success. The final native replay exercises locked-transaction rejection and retry. Forced post-add property/recompute faults remain unexercised. Full EmbedBitmap/external missing, alpha-mode, option-prompt and object-name semantics remain outside this validated slice.

[Validation and remaining scope](../validation/2026-10-09-rhino-core-point-consumers.md).
