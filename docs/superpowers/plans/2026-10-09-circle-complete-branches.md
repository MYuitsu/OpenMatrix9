# Circle nonplanar tangencies, additional History and active layers

User explicitly requested these three extensions after the validated spatial slice.
Source reread: Matrix8 Book1 PDF138–140, Circle continuation before Ellipse.

- Safe Rust owns a bounded spatial tangency optimizer, validations, candidate
  ranking and explicit ambiguity selection. Exact native curve callbacks return
  position/tangent at normalized bounded parameters synchronously on the GUI
  thread; Rust never retains native pointers. Planar OCCT behavior is retained.
  Nonplanar curve contacts must satisfy circle plane/radius/tangent equations;
  infeasible skew lines reject. Root search is bounded, not exhaustive/certified.
- Safe Rust replays versioned Circle recipes including Deformable output.
  Native History stores explicit vertex/curve/selection links, source snapshots,
  parameter fractions and model properties; typed coordinates remain constants.
  `Point=Object.VertexN` / `Direction=Object.VertexN` provide associativity.
  Record/Update/Lock, deletion, invalid recovery, parent changes, detach,
  Undo/Redo and cold restore extend the proven native framework.
- Rust owns active layer parsing/validation/default records. Native plain groups
  persist membership/state/style with no placement transforms or cycles. Until
  the user selects a layer, previous root green output is retained. Locked active
  layers reject before commit; layer assignment and Circle creation share a
  transaction. Existing import metadata is preserved.
- Independent implementations have file ownership; main integrates command/FFI,
  builds once against matching SDK and runs native suites sequentially. Preserve
  unrelated concurrent changes, original contract and guides. Only verified
  slices enter spec_v1 catalogs/status/checklist/manifest and the ledger.
