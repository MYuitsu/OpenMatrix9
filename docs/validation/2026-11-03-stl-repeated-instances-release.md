# Repeated B-rep STL export: release baseline

Windows x64, OCCT 8.0.1, optimized release build with `CARGO_TARGET_DIR=E:/om9-surface-snap-validation` and `-j 2`. The same test executable was run once per fixture in fresh processes. Each exports a radius 5 mm sphere at 0.005 mm linear tolerance and 0.3 rad angular criterion; selected copies are separated by 15 mm. All STL destinations were temporary, triangle/file lengths were checked, then files were removed. The user's OM9 window was not touched.

| Fixture | Triangles | Export elapsed | Process peak commit (MiB) | Peak working set (MiB) |
|---|---:|---:|---:|
| One sphere | 19,950 | 68.296 ms | 69.86 | 64.05 |
| Eight independent B-rep copies | 159,600 | 513.778 ms | 95.42 | 95.45 |
| Eight instances of one shared B-rep definition | 159,600 | 498.717 ms | 95.19 | 95.27 |

Peak commit corresponds to the Windows process-lifetime `PeakPagefileUsage` counter, not bytes physically paged out; it includes initialization and all Rust/native allocations. Baseline process peaks differed: 65,142,784, 57,167,872 and 57,069,568 bytes respectively. There was one run per fixture, uncontrolled machine load and no measurement of individual OCCT calls or desktop responsiveness. These numbers are not a performance distribution, an OCCT-only attribution, a process memory limit or an accuracy certificate.

Current source traversal calls canonical B-rep tessellation independently per occurrence; these numbers are consistent with repeated work but do not isolate it from welding, topology checking, file writing or other costs. A future shared export snapshot must key the actual source and full meshing profile including local deflection, retain independent occurrence placement and topology/weld checks, and account its lifetime and native/scratch coexistence against the shared budget. The current single-slot worker and aggregate admission do not yet establish a safe cache envelope, so this diagnostic does not add an unbudgeted cache.

Executable SHA-256 `e830641496bdf272e69023b60a9d180379d1cb95e23bfc982cbbd1e1ff2ae710`; fixture source SHA-256 `e246fbe5f10cc6032492b9b0504ff4409bfe39cf9c61bd257726d131e9798484`. The fixture extends the earlier independent-copy probe with actual shared definition instances; it does not compare against a prior executable or prove an improvement.
