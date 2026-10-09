# OM9-TRANSFORM-022 — ReleaseFromCage

Source: Spec v1 `specs/05-transform/om9-transform-022-releasefromcage.md`.

Implemented slice: Enter/OK releases only the selected whole captives through
the native service's Undo transaction. Current geometry, controls and unrelated
captives are preserved. Release creates no substitute geometry.

The [native Cage command contract](cage-command-native-contract.md) records
stable routes, input roles, ownership and limits. Rust tests pass;
the matching SDK command fixture passes 27/27 checks.
Status: partially_implemented; validated_supported_slice.
