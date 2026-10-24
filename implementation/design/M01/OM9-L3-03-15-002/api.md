# OM9-L3-03-15-002 — Matrix File Import Export Entrypoints

## Contract

- Sources: `crates/om9-app/src/three_dm_host.rs, native/native_exchange.cpp`
- Owner: `OM9-L2-03-15`
- Positive oracle: Import/Export 3DM through the shared host adapter
- Negative oracle: disabled/missing importer must not create an empty file or claim success
- Verification: existing 2026-10-19 host evidence; file_widgets 21 passed
- Manual M01-UI review pending.
