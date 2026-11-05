# BRep Object Info dimensions

Scope: `OM9-L3-03-14-005`, standalone app Object Info label and receipt for a single selected root `Geometry::Brep`. The app calls the existing OCCT `creation::measure` on the canonical BRep after applying its placement; the result is expressed in the document's current length unit. `AddOptimal` is used by the existing kernel measurement. The scene mesh and transformed local bounding-box corners are not used as the numeric oracle. A one-entry cache keys the result by document, revision, epoch, object identity/revision, placement, unit and limits. It is shared by label and receipt.

On a failed or unavailable measurement, dimensions and kernel validity are unavailable; portable source/display validation is not presented as OCCT validity. The existing Object Info format rounds each component to three decimals. Measurement and a nonidentity BRep transform are synchronous on a cache miss; complex shapes may pause the UI. NativeBrep and instances remain outside this change.

Windows; root workspace; `CARGO_TARGET_DIR=L:/om9-target`; `-j 2`; default `occt-brep` features. Base `89ef2676b597ebc5ab38fa5c6acf0ad47a2f261b`, with pre-existing unrelated worktree changes. SHA-256: `crates/om9-app/src/ui/object_info.rs` CB953ECD5CDD9140C87B9496B20BB9E02B02E0D21F5A8C772BAF1392FFF9ED73; `crates/om9-app/src/ui.rs` 2CA33CA7D3FD1DB03FC125F0DADAA9A6C6117DA0BBAD882E2AA2F789FE26BD26; `crates/om9-core/src/object_info.rs` 180C27C13B367CAABBECE3FD319EABD90CA208B6BD9C4EA5C54E3AB36A72471F.

- RED: `cargo test -p om9-app -j 2 --lib selected_brep_reports_world_bounds_in_document_units` failed with the old unavailable label for a translated, rotated box in centimetres.
- GREEN: `cargo test -p om9-app -j 2 --lib ui::object_info` — 4 passed: transformed box and units, cache invalidation, malformed kernel source, rotated sphere. For the 45° box, OCCT bounds exceeded the analytic extent by approximately 2e-7 mm; assertions allow 5e-7 mm for that fixture. These are fixture-specific observations, not general tolerances.
- `cargo test -p om9-core -j 2 --lib object_info` — 1 passed.
- `cargo check -p om9-app -j 2 --no-default-features --lib` — passed with 17 existing unrelated warnings; this developer fallback does not provide OCCT coverage.
- `python tools/public_source_audit.py` — 0 errors before evidence update.

No real-window visual acceptance, application build/restart, Rhino comparison, broad imported BRep corpus, nonuniform transform certification or complex-shape latency measurement was performed. The current OM9 process continues using its old executable. Next: verify selected solids in a freshly built desktop executable, exercise nonuniform/imported cases and measure selection latency before claiming broad precision or performance.
