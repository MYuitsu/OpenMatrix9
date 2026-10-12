# Independent review: geometry contract and Rust interfaces

Date: 2026-10-10. Baseline: `fcdb0ac`. Reviewed Task 3's
`rust/src/geometry_contract.rs` and tests, plus
`geometry_contract_ffi.rs` and `nurbs_basis_ffi.rs` and their tests.
The Task 2 portable basis review remains separate. Native companion extraction,
native geometry operations, release/GUI execution and combined final acceptance
are outside this review. Refreshed against the final frozen portable sources,
including Rust-owned companion limits and explicit fact/domain scopes.

## Conclusion

No remaining blocking finding in the reviewed portable descriptors, operation
policy and Rust interfaces. An initial descriptor-summary data-loss concern was
reported and corrected during review: inspected curve/face form and domains now
populate typed facts and survive the returned summary together with their scopes.
The conclusion applies to the final hashes below. This conclusion accepts
no native lifetime, geometry-preservation, Rhino-parity or full P0 claim.

## Reviewed behavior

- Owned descriptors reuse the existing `Stamp`; they introduce no second identity
  registry or native handle. Unknown inspection is explicit and does not become
  false, zero topology or empty attributes. Schema validation checks compatibility
  only and is documented as separate from basis/native validity.
- Surface periodicity and closure never classify a solid. The FFI's solid fact
  comes from its supplied native representation tag; generic BRep and unknown
  tags leave solid status unknown. Native callers must inspect that tag correctly.
- Topology preserves ordered edge-use occurrences and distinct edge identities.
  Incomplete face/loop inspection yields unknown counts. Clone tests demonstrate
  independent owned topology and warning buffers, not native clone semantics.
- Affine parameter maps require finite coefficients/domains, increasing domains,
  a nonzero scale, nonempty external evidence identity and exact endpoint-image
  agreement. Queries apply the actual affine formula and reject outside-domain
  and nonfinite results. Reversed maps are supported. Nonempty `evidence_id` is
  provenance supplied by the caller, not independent verification of geometry.
- Raw parameter reuse requires an explicit preserved-parameterization fact.
  Preserved locus/topology and conversion permission cannot supply it. A known
  nonidentity or malformed map disables raw reuse; future schemas reject.
  An explicit preserved fact with unknown map remains a caller-supplied claim,
  not a map synthesized by this module.
- Operation policy permits read-only inspection and the existing v5 working
  exchange branches. Unknown IDs/options, strict preservation requests,
  tessellation/healing requests and additional solvers reject. The legacy coarse
  API remains unchanged; its Boolean bit does not enable a detailed solver gate.
- Descriptor FFI rejects inconsistent representation/legacy tags, unknown flag
  bits, values without known flags, invalid domain counts/ranges and zero stamp
  components. These are metadata checks: nonzero stamp fields do not prove the
  registry still validates that stamp. Native before/after identity verification
  remains required, as does unit/world provenance in the companion capture.
- After the correction, `RepresentationDescriptor::Curve` and
  `TrimmedFace(SurfaceFacts)` retain scalar observations. The summary exports
  validated form, known domain count, domain values and their explicit scopes. Face component facts
  retain the trimmed-face tag and do not promote its surface to a solid. No full
  basis arrays, native pointers or retained native handles are claimed here.
- `GeometryFactScope` separates supporting-geometry facts from facts observed on
  restricted geometry. `DomainScope` independently distinguishes the supporting
  domain, an edge restriction and a face UV restriction. Known form/boolean flags
  require a valid fact scope; known domains require a compatible domain scope.
  Unrecognized scopes and missing or incompatible scopes for known facts reject.
  Supporting closure/periodicity may
  accompany a restricted domain only with both scopes explicit; a supporting
  circle's closed flag does not then become a claim that its restricted edge is
  closed. Ownership is a stack-owned copy of scalar facts during the call, with
  no retained native handle or implied correspondence to a converted basis.
- Rust exposes the companion extraction budget: 8192 total poles and 16384 knots
  per axis. Curve dimensions, surface product and knot counts are checked before
  extraction. Over-budget status means basis uninspected; the native caller must
  honor that gate. It is not a conversion-validity failure or a fabricated empty basis. This bounded
  metadata budget remains distinct from the larger native conversion policy.
- Basis FFI performs checked degree/count/grid/payload preflight before borrowing,
  then rejects null nonempty buffers, misalignment, address wraparound and invalid
  periodic flags. Absent weights require the documented null/zero pair. The
  module documents initialized immutable live allocations, synchronous lifetime,
  concurrency, pointer nonretention and native exception obligations. Address
  checks cannot prove allocation accessibility, initialization or lifetime.
- Inspected the current C++ basis ABI record declarations and their static
  assertions only. Independent Rust checks agree with their sizes and relevant
  offsets on the current 64-bit target. This is not native compilation/linkage
  or runtime ABI evidence; other targets require their own review.

## Verification evidence

- Earlier checkpoint: independently ran Cargo tests before the descriptor correction:
  geometry contract **14 passed**, descriptor FFI **8 passed**, basis FFI
  **12 passed** (34 total, zero failures).
- Earlier checkpoint: after the correction, compiled the actual reviewed sources and tests in a
  temporary standalone Rust test harness. Existing tests: 14 geometry, 9
  descriptor FFI, 12 basis FFI. Three additional review probes checked exact
  curve/face domain/form retention, surface count/address rejection before
  pointer reads, and current Rust/C++ basis record sizes/offsets.
  Result: **38 passed, 0 failed**. The harness avoids unrelated concurrent
  native integration work. It predates the final scope fields and does not
  substitute for the final combined focused check below.
- Final frozen checkpoint: independently ran Cargo's six focused integration
  targets against the combined crate. **67 passed, 0 failed, exit 0**: 14 geometry
  contract, 13 descriptor FFI, 12 basis FFI, 20 basis, 3 generated spline basis and
  5 spline geometry. Current descriptor tests include retained form/domain,
  separate supporting/restriction scopes, unknown/incompatible-scope rejection,
  companion budget boundaries/product overflow, and descriptor ABI size/offsets.
  Only focused Rust tests were rerun; no full-suite, SDK/native, GUI or release
  execution was performed by this reviewer for the refresh.
- Read the retained Task 3 map RED log
  `H:/FreeCAD-src/build/p0-geometry-contract-red-map-20261010-001301-2501220.log`:
  11 passed, 3 failed, exit 101, including raw reuse and map scaffold failures.
  Read the retained final GREEN log
  `H:/FreeCAD-src/build/p0-geometry-contract-final-20261010-001434-5540319.log`:
  14 passed, 0 failed, exit 0. The descriptor FFI RED log records 3 passed and
  5 failed against its initial scaffold. Parent-reported fix RED/GREEN details
  beyond these retained files are not certified by this review.
- No native, release or GUI build, Git write or implementation-source edit was
  performed by this reviewer. Final native stamp/provenance capture, gates,
  runtime fixtures and unsupported acceptance clauses remain open.

## Reviewed content hashes (SHA-256)

| File | Hash |
|---|---|
| `rust/src/geometry_contract.rs` | `25d4ad12b4a17228aa492fe17e548c660334649f79df3e3f658880bbf473f59e` |
| `rust/tests/geometry_contract.rs` | `03d83d3e22ea02257364320ac3f42867687a376e47cf6ca162a60b9b1d7711b6` |
| `rust/src/geometry_contract_ffi.rs` | `3547370f686124e3a522a3630073328b6d4c94acf31d63662df82a3a2869770b` |
| `rust/tests/geometry_contract_ffi.rs` | `e3c5d223662a943a4a09ec0e582b7e44eedebbe3fd10c9bf8fe3518dbb190bf7` |
| `rust/src/nurbs_basis_ffi.rs` | `c2c6f1ea5afd97fc1ce41993e37bd4bd85a6a6f8c27d1753a13646966ba3be58` |
| `rust/tests/nurbs_basis_ffi.rs` | `d7f331f333401d076589874de781f0cec7f172d9f8c0ebc13819076feac375f0` |
| `rust/src/lib.rs` | `ef8648b92fc44a2d69a384f70a7bc68484979cdd993e632943337b50d746e270` |

Further changes require a matching review/checkpoint; these hashes do not cover
the parent thread's native companion implementation or final binary.
