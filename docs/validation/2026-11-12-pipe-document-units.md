# Pipe radius follows document units — 2026-11-12

For a 100 mm rail in a saved/reopened centimetre document, `Pipe 0.2` previously made a 0.2 mm radius pipe with volume approximately 12.56637 mm³ instead of a 2 mm radius pipe with volume approximately 1256.63706 mm³. The product-controller regression failed before the fix. Numeric staged radii were also stored in raw document units while picked points yielded millimetres, so mixing them could give inconsistent radii.

The direct numeric dispatcher now converts radii from document units. Staged numeric radius and diameter input is converted when accepted, so stored radii and picked-point radii share millimetres for previews, stations, chain and multiple-rail calculations. The simple staged commit calls the same internal millimetre-based helper as the direct dispatcher, without converting its already converted values a second time. Geometry/selection and Undo contracts remain unchanged.

The reopened-document test checks 12 centimetre routes on a straight rail: CMD, menu, mixed numeric/point inputs in both orders, varying radius, diameter, an intermediate numeric station, thick pipe and round caps. It checks closed solids against analytic volumes, full Undo/Redo and saved/reopened output. Two further centimetre cases check Multiple and PipeChain on two straight rails, result counts, analytic volumes and a single Undo/Redo transaction. Direct and staged unit-conversion overflow preserve document and Redo; staged entry retains the prompt.

A separate centimetre preview fixture verifies the interpolated world-space radius at a quarter of a straight rail after a numeric first radius and mouse-derived second radius. The preview leaves the document and Undo unchanged; this is a controller preview, not a physical viewport observation.

Validation: Solid workflow suite passed 116 tests with one ignored; Pipe-filtered egui interaction tests passed 29; Solid input regressions passed 99. The no-default-features fallback test passed one case. Those egui and fallback cases are not necessarily centimetre fixtures. Physical desktop clicks, all curves, all station profiles, explicit unit suffixes and native peak memory remain unverified.

The debug executable built in the existing `L:/om9-target` directory with `-j 2`; the public source audit reported no errors. The dedicated centimetre preview test passed separately after the workflow batch.
