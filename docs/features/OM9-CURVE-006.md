# OM9-CURVE-006 — Ellipse

## Source evidence

Matrix 8 Book 1 PDF pages 140–141 (printed 130–131), resolved through the private reference registry. Source defines center/two semiaxis endpoints, right-click Diameter, Corner, Vertical, FromFoci/MarkFoci, AroundCurve and Deformable with Degree/PointCount. It gives no numeric radius defaults or History requirement. The original engineering contract and Rust request-planner example remain authoritative and unchanged.

## Supported behavior and host choices

Rust owns all portable construction/session/validation. C++ only adapts Qt events, exact OCCT/Part geometry, native edge references and document/layer transactions.

- Center: center, first semiaxis endpoint, second semiaxis endpoint. First axis projects into the latched CPlane; the second size is the perpendicular component. Three-dimensional centers retain their elevation.
- Diameter: first axis start/end then second semiaxis endpoint. Both first-axis vector and center use the same plane projection. Numeric first-axis entry means full diameter.
- Corner: opposite enclosing rectangle corners aligned with CPlane X/Y. Vertical: first axis in CPlane, second along CPlane normal.
- FromFoci: two distinct foci and an off-axis curve point define the spatial ellipse plane. MarkFoci creates two separate native vertex objects at the exact analytic foci.
- AroundCurve: a bounded native `Object.EdgeN`, center picked on that edge or `OnCurve=0..1`; ellipse normal follows its exact tangent. World picks and CPlane typed coordinates remain distinct. Changed/deleted references reject without advancing and permit Undo/reselect.
- Deformable: Rust fits a periodic unit-circle spline then applies the ellipse frame/scales. Degree and pole count are preserved. Persisted `EllipseApproxDeviation` is the maximum Euclidean difference at 1024 corresponding parameter samples, not a certified continuous Hausdorff bound.

Displayed OM9 defaults (not attributed to Matrix): Center, Deformable=false, Degree=3, PointCount=8, MarkFoci=false. Degree 1–11, pole count 4–256 and strictly greater than degree. Positive numeric lengths support the shared input-unit context and explicit unit suffixes. Minimum semiaxis 1e-7 mm; all coordinates and the complete analytic extent stay within +/-1e9 mm. Equal semiaxes produce an exact circle; unequal semiaxes remain an analytic ellipse.

## Native outputs and lifecycle

Exact and deformable outputs are valid closed single-edge `Part::Feature` wires, tagged `OM9FeatureId=OM9-CURVE-006`, `OM9Role=Ellipse`. MarkFoci adds `Focus1` and `Focus2` vertex features with the same feature ID. No document object is created during preview. Invalid input preserves the current picks/options; Undo removes the last pick (or selected path), Cancel clears the session.

Ellipse plus both foci share one transaction and current OM9 layer. Layer lock is checked before creating the transaction; membership/style/visibility follow the existing layer adapter. Creation Undo/Redo and FCStd preserve geometry, identity and membership. Native references are released on completion, cancellation and restart.

## Limits

Outputs are snapshots; Ellipse History is explicitly unsupported. Coincident foci and collinear FromFoci points are rejected because they cannot determine the requested plane. This scope does not accept every Rhino layer/import mapping, automatic topology correspondence, all object-editing/CV operations or the complete original engineering contract.

## Validation

Native acceptance is recorded separately in [the validation report](../validation/2026-10-09-ellipse.md); Rust unit/session checks alone do not count as native proof.
