# Selection of larger valid trimmed surfaces

**Later update:** the separate 500-boundary ExtractWireframe/DupBorder limit
mentioned below was subsequently aligned with the 707-edge topology limit for
the tested planar fixtures. [Evidence](2026-11-13-large-trim-boundary-commands.md).

An authored planar trim with 501 boundary edges was valid but exceeded the
old 500-edge screen, so the scene displayed the untrimmed support surface and
normal point selection refused it. The topology verifier now counts unordered
edge pairs (`n * (n - 1) / 2`) against the existing 250,000-pair bound. The
largest permitted count is consequently 707; a 708-edge source still exceeds
the bound. This bound limits that pair comparison stage, not the total CPU
time or the work of every other operation.

The display builder follows the same 707-edge maximum. Incremental scene
admission now scans recognized trim-loop JSON into fixed stack buffers and
counts actual scanline crossings before reserving. It uses the same output
headroom as the decoded planner and falls back to the previous conservative
estimate if the JSON structure is uncertain. It does not deserialize or
allocate a polygon before admission.

The reproduced 501-edge fixture now displays its trim and selects inside but
not outside, with source metadata unchanged. Full and incremental scenes match
and the estimated peak covers the retained cache. A 505-edge fixture with a
hole selects material but not the hole and also matches the incremental
scene. A 707-edge fixture passes full and incremental construction; topology
tests reject an intersecting polygon and the 708-edge boundary. A simulated
egui click after `.om9` save/reopen selects the 501-edge surface and keeps the
document and a previous Redo entry. Raw preflight crossing counts matched the
actual builder for the 505-edge fixture; escaped, repeated and malformed loop
field fixtures do not use the optimized scan.

The full `om9-scene` suite, the `trim_uv` core test, and the affected
`wireframe`, `isocurve` and `rubber_band` app groups passed. The initial
attempt to use the old worst-case incremental estimate failed with
`Budget("points")`; the current scanner fixes that tested fixture. These are
authored planar fixtures and do not prove all 501–707-edge topologies, curved
surfaces, physical desktop, allocator peak or responsiveness on large
drawings. `ExtractWireframe` retains its separate 500-boundary output limit.
The running desktop binary was not replaced.
