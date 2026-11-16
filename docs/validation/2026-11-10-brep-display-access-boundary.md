# B-rep display access boundary

`Brep.display` is now private. Workspace consumers use `display()` to borrow
the existing `Arc<BrepDisplay>`, or `into_display()` to transfer it when
consuming the B-rep. Deliberately corrupted validation fixtures use the
hidden `display_mut_for_validation()` escape hatch. Existing display sharing,
serialization and ownership are unchanged. This is a Rust API change for
external callers that accessed the public field directly.

This is a preparatory boundary, not an eviction mechanism: `display()` still
returns a borrowed strong owner and the validation escape hatch still permits
mutation. A reclaimable implementation must replace this interface with an
admitted, explicitly pinned/fallible access path and migrate its callers; it
must also integrate source/topology proof, history restoration and v4 fallback.

Validation: 238 app integration targets compiled with default features, with
`curve_shape_audit` excluded because its unchanged fixture references missing
`controller` and `CURVE_MENU` symbols. All targets of the remaining workspace
packages compiled. App no-default check passed. Executed tests: core B-rep 23;
scene unit 34; kernel topology 14 passed and 3 ignored; app topology remesh 28
passed and 1 ignored; presentation snapshots 15; B-rep picking 20; Sphere and
Gumball 2. Public-source audit and `git diff --check` passed. Build target was
`L:/om9-target`, with `-j 2`. No native/process peak or desktop validation was
performed.
