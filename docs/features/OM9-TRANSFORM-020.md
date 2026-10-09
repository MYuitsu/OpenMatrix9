# OM9-TRANSFORM-020 — Cage Edit

Source: Spec v1 `specs/05-transform/om9-transform-020-cage-edit.md`.

Implemented slice: select whole captives, then one existing native 3D cage;
Global/Local and finite nonnegative falloff create persistent native bindings.
The binding updates independently of global History Record/Update. An explicit
Restore choice routes retained 3DM records through the verified restore service.
Line/surface controls and Accurate/Fast/refit modes remain unsupported.

The [native Cage command contract](cage-command-native-contract.md) records
stable routes, input roles, ownership, stale checks and limits. Rust tests pass;
the matching SDK command fixture passes 27/27 checks.
Status: partially_implemented; validated_supported_slice.
