# Length: selected B-rep edge after save and reopen

The Length controller formerly mapped a selected display edge UUID to its array position and passed that position as an OCCT edge index. A 2 × 3 × 4 mm Box fixture permutes two different-length display/topology edges, saves v4 and reopens: the selected 3 mm edge was reported as 4 mm. This red test used `Controller::measure_selected_length`.

The selected-edge path now validates the selection and scene, obtains a segment of the displayed edge, and resolves a unique source edge against that segment in the same OCCT shape/map used for `GCPnts_AbscissaPoint::Length`. The whole-object measurement path is unchanged. The check is numerical, depends on the declared display deflection and may refuse ambiguous or unverifiable data. The native operation remains synchronous; OCCT temporary allocations and latency are not measured or individually admitted by this change.

Validation on 2026-11-11 with `L:/om9-target`, `-j 2`:

- `cargo test -p om9-app --test measure_extended`: 9 passed, including the reopened reordered edge (3 mm, including `measure_selection` feedback), a translated cylinder circular edge against `4π` mm, and invalid display refusal with document and Redo retained. Whole-box, area, volume and other measurement tests also pass.
- `cargo test -p om9-app --test brep_picking_precision -- --test-threads=1`: 20 passed. `cargo check -p om9-app --no-default-features`, `cargo build -p om9-app --bin om9` and the public-source audit passed; build has existing warnings.
- `cargo test -p om9-app --test solid_difference_ui reopened_reordered_length_routes_through_measure_icon_and_cmd`: one egui test passed for both the actual Measure palette icon and typed CMD. Each run reopened the reordered Box, selected the 3 mm edge by synthetic Ctrl+Shift viewport click, and asserted the displayed `Length = 3 mm` with unchanged document. This is headless egui, not an operating-system desktop interaction or GPU rendering check.
- Follow-up `cargo test -p om9-app --test measure_extended`: 10 passed. A cylinder circular edge at display deflections 0.1 and 0.01 mm, scaled 2× in X and 3× in Y and translated, measured the resulting ellipse within 1e-6 mm of independent composite Simpson integration. These are two separately generated analytic fixtures; this does not prove a global error bound or all affine transforms.

This does not prove all trimmed/imported curves, display profiles, placements, duplicate overlapping edges, pressure conditions, desktop interaction, or native memory/latency. Existing direct index-based kernel APIs are not certified by this fix.
