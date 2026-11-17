# DupBorder: closed and periodic standalone surfaces

Previously DupBorder always emitted four parameter-edge isocurves for a standalone NURBS surface. An authored closed cylindrical surface therefore produced two circular boundaries plus two copies of the internal parameter seam. The user approved omitting the seam when closure can be verified.

For a complete clamped axis, the controller now verifies exactly matching first/last control rows and weights before treating that axis as closed. For a full-domain periodic axis, it uses the existing repeated-periodic-basis certification. A partial domain is not assumed closed merely because its underlying surface has matching rows. Uncertified full-domain periodic input is refused, and a surface closed on both axes is refused with no boundary rather than publishing an empty transaction. Other unsupported or unverified closure representations are not claimed as certified; a geometrically closed surface with a different representation can still retain redundant parameter edges.

The reopened, scale/shear cylindrical surface fixture initially failed with four outputs; after the change it produces two world-space circular boundary curves, with one Undo/Redo. A certified periodic surface fixture produces two closed boundary curves whose sampled points match its own source surface at both open ends after save/reopen; this comparison shares OM9 evaluation code and is not an independent mathematical proof of every point. A partial cylinder and an almost-closed cylinder retain four edges. A certified doubly periodic torus has no boundary and refuses without changing the document or Redo; an uncertified periodic surface also refuses without losing Redo. The source placement and B-rep DupBorder route remain intact.

`cargo test -p om9-app --test dup_border --test wireframe --test curve_menu --target-dir L:/om9-target -j 2 -- --test-threads=1`: DupBorder 17 passed; wireframe 44 passed with 3 ignored; curve menu 6 passed. A further focused periodic test confirmed the output curves classify as closed. `cargo check -p om9-app --no-default-features --target-dir L:/om9-target -j 2` passed with existing warnings. The user approved recording the tested rule in feature guidance.

The certified rational periodic fixture was extended through a second save/open: its lower DupBorder curve is extruded by 5 mm into one closed solid. The volume agrees within 1e-5 relative with an area estimated by sampling the original surface at 4,096 intervals; world bounds confirm its translated position and height, and extrusion Undo/Redo restores the respective documents. This is one fixture, and the sampled area is numerical rather than an independent exact proof. The same 17 DupBorder, 44 wireframe (3 ignored), and 6 curve menu tests passed after adding this downstream check.

The closure checks are deliberately conservative, not a universal geometric topology classifier. Arbitrary NURBS imports, other periodic bases, trimmed B-reps, all closed surfaces, desktop and native peak remain unverified. A stored surface with a partial periodic domain retains parameter boundaries; the single extrusion fixture does not certify other downstream commands.

## Collapsed conical boundary

On a reopened conical NURBS surface, DupBorder previously emitted the circular
base and a second curve that was entirely the apex point. It now reuses the
ExtractWireframe source check: a fully clamped boundary is omitted only when
all control points in that boundary row are exactly identical. For the tested
translated cone, CMD and menu return the base curve; sampled points match the
source surface and one Undo/Redo restores the result. The source evaluator is
shared with the output comparison. A second reopened cone with a small but
nonzero top retains both boundary curves and passes the same checks. An output
limit refusal for the latter keeps the document and existing Redo.

The Curve menu, DupBorder, Isocurve and Wireframe batches passed 6, 19, 20 and
70 tests; four Wireframe diagnostics were ignored. A separate no-OCCT controller
test for the pointed cone passed, as did the existing debug build and public
source audit. Arbitrary surfaces, output roundtrip for this fixture, desktop,
and physical memory pressure remain unverified. No new build directory was
created.
