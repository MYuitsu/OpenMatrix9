# OM9-TRANSFORM-041 — Create Cage

Source: Spec v1 `specs/05-transform/om9-transform-041-create-cage.md`.

Implemented slice: whole native geometry defines a World BoundingBox with
positive extents; explicit OM9 2/2/2 counts and 1/1/1 degrees create one
independent editable native CageControl. Creating a cage does not capture its
bounding-box sources. Other coordinate/base modes remain unsupported.

The [native Cage command contract](cage-command-native-contract.md) records
stable routes, input roles, defaults, ownership and limits. Rust tests pass;
the matching SDK command fixture passes 27/27 checks.
Status: partially_implemented; validated_supported_slice.
