# Sphere FitPoints from source vertices — 2026-11-06

The `Sphere > FitPoints > Selection` path previously obtained B-rep vertex
positions from offsets in the display snapshot and used local coordinates for
selected points. It now obtains topological vertex coordinates from the OCCT
source and applies each entity placement to the selected data. Authored mesh
vertices remain mesh geometry; curve input remains a 16-sample approximation.
Repeated selection checks the cumulative point limit before modifying the
pending input. The native adapter checks shape validity and vertex count;
count equality does not prove persistent topology-ID provenance.

With OCCT 8.0.1, target `L:\om9-target` and `-j 2`, the focused app test
`solid_fit_source` passed 3 tests: an independent 4×5×6 box at two display
deflections, translated world-space fit and Undo, a placed authored point,
and point-limit refusal without commit. The source vertex test also works
with a point limit lower than the display mesh size. Four DupBorder tests
and 31 core curve/surface tests passed in the same batch before the final
point-limit test was added. The no-default-features check passed with warnings.

Evidence is fixture-specific, not a general accuracy or topology certificate.
Native OCCT allocation and latency, desktop gestures, NativeBrep, rotated or
nonuniform placements, very large selections, other Solid/Curve commands,
and global memory admission for these temporary point vectors remain open.
The behavior change for placed input corrects the world coordinates used by
the fit; it does not assert original Matrix9 behavior.


## NativeBrep mixed selection gap (2026-11-13)

The ignored regression sphere_fit_selection_includes_native_brep_vertices creates one box as a NativeBrep from an OM9-authored B-rep and four authored points, selects all five and requests FitPoints selection data. The box carries translation [10, -3, 7]. Expected are four points plus eight source vertices with that placement; the current function returns only the four points because its match ignores Geometry::NativeBrep. The focused ignored test failed as expected: 4 versus 12. This is a controller selection-data reproducer, not a saved/reopened document, a completed Sphere operation, independent third-party file or desktop test. If other supported selected objects provide enough points, the partial list can proceed into the existing fit flow; the resulting wrong sphere was not separately measured here. No product behavior changed. The proposed source-vertex support and refusal behavior await user decision.

A follow-up temporary controller diagnostic used four unit-sphere points and a NativeBrep box translated to start at x=10. Sphere, FitPoints, Selection and Enter committed a sphere whose maximum x bound was below 2, consistent with fitting only the four points; the selected box was omitted. This establishes that the incomplete list can reach a committed product result in this authored fixture. The diagnostic assertion was removed after execution because it described incorrect behavior; the ignored regression above remains. No saved/reopened, third-party file or desktop path was tested.

### Existing source-only building blocks

The retained NativeBrep payload can already be converted to OCCT source with typed admission by the path used by ExtractWireframe. The existing inspect_source returns face, edge and vertex counts without a display snapshot; the existing native vertex reader used by source_vertices takes source text and a bounded output buffer. The Rust source_vertices wrapper currently requires a complete Brep solely to validate its stored source/topology and determine the output count, so it cannot simply accept the NativeBrep payload or source string as-is. A possible implementation, if approved, must carry the converted source reservation while inspecting and extracting its bounded vertex count, include the already accumulated selection points in admission, apply placement after source extraction, and propagate errors before adding the batch to the pending fit. OCCT parse and validation allocations remain outside the managed cap. This is a code-path review, not an implemented adapter, runtime proof or claim of bounded native peak.

### Other selection types and snapshot timing

The current selection collector explicitly handles Point, Curve, Mesh and OCCT Brep; all other geometry variants, including Surface and NativeBrep, take the default branch and contribute no points. The reproduced incorrect committed sphere above concerns NativeBrep; Surface selection was not separately exercised and this source review does not establish its intended FitPoints semantics. Selection values are copied into the pending point list only after the collector returns successfully; this observation does not prove every later stale-input or memory-recovery case. A NativeBrep fix alone should not be described as support for all selected geometry.

### NativeBrep source vertices implemented (2026-11-13)

Sphere FitPoints now converts selected NativeBrep payloads to canonical source with admission, inspects source topology and extracts native vertices without creating a display mesh. The controller applies each entity placement and includes pending and accumulated point buffers in the temporary reservation. In an authored 3DM-derived box fixture, a translated NativeBrep and four points were saved and reopened; all eight source vertices and four points were selected, and the fitted sphere matched the independent box circumcenter and radius. Undo and Redo restored exact document snapshots. A separate malformed source after a valid source was rejected before adding partial fit points; document and existing Redo were retained. Simulated managed pressure also refused without mutation and retry succeeded; a separate injected vertex-buffer admission failure retained its typed budget error. These error fixtures were not reopened. Six source tests passed, plus 55 sphere-filtered library tests with one ignored; no-default check and existing debug build passed. Native parser allocations are outside the managed cap; actual OOM, desktop, arbitrary affine placements and third-party files were not tested. Other ignored selection geometry is unchanged.

### Scaled and reflected NativeBrep placement

A separate authored NativeBrep box was saved and reopened with nonuniform axis scaling, a reflection and translation. The selected source supplied all eight independently expected world-space corners. Sphere FitPoints produced the analytic center [14, -1.75, 4] and radius sqrt(26.5625); exact document snapshots passed Undo/Redo. One focused controller test passed. This does not establish shear, every affine transformation, third-party files, physical desktop or native peak memory.

### FitPoints command and menu dispatch

The same saved/reopened translated NativeBrep box and four points were then exercised in fresh controllers through both the Sphere/FitPoints command sequence and the SolidSphereFitSpheretoPoints menu dispatcher. Each produced the independently checked analytic center and radius and restored complete document snapshots through Undo/Redo. The focused test passed both paths. The menu was dispatched programmatically; no physical menu click or desktop evidence is claimed.


### Actual executable with reopened NativeBrep

A separate om9 executable process opened an authored saved document containing a translated NativeBrep box and four points, selected the inputs through its MCP command interface, ran Sphere FitPoints Selection, undid and redid the result, and saved it. The independently reopened output retained each source entity and had a sphere with the analytic center and radius; process state counts were 5, 6, 5 and 6 at the relevant stages. The focused test and the full MCP stdio group passed: 10 passed, one unrelated manual desktop inspection ignored. This exercises the actual debug executable and command interface, not a physical menu click, displayed viewport, real memory pressure or third-party 3DM.
