# Native definition and member copy budget

Copied native definition metadata now shares the existing 512 MiB serialized-copy-data budget with copied geometry and attributes in each source namespace. Full openNURBS remains in progress; public primary integration and actual Rhino5 application acceptance remain pending.

The writer counts all requested source-definition copies and measures each original component once before allocating the first definition copy. It uses the existing Rust overflow-safe multiplication/addition guard. Geometry copy preflight starts with this definition total, rather than an independent quota. Later geometry refresh and final transformed/metadata checks start with the actual current serialized copied definitions, so current member lists and definition overlays count too. This limits serialized native copy data; it is not a measured process RSS guarantee or a global quota across multiple source namespaces.

## Verification

- RED: 300 copies of a valid definition with a 1 MiB wide-character description did not hit the expected early definition-data guard. The new preflight passes the same oracle and leaves an existing destination byte-for-byte unchanged.
- Combined fixture: 300 definitions with 64 KiB descriptions and copies of a valid 40,000-CV NURBS curve each fit the quota independently, but their sum exceeds it. Member-copy preflight rejects before member cloning and preserves the destination. Existing small copies still pass.
- Fresh native target build/run exit0 and full CTest 8/8 exit0, 27.40s, including both immutable user ring geometry fixtures.
- Isolated SDK module build exit0. Fresh FreeCAD 105/105 checks across five suites, all owned final process exit0; Python source/install bytes match and native binary stays unchanged during this regression.
- The preceding full GUI result of 386/386 across 25 suites belongs to the preceding binary. It is historical evidence, not a claim that all 386 were rerun for this allocation-only change. Rust budget implementation/tests are unchanged; prior Rust evidence remains applicable.

|Suite|Checks|Artifact|
|---|---|---|
|definition_copy|27/27|`build/three_dm_definition_copy_smoke-1/5cb47f19bd2948b0849a5e2ca3b5b142/results.json`|
|source_members|23/23|`build/three_dm_source_members_smoke-1/378b60e5335743869a104aae3b66a448/results.json`|
|independent_member|18/18|`build/three_dm_independent_member_smoke-1/96afbb0d970d40f7bfb0db3de448b0c8/results.json`|
|shared_proxy_geometry|8/8|`build/three_dm_shared_proxy_geometry_smoke-1/c9adcc39d7c346d6a491e7eb45ea45fb/results.json`|
|new_blocks|29/29|`build/three_dm_new_blocks_smoke-1/8139c19b6e07487eaf3741e34a676f3c/results.json`|

## Remaining scope

Source-member proxy creation/copy, legacy owner/source/proxy migration, linked/external resources, different-source document policy, shared menu/CMD routing, broader class/reference coverage and final package review remain pending. Large graph performance and aggregate quotas across source namespaces still need work.

## Fingerprints

- `Gui/ThreeDmMerge.cpp` SHA256 `f5d0962e6016776524c40f7c0dcbfefe35caafa3efa52d437f62d8fe212e33a5`
- `tests/native/three_dm_merge.cpp` SHA256 `9ce34ed07dc16cd62ba52cce32fd1f9dfb3d1f2327e7b99b66b25cff85da6535`
- Isolated `bin/OpenMatrix9Gui.pyd` SHA256 `a67dc12a30bb5b30df61c02d4bffebcacf9030530ac2a2c09c33c5fbd772c854`
