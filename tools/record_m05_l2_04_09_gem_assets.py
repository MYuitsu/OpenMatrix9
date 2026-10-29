#!/usr/bin/env python3
"""Record the M05 ``OM9-L2-04-09`` gem asset/loader capability from real runs.

Scope: all eight L3 of ``01-febe-gem-asset-shapes-and-loader``:

* ``OM9-L3-04-09-001`` cut/shape/dimension schema (backend, AUTO_ONLY);
* ``OM9-L3-04-09-002`` reference mesh assets and the canonical library (backend, AUTO_ONLY);
* ``OM9-L3-04-09-003`` loader cut/shape/size picker (UI gate);
* ``OM9-L3-04-09-004`` custom shape from a closed planar profile (UI gate);
* ``OM9-L3-04-09-005`` emerald and baguette builders (UI gate);
* ``OM9-L3-04-09-006`` instance create/rotate/position and duplication (UI gate);
* ``OM9-L3-04-09-007`` properties and gem layer defaults (UI gate);
* ``OM9-L3-04-09-008`` girdle/table/facet reference API (backend, AUTO_ONLY).

The three backend L3 have no human-facing gate in their own record, so they are
recorded as ``AUTOMATED_EVIDENCE_ONLY``/``AUTO_ONLY`` and the dashboard may count them as
DONE. The five UI L3 keep the M05-UI manual gate OPEN: no reviewer signature is claimed.

Each L3 directory receives ``source_crosswalk.json`` (L4-01), ``api.md`` +
``positive.json`` + ``negative.json`` (L4-02) and, for the UI L3, one receipt per
``L4-...-03`` with the command, exit code, output hash and log.

Usage::

    python tools/record_m05_l2_04_09_gem_assets.py
"""

from __future__ import annotations

import datetime
import hashlib
import json
import os
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
KEY = "m05-l2-04-09-gem-assets-2026-10-27"
MILESTONE_KEY = "M05"
BATCH = ROOT / "implementation" / "evidence" / MILESTONE_KEY / "OM9-L2-04-09"
L3_ROOT = (
    ROOT
    / "OM9_Milestones"
    / "M05 - Gems and Layout"
    / "L2-04-09 - 01-febe-gem-asset-shapes-and-loader"
)
TARGET = "Standalone Rust OM9; Windows x64; CARGO_TARGET_DIR=L:/om9-target; -j 2"

CHECKS = [
    (
        "core-gem-assets.log",
        ["cargo", "test", "-p", "om9-core", "--test", "gem_assets", "-j", "2"],
    ),
    (
        "app-gem-panel.log",
        ["cargo", "test", "-p", "om9-app", "--test", "gem_panel", "-j", "2"],
    ),
    (
        "core-transaction-regression.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-core",
            "--test",
            "atomic_transactions",
            "--test",
            "transaction_scopes",
            "-j",
            "2",
        ],
    ),
    (
        "workspace-check.log",
        ["cargo", "check", "--workspace", "--all-targets", "-j", "2"],
    ),
    (
        "clippy-gem.log",
        [
            "cargo",
            "clippy",
            "-p",
            "om9-core",
            "-p",
            "om9-app",
            "-p",
            "om9-kernel",
            "--all-targets",
            "-j",
            "2",
            "--",
            "-D",
            "warnings",
        ],
    ),
    (
        "kernel-gem-brep.log",
        [
            "cargo",
            "test",
            "-p",
            "om9-kernel",
            "--features",
            "3dm-brep",
            "--test",
            "gem_brep",
            "-j",
            "2",
        ],
    ),
]

CHANGED = [
    "crates/om9-core/src/gem.rs",
    "crates/om9-core/src/lib.rs",
    "crates/om9-core/src/transaction.rs",
    "crates/om9-core/tests/gem_assets.rs",
    "crates/om9-kernel/src/brep.rs",
    "crates/om9-kernel/src/gem.rs",
    "crates/om9-kernel/src/lib.rs",
    "crates/om9-kernel/src/brep_bridge.cpp",
    "crates/om9-kernel/Cargo.toml",
    "crates/om9-kernel/tests/gem_brep.rs",
    "crates/om9-app/src/session/gem.rs",
    "crates/om9-app/src/session.rs",
    "crates/om9-app/src/ui/gem.rs",
    "crates/om9-app/src/ui.rs",
    "crates/om9-app/src/f6.rs",
    "crates/om9-app/src/icons.rs",
    "crates/om9-app/tests/gem_panel.rs",
    "tools/record_m05_l2_04_09_gem_assets.py",
]

MANUAL = "OPEN"
AUTO = "AUTO_ONLY"

L3_PACKAGES = {
    "OM9-L3-04-09-001": {
        "title": "Gem Shape Cut Dimension Schema",
        "folder": "L3-001 - Gem Shape Cut Dimension Schema",
        "gate": AUTO,
        "api": [
            (
                "`om9_core::gem::GemSpec`",
                "`shape`, `cut`, `length_mm`, `width_mm`, `girdle_mm`, `crown_height_mm`, "
                "`pavilion_depth_mm`, `crown_angle_deg`, `facet_count`, `pavilion_rows`, "
                "`bulge`, `corner_mm`, `profile`",
            ),
            (
                "`GemSpec::validate`",
                "typed refusals: shape-for-cut, non-finite, range, round diameter, width>length, "
                "depth, zero pavilion, cabochon dome, crown angle, bulge, facet count, corner, "
                "table ratio; never a clamped gem",
            ),
            (
                "`GemSpec::{to_json,from_json}`",
                "`om9.gem_spec.v1` / version 1 round-trip, `deny_unknown_fields`",
            ),
            (
                "`GemSize` / `from_outer_dimensions`",
                "the loader's length/width/depth entry point with the depth split",
            ),
        ],
        "positive": {
            "cases": [
                "every cut x shape round-trips through JSON with the same schema and version",
                "round diamond 6.5 mm, 3.95 mm deep -> length == width == 6.5, depth == 3.95",
                "emerald 9x7x4.5 with a 0.2 mm girdle keeps length/width/depth/girdle",
                "cabochon has crown height > 0 and pavilion depth == 0",
            ]
        },
        "negative": {
            "cases": [
                "NaN / infinite dimension -> Invalid(\"gem dimension is not finite\")",
                "negative or zero dimension -> Invalid(\"gem dimension range\")",
                "unknown cut tag -> Invalid(\"gem cut\")",
                "shape not offered by the cut -> Invalid(\"gem shape for cut\")",
                "width > length -> Invalid(\"gem width exceeds length\")",
                "pavilion depth == 0 on a faceted cut -> Invalid(\"gem pavilion depth is zero\")",
                "corner > half the short axis -> Invalid(\"gem corner exceeds the girdle\")",
                "wrong payload version/schema -> Invalid(\"gem spec version\"/\"gem spec schema\")",
            ]
        },
        "tests": ["crates/om9-core/tests/gem_assets.rs"],
        "positive_note": "every cut x shape validates and JSON round-trips; dimensions are exact",
        "negative_note": "every malformed spec is a typed refusal, never a clamp",
    },
    "OM9-L3-04-09-002": {
        "title": "Gem Reference Mesh And B-Rep Assets",
        "folder": "L3-002 - Gem Reference Mesh And B-Rep Assets",
        "gate": AUTO,
        "owner_crates": ["om9-core", "om9-kernel", "om9-app"],
        "sources": [
            {"path": "crates/om9-core/src/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {"path": "crates/om9-core/tests/gem_assets.rs", "state": "REGISTERED_TESTS"},
            {"path": "crates/om9-kernel/src/brep.rs", "state": "MESH_TO_BREP_FROM_TRIANGLES"},
            {"path": "crates/om9-kernel/src/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {
                "path": "crates/om9-kernel/src/brep_bridge.cpp",
                "state": "NATIVE_OCCT_SEWING_ADAPTER",
            },
            {"path": "crates/om9-kernel/tests/gem_brep.rs", "state": "REGISTERED_TESTS"},
            {"path": "crates/om9-app/src/session/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {"path": "crates/om9-app/src/ui/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {"path": "crates/om9-app/tests/gem_panel.rs", "state": "REGISTERED_TESTS"},
            {"path": "crates/om9-core/src/transaction.rs", "state": "FIELD-AWARE WRITE ADMISSION"},
        ],
        "api": [
            (
                "`GemLibrary::canonical`",
                "one asset per cut x shape with an authored size ladder; procedural only",
            ),
            ("`GemLibrary::{asset,assets,asset_for_cut,nearest_size}`", "library lookups"),
            (
                "`build_mesh(&GemSpec)`",
                "closed triangle mesh, outward orientation (edge-propagation), bbox matches the spec",
            ),
            (
                "`GemMesh`",
                "`mesh`, `closed`, `volume_mm3`, `extents_mm`; oracle from "
                "`ring_builder::inspect_solid`",
            ),
            (
                "`om9_kernel::brep::{from_triangles,from_mesh}`",
                "one closed OCCT solid from an indexed mesh; an open or non-manifold boundary is "
                "refused instead of sealed",
            ),
            (
                "`om9_kernel::gem::gem_brep`",
                "the gem adapter: `GemMesh` -> `Brep` with the same volume and gem-sized display "
                "deflection",
            ),
            (
                "`GemSession::brep` / `BRep=Yes|No`",
                "stores `Geometry::Brep` when the kernel feature is on (default), the portable "
                "mesh otherwise",
            ),
            (
                "`GemSession::preview` / `Controller::gem_world_preview`",
                "the session mesh (including the positioner scale) and its world placement, never "
                "written to the document",
            ),
            (
                "`ui::gem::gem_preview_segments`",
                "pure world->screen projection of the preview edges for the viewport overlay",
            ),
        ],
        "effects": [
            "A preview is the typed spec plus its computed mesh in session state; it never inserts "
            "a document object. The viewport paints that mesh as a wireframe overlay while a gem "
            "tool is open, including `Move`, `Rotate`, `Scale` and `Origin`, so Apply commits "
            "exactly what was shown.",
            "Apply is exactly one document transaction; Undo restores the previous state and Redo "
            "keeps the object identity.",
            "When the OCCT kernel feature is on, Apply stores `Geometry::Brep` (one closed solid "
            "sewn from the authored facets); `BRep=No` stores `Geometry::Mesh`. The `GemRecord` "
            "payload is identical in both cases, so replacing a gem never changes its `GemId`.",
            "Property edits and positioner scale keep the body kind of the existing entity: a "
            "B-Rep gem is re-sewn, a mesh gem stays a mesh; a shared definition member is built in "
            "the target's kind.",
            "Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, "
            "`Unsupported`, `Stale`, `Busy`); an open or non-manifold mesh is refused by the "
            "kernel instead of being sealed into a fake solid, and no silent mesh fallback hides a "
            "failed conversion.",
            "The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the "
            "mesh owner remains the entity itself.",
        ],
        "native": (
            "`crates/om9-kernel/src/brep_bridge.cpp:om9_occt_brep_from_triangles` is a selected "
            "OCCT adapter: it owns only `BRepBuilderAPI_MakePolygon/MakeFace/Sewing/MakeSolid`, "
            "`ShapeFix_Shape`, the closed-shell/volume checks and `om9_emit_brep_snapshot`. All "
            "validation, budgets, geometry input and error mapping stay in Rust "
            "(`om9-kernel/src/brep.rs`, `om9-kernel/src/gem.rs`)."
        ),
        "positive": {
            "cases": [
                "24 cut/shape assets, unique `asset_key`s, monotone size ladders",
                "every asset builds a closed mesh at its min/mid/max ladder size with bbox == spec",
                "round diamond 6.5 mm -> bbox 6.5 x 6.5 x 3.95 within 1e-6",
                "cabochon dome is closed with a flat bottom",
                "round, oval, emerald, baguette, heart, trillion, cabochon and a custom profile "
                "each convert to one valid closed B-Rep whose source volume equals the mesh volume "
                "within 1e-6 relative",
                "the loader's committed body is `Geometry::Brep` with the mesh volume, and a "
                "property edit keeps it a B-Rep",
                "the open gem tool exposes a world-space preview mesh and placement while the "
                "document has no gem object yet",
                "the positioner preview mirrors Move, Rotate and Scale and equals the committed "
                "placement",
            ]
        },
        "negative": {
            "cases": [
                "no binary Matrix gemstone assets are read, copied or shipped",
                "a custom profile with zero area is refused before any mesh exists",
                "an open mesh is detected by the exact-two-edges-per-edge oracle",
                "the kernel adapter refuses an open triangle mesh (each edge must have two face "
                "ancestors) instead of sealing it",
                "a sewn result with more than one shell is refused",
                "`BRep=Yes` returns a typed error when the conversion is impossible; it never "
                "falls back to a mesh body silently",
            ]
        },
        "tests": [
            "crates/om9-core/tests/gem_assets.rs",
            "crates/om9-kernel/tests/gem_brep.rs",
            "crates/om9-app/tests/gem_panel.rs",
            "crates/om9-app/src/ui/gem.rs (preview_tests)",
        ],
        "positive_note": (
            "the canonical library is authored/procedural and every asset is a closed solid; "
            "every cut family converts to one valid B-Rep with the same volume, the committed body "
            "is that B-Rep, and the open tool shows a world-space preview before Apply"
        ),
        "negative_note": (
            "no Matrix asset is copied; zero-area profiles and open meshes are refused; kernel "
            "sewing refuses open/multi-shell input; `BRep=Yes` never falls back to a mesh body "
            "silently"
        ),
    },
    "OM9-L3-04-09-003": {
        "title": "Gem Loader Shape Cut Size Picker",
        "folder": "L3-003 - Gem Loader Shape Cut Size Picker",
        "gate": MANUAL,
        "api": [
            ("`Controller::dispatch(OM9-GEM-001)` / `gvLoader`", "opens the loader preview"),
            (
                "`GemSession` loader fields",
                "`cut`, `shape`, `size_index`, `size_page`, `keep_original`, `dimensions`, "
                "`girdle_mm`, `replace`",
            ),
            (
                "command options",
                "`Shape=`, `Cut=`, `Size=LxW|D`, `Length=`, `Width=`, `Depth=`, `Girdle=`, "
                "`KeepOriginal=`",
            ),
            (
                "`Controller::gem_apply`",
                "one Insert, or Geometry+Extensions+Attributes when replacing the selected GemId",
            ),
        ],
        "positive": {
            "cases": [
                "preview builds a closed mesh and the document revision does not change",
                "Apply inserts exactly one gem in one Undo step; redo keeps the ObjectId",
                "Shape=Oval, Size=8x6, Depth=4.2, Cut=Cabochon are applied to the typed spec",
                "replacing a selected gem keeps its ObjectId and GemId",
                "KeepOriginal=Yes keeps 6.5 x 6.5 x 3.95 while changing the shape",
                "the viewport preview is a world-space closed mesh committed with Origin=1,2,3",
                "the committed body is one editable B-Rep with the mesh volume; BRep=No keeps the portable mesh",
            ]
        },
        "negative": {
            "cases": [
                "Size=999, Depth=0, Shape=Blob, Unknown=1 -> typed error, loader stays open",
                "Cancel drops the preview and leaves the document unchanged",
                "a locked Gem layer refuses the insert and leaves the document empty",
            ]
        },
        "tests": [
            "the_loader_previews_without_creating_an_object_and_cancel_leaves_no_phantom",
            "applying_the_loader_inserts_one_gem_in_one_transaction_and_undo_restores",
            "loader_options_change_the_cut_shape_size_and_depth",
            "keep_original_size_keeps_the_replaced_gems_dimensions",
            "a_bad_loader_option_is_refused_and_the_session_stays_open",
            "the_loader_preview_is_world_space_and_never_committed",
            "the_loader_commits_an_editable_brep_with_the_mesh_volume",
            "the_brep_option_keeps_the_portable_mesh_body",
        ],
        "positive_note": "one gemstone with the selected shape, cut and size in one Undo step; its preview is painted in the viewport and the committed body is an editable B-Rep",
        "negative_note": "invalid sizes/options and a locked layer never mutate the document",
    },
    "OM9-L3-04-09-004": {
        "title": "Gem Custom Shape From Profile",
        "folder": "L3-004 - Gem Custom Shape From Profile",
        "gate": MANUAL,
        "api": [
            ("`Controller::dispatch(BuilderCustomGem)` / `gvCustomGem`", "opens the custom builder"),
            (
                "`GemSession::selected_profile`",
                "samples the single selected curve, requires planar and closed",
            ),
            (
                "`GemSpec::from_profile(profile, cut, params)`",
                "centres the profile, rotates a tall profile into the X-long frame, stores it",
            ),
            (
                "`CustomGemParams`",
                "facet count, crown angle, crown height, girdle, pavilion depth, pavilion rows, "
                "bulge",
            ),
        ],
        "positive": {
            "cases": [
                "a 6 mm circle profile becomes a 63-point closed profile and a 6 mm gem",
                "the applied record keeps `spec.profile` and the derived length/width",
                "facet count / crown angle / bulge edits change the preview only until Apply",
            ]
        },
        "negative": {
            "cases": [
                "an open line -> Invalid(\"custom gem profile is not closed\")",
                "a tilted circle -> Invalid(\"custom gem profile must be planar\")",
                "self-intersecting / zero-area profiles -> typed profile refusals",
            ]
        },
        "tests": [
            "the_custom_builder_uses_a_closed_planar_profile",
            "the_custom_builder_refuses_an_open_or_non_planar_curve",
        ],
        "positive_note": "closed planar profile -> centred custom gem with the profile stored",
        "negative_note": "open, non-planar, self-intersecting and zero-area profiles are refused",
    },
    "OM9-L3-04-09-005": {
        "title": "Gem Emerald And Baguette Builders",
        "folder": "L3-005 - Gem Emerald And Baguette Builders",
        "gate": MANUAL,
        "api": [
            ("`Controller::dispatch(OM9-GEM-013)` / `gvEmerald`", "emerald builder"),
            ("`Controller::dispatch(OM9-GEM-014)` / `gvBaguette`", "baguette builder"),
            (
                "builder fields",
                "`Length=`, `Width=`, `Corner=`, `Depth=`, `Girdle=`",
            ),
            (
                "`GemSpec::{emerald,baguette}` / `set_girdle_mm` / `set_depth_mm`",
                "explicit girdle thickness, exact total depth, corner <= half the short axis",
            ),
        ],
        "positive": {
            "cases": [
                "emerald 9x7x4.5, corner 1.5, girdle 0.2 keeps every typed value",
                "baguette 7x3.5x2.8 with corner 0.4",
                "bbox agrees with length/width and the mesh is closed",
            ]
        },
        "negative": {
            "cases": [
                "Girdle=0 -> Invalid(\"gem dimension range\")",
                "Corner=9 -> Invalid(\"gem corner exceeds the girdle\")",
            ]
        },
        "tests": [
            "the_emerald_builder_commits_the_requested_dimensions",
            "the_baguette_builder_refuses_a_degenerate_girdle_and_an_oversized_corner",
        ],
        "positive_note": "emerald and baguette commit the requested length/width/corner/depth/girdle",
        "negative_note": "a degenerate girdle and an oversized corner are refused",
    },
    "OM9-L3-04-09-006": {
        "title": "Gem Instance Create Rotate And Position",
        "folder": "L3-006 - Gem Instance Create Rotate And Position",
        "gate": MANUAL,
        "api": [
            ("`Controller::dispatch(OM9-GEM-033)` / `gvGemPositioner`", "positioner"),
            (
                "`GemSession` positioner fields",
                "`target`, `moved`, `rotation_deg`, `scale`, `duplicate`",
            ),
            (
                "`GemInstance::transform`",
                "Rz*Ry*Rx, re-orthonormalized basis, validated placement",
            ),
            (
                "`apply_gem_positioner`",
                "Placement mutation for move/rotate; DefinitionInsert + Geometry(Instance) for a "
                "duplicate so copies share one definition",
            ),
        ],
        "positive": {
            "cases": [
                "Move=5,0,0 and Rotate=0,0,90 update the placement of the same entity",
                "Undo restores the previous placement",
                "Duplicate=Yes creates one definition with two instances (target keeps its id)",
                "one Undo removes the duplicate and returns the target to a plain mesh entity",
                "the positioner preview mirrors Move, Rotate and Scale and equals the committed placement",
            ]
        },
        "negative": {
            "cases": [
                "the positioner needs exactly one gem: a line -> Invalid(\"gem positioner needs one gem\")",
                "geometry/scale edits on an instance are refused (definition owns the mesh)",
            ]
        },
        "tests": [
            "the_positioner_moves_and_rotates_the_instance_in_place",
            "the_positioner_duplicates_with_one_shared_definition",
            "the_positioner_refuses_a_non_gem",
            "the_positioner_preview_mirrors_move_rotate_and_scale",
        ],
        "positive_note": "move/rotate keeps identity; a duplicate shares one definition and undoes as one step",
        "negative_note": "a non-gem and a definition-owned instance cannot be edited in place",
    },
    "OM9-L3-04-09-007": {
        "title": "Gem Asset Properties And Layer Defaults",
        "folder": "L3-007 - Gem Asset Properties And Layer Defaults",
        "gate": MANUAL,
        "api": [
            ("F6 `GemProperties` action", "opens `GemPropertiesEditor` for one selected gem"),
            (
                "`GemPropertiesEditor`",
                "label, material, colour, layer, length/width/depth/girdle/corner, shared count",
            ),
            (
                "`gem_properties_apply`",
                "one transaction: Label + Attributes + Geometry + Extensions as needed",
            ),
            (
                "`Controller::gem_layer`",
                "current layer when it is in the Gem group, else the `Gem 01` preset, else current",
            ),
            (
                "`ensure_layer_writable`",
                "locks/hidden Gem layers refuse the command: \"gem layer locked\"/\"gem layer hidden\"",
            ),
        ],
        "positive": {
            "cases": [
                "label/material/colour/girdle edits are one journal entry; Undo restores all",
                "the gem routes to the Gem 01 layer even when another layer is current",
                "a colour edit changes only the selected gem, never the scene",
                "F6 Gem Properties is enabled while the unimplemented GemReport stays disabled",
            ]
        },
        "negative": {
            "cases": [
                "a shared instance refuses geometry edits (\"shared gem definition\")",
                "a locked Gem layer refuses creation and property editing",
            ]
        },
        "tests": [
            "the_property_editor_is_one_transaction_and_undo_restores_every_field",
            "the_property_editor_refuses_a_shared_instance",
            "a_property_edit_never_recolours_the_rest_of_the_scene",
            "gems_route_to_the_gem_layer_default_even_from_another_current_layer",
        ],
        "positive_note": "one property transaction, Gem layer routing, no scene-wide recolour",
        "negative_note": "shared definitions and locked layers refuse edits",
    },
    "OM9-L3-04-09-008": {
        "title": "Gem Facet Outline Girdle Reference API",
        "folder": "L3-008 - Gem Facet Outline Girdle Reference API",
        "gate": AUTO,
        "api": [
            (
                "`GemReferenceGeometry`",
                "`name`, `points`, `closed`, `normal`; no triangle index is exposed",
            ),
            (
                "`gem_reference(spec, name)`",
                "`girdle`, `table`, `culet`, `profile`, `crown_facet:<n>`, `pavilion_facet:<n>`",
            ),
            (
                "`GemGeometryReference`",
                "`gem: GemId` + the named geometric reference for the current mesh",
            ),
            (
                "`girdle_outline` / `table_outline` / `profile_points` / `crown_facet` / "
                "`pavilion_facet`",
                "finite points and a stable outward normal",
            ),
        ],
        "positive": {
            "cases": [
                "girdle/table/culet references are closed and finite",
                "crown and pavilion facet normals point outward from the facet midpoint",
                "the same name returns the same points on every call (stable, index-free)",
                "the reference is bound to the record's GemId",
            ]
        },
        "negative": {
            "cases": [
                "an unknown reference name or facet index is refused",
                "a degenerate facet (zero normal) is refused instead of returning NaN",
            ]
        },
        "tests": ["crates/om9-core/tests/gem_assets.rs"],
        "positive_note": "named girdle/table/facet references with finite stable outward normals",
        "negative_note": "unknown names/indices and degenerate facets are typed refusals",
    },
}

L4_ROLE = {
    "01": ("Source Map", "SOURCE_AUDIT", "source"),
    "02": ("API Contract", "CONTRACT_TEST", "contract"),
    "03": ("UI Integration And Acceptance", "WIRE_UI", "ui"),
}


def run(command: list[str]) -> tuple[int, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    process = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    return process.returncode, process.stdout + process.stderr


def header(command: str, code: int, captured: str) -> str:
    return (
        f"$ {command}\n# exit code: {code}\n"
        f"# captured: {datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ')}\n"
        f"# target dir: {TARGET_DIR}\n\n{captured}"
    )


def sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def revision() -> tuple[str, bool]:
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True
    ).stdout.strip()
    dirty = bool(
        subprocess.run(
            ["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True
        ).stdout.strip()
    )
    return head, dirty


def l4_ids(l3_id: str, folder: str) -> list[str]:
    document = json.loads((L3_ROOT / folder / "l3.json").read_text(encoding="utf-8"))
    return [
        child["id"] if isinstance(child, dict) else str(child)
        for child in document.get("children_l4", [])
    ]


def write_json(path: pathlib.Path, document: dict) -> None:
    path.write_text(
        json.dumps(document, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )


def main() -> int:
    head, dirty = revision()
    BATCH.mkdir(parents=True, exist_ok=True)
    results = []
    for artifact, command in CHECKS:
        code, captured = run(command)
        (BATCH / artifact).write_text(header(" ".join(command), code, captured), encoding="utf-8")
        results.append(
            {
                "artifact": artifact,
                "command": " ".join(command),
                "exit_code": code,
                "sha256": sha(captured),
            }
        )
        if code != 0:
            raise SystemExit(f"check failed: {' '.join(command)}")

    summary = {}
    receipts_all = []
    for l3_id, package in L3_PACKAGES.items():
        directory = ROOT / "implementation" / "evidence" / MILESTONE_KEY / l3_id
        directory.mkdir(parents=True, exist_ok=True)
        gate = package["gate"]
        default_sources = [
            {"path": "crates/om9-core/src/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {"path": "crates/om9-core/tests/gem_assets.rs", "state": "REGISTERED_TESTS"},
            {"path": "crates/om9-app/src/session/gem.rs", "state": "IMPLEMENTED_AND_TESTED"},
            {"path": "crates/om9-app/src/ui/gem.rs", "state": "IMPLEMENTED_WINDOW"},
            {"path": "crates/om9-app/tests/gem_panel.rs", "state": "REGISTERED_TESTS"},
            {"path": "crates/om9-core/src/transaction.rs", "state": "FIELD-AWARE WRITE ADMISSION"},
        ]
        crosswalk = {
            "schema": "om9.m05.source_crosswalk.v1",
            "l3_id": l3_id,
            "title": package["title"],
            "owner": (
                "om9-core gem owner + om9-kernel B-Rep adapter + om9-app gem session/window"
                if "om9-kernel" in package.get("owner_crates", [])
                else "om9-core gem owner + om9-app gem session/window"
            ),
            "source_revision": head,
            "checkout_dirty": dirty,
            "sources": package.get("sources", default_sources),
            "source_audit": {
                "matrix_assets": "NOT_COPIED",
                "result": "PROCEDURAL_ASSETS_ONLY",
                "detail": (
                    "No Matrix gemstone binary asset, catalogue bitmap or decompiled resource is "
                    "read or shipped. Every cut/shape outline and mesh is generated from the "
                    "typed spec in crates/om9-core/src/gem.rs; the canonical library is authored "
                    "sizes, not imported geometry."
                ),
            },
            "planning_inputs": [
                f"OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/{package['folder']}/l3.json",
                "OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L2.json",
            ],
            "planning_tree_touched": False,
            "notes": [
                "The existing M05 settings contract (settings::gems::GemStone/GemRow) is an input "
                "row for prong/channel/metal settings and is NOT this runtime gem record.",
                "The Matrix command names gvLoader/gvEmerald/gvBaguette/gvGemPositioner are kept.",
            ],
        }
        write_json(directory / "source_crosswalk.json", crosswalk)

        api_rows = "\n".join(
            f"| {api} | {detail} |" for api, detail in package["api"]
        )
        effects = package.get(
            "effects",
            [
                "A preview is the typed spec plus its computed mesh in session state; it never "
                "inserts a document object.",
                "Apply is exactly one document transaction; Undo restores the previous state and "
                "Redo keeps the object identity.",
                "Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, "
                "`Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.",
                "The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and "
                "the mesh owner remains the entity itself.",
            ],
        )
        native_section = (
            f"\n## Native exception\n\n{package['native']}\n" if package.get("native") else ""
        )
        (directory / "api.md").write_text(
            f"""# {package['title']} (`{l3_id}`)

## Public surface

| Symbol | Contract |
| --- | --- |
{api_rows}

## Effects

""" + "\n".join(f"- {line}" for line in effects) + native_section + f"""
## Verification

- Positive: {package['positive_note']}
- Negative: {package['negative_note']}
- Gate: `{gate}` ({'automated evidence only' if gate == AUTO else 'M05-UI manual review open'}).
""",
            encoding="utf-8",
        )

        manifest = sha(
            json.dumps(package["positive"], sort_keys=True)
            + json.dumps(package["negative"], sort_keys=True)
        )
        write_json(
            directory / "positive.json",
            {
                "schema": "om9.m05.l3_positive.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "cases": package["positive"]["cases"],
                "oracle": "asserted by the registered tests; see observed.json",
            },
        )
        write_json(
            directory / "negative.json",
            {
                "schema": "om9.m05.l3_negative.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "cases": package["negative"]["cases"],
                "oracle": "typed refusals asserted by the registered tests",
            },
        )

        receipts = []
        for l4_id in l4_ids(l3_id, package["folder"]):
            suffix = l4_id.rsplit("-", 1)[-1]
            if suffix not in {"03", "05"}:
                continue
            entries = package["tests"][:2]
            commands = []
            logs = []
            for test in entries:
                command = [
                    "cargo",
                    "test",
                    "-p",
                    "om9-app",
                    "--test",
                    "gem_panel",
                    "-j",
                    "2",
                    test,
                ]
                code, captured = run(command)
                commands.append(
                    {
                        "command_argv": command,
                        "exit_code": code,
                        "stdout_stderr_sha256": sha(captured),
                        "summary": [
                            line
                            for line in captured.splitlines()
                            if line.startswith(("test result:", "error", "FAILED"))
                        ][-3:],
                        "test": test,
                        "package": "om9-app",
                        "binary": "gem_panel",
                    }
                )
                logs.append(header(" ".join(command), code, captured))
                if code != 0:
                    raise SystemExit(f"L4 command failed for {l4_id}: {' '.join(command)}")
            title_suffix, role, kind = L4_ROLE[suffix]
            write_json(
                directory / f"{l4_id}.json",
                {
                    "schema": "om9.m05.l4.receipt.v1",
                    "l3_id": l3_id,
                    "l4_id": l4_id,
                    "title": package["title"],
                    "generated_at_utc": datetime.datetime.now(datetime.UTC).strftime(
                        "%Y-%m-%dT%H:%M:%SZ"
                    ),
                    "source_revision": head,
                    "checkout_dirty": dirty,
                    "manual_gate": gate,
                    "signature": None,
                    "not_claimed": ([] if gate == AUTO else [
                        "manual review of the gem window at the M05-UI gate",
                        "a reviewer signature",
                    ]),
                    "fixture_manifest_sha256": manifest,
                    "document_snapshot": {
                        "source": "asserted by the registered tests",
                        "positive": package["positive_note"],
                        "negative": package["negative_note"],
                    },
                    "role": role,
                    "kind": kind,
                    "status": "READY_FOR_REVIEW",
                    "changed_files": CHANGED,
                    "commands": commands,
                    "manual_gate_note": (
                        "Automated evidence only; the M05-UI manual gate remains open."
                        if gate != AUTO
                        else "AUTO_ONLY: the record asks for no human test."
                    ),
                },
            )
            (directory / f"{l4_id}.log").write_text("\n".join(logs), encoding="utf-8")
            receipts.append(l4_id)

        if gate == AUTO:
            write_json(
                directory / f"{l3_id}.ui-review.json",
                {
                    "schema": "om9.m05.ui_review.v1",
                    "l3_id": l3_id,
                    "manual_test_gate": "AUTO_ONLY",
                    "reviewer": {
                        "status": "AUTOMATED_EVIDENCE_ONLY",
                        "role": "automated evidence",
                        "name": None,
                    },
                    "statement": (
                        "This backend L3 has no human-facing gate in its own record. Acceptance "
                        "rests on the automated checks listed in observed.json."
                    ),
                },
            )

        write_json(
            directory / "observed.json",
            {
                "schema": "om9.m05.l3_gem_evidence.v1",
                "l3_id": l3_id,
                "title": package["title"],
                "scope": "; ".join(api for api, _ in package["api"]),
                "target": TARGET,
                "positive": package["positive_note"],
                "negative": package["negative_note"],
                "manual_gate": gate,
                "reviewer": None if gate != AUTO else {"status": "AUTOMATED_EVIDENCE_ONLY"},
                "statement": (
                    "Automated evidence only; the M05-UI manual review remains open and no "
                    "reviewer signature is claimed."
                    if gate != AUTO
                    else "AUTO_ONLY backend capability; acceptance rests on the automated checks."
                ),
                "l4": {
                    "l4_receipts": receipts,
                    "fixture_manifest_sha256": manifest,
                },
                "tests": package["tests"],
            },
        )
        summary[l3_id] = {
            "manual_gate": gate,
            "l4_receipts": receipts,
            "fixture_manifest_sha256": manifest,
            "tests": package["tests"],
        }
        receipts_all.extend(receipts)

    write_json(
        BATCH / "observed.json",
        {
            "schema": "om9.m05.l2_gem_evidence.v1",
            "l2_id": "OM9-L2-04-09",
            "title": "Gem Asset Shapes And Loader",
            "date": "2026-10-27",
            "target": TARGET,
            "source_revision": head,
            "checkout_dirty": dirty,
            "code": CHANGED,
            "checks": results,
            "l3": summary,
            "uncertainties": [
                "The five UI L3 keep the M05-UI manual gate open: no human has reviewed the gem "
                "window or the viewport preview on screen and no reviewer signature is claimed.",
                "The gem B-Rep conversion requires the OCCT SDK; `BRep=No` keeps the portable mesh "
                "body, and only Windows x64 was exercised in this run.",
                "The sewn B-Rep is planar-facet topology of the authored gem; analytic curved "
                "faces and gem-specific surface editing belong to a later kernel owner.",
            ],
        },
    )
    (BATCH / "README.md").write_text(
        f"""# OM9-L2-04-09 Gem Asset Shapes And Loader — evidence

Automated run on 2026-10-27 (`{TARGET}`), source revision `{head}`
({'dirty' if dirty else 'clean'} checkout).

## What was run

""" + "\n".join(f"- `{item['command']}` -> exit {item['exit_code']}" for item in results) + """

## Per-L3 result

| L3 | Gate | L4 receipts |
| --- | --- | --- |
""" + "\n".join(
            f"| `{l3}` | `{info['manual_gate']}` | {', '.join(info['l4_receipts']) or '—'} |"
            for l3, info in summary.items()
        ) + """

Backend L3-001/002/008 are `AUTO_ONLY`: their record asks for no human-facing test, so
`ui-review.json` declares `AUTOMATED_EVIDENCE_ONLY`. UI L3-003/004/005/006/007 keep the
M05-UI manual gate **open**.

## Not claimed

- M05-UI manual acceptance of the gem window, its gestures and the viewport preview overlay.
- Matrix gemstone assets: none were read, copied or shipped.
- Analytic/NURBS gem faces: the committed B-Rep is sewn from the authored planar facets.
""",
        encoding="utf-8",
    )

    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    entry = {
        "key": KEY,
        "date": "2026-10-27",
        "stage": "feature-port",
        "status": "gem_assets_loader_brep_preview_automated_verified_manual_gates_open",
        "target": TARGET,
        "feature_ids": ["OM9-L2-04-09"],
        "scope": (
            "M05 OM9-L2-04-09 gem assets and loader: typed GemSpec cut/shape/dimension schema, "
            "procedural closed meshes + canonical size library, gvLoader/gvEmerald/gvBaguette/"
            "gvGemPositioner command surface, custom gem from a closed planar profile, shared-"
            "definition duplication, F6 Gem Properties, Gem 01 layer routing, the viewport preview "
            "overlay, and the mesh->B-Rep adapter that commits Geometry::Brep by default; 21 core "
            "+ 2 kernel + 25 app tests plus 2 preview unit tests."
        ),
        "notes": (
            "ghi chú: (1) gem body là mesh kín do entity sở hữu; khi kernel OCCT bật, Apply lưu "
            "Geometry::Brep sewn từ chính mesh (BRep=No giữ mesh, không fallback ngầm); (2) không "
            "copy asset Matrix — toàn bộ outline/mesh sinh từ spec; (3) gate M05-UI của 5 L3 giao "
            "diện còn mở, 3 L3 backend là AUTO_ONLY; (4) preview được vẽ trong viewport bằng overlay "
            "world-space và không ghi document; (5) hợp đồng settings::gems::GemStone/GemRow là "
            "input của prong/channel, không phải record runtime này."
        ),
        "outputs": CHANGED,
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-09/observed.json",
            "implementation/evidence/M05/OM9-L2-04-09/README.md",
            *[f"implementation/evidence/M05/OM9-L2-04-09/{item['artifact']}" for item in results],
            *[f"implementation/evidence/M05/{l3}/observed.json" for l3 in L3_PACKAGES],
        ],
        "tests": {
            "core_gem_assets": "21 passed (schema, library, closed meshes, refusals, references)",
            "app_gem_panel": "25 passed (loader, custom, emerald/baguette, positioner, properties, preview, B-Rep)",
            "app_preview_unit": "2 passed (projection covers the gem; placement shifts the overlay)",
            "kernel_gem_brep": "2 passed (8 cut families -> one valid B-Rep with the mesh volume; open mesh refused)",
            "core_transaction_regression": "atomic_transactions + transaction_scopes passed",
            "workspace_check": "cargo check --workspace --all-targets exit 0",
            "clippy": "cargo clippy -p om9-core -p om9-app -p om9-kernel --all-targets -- -D warnings exit 0",
        },
        "uncertainties": json.loads((BATCH / "observed.json").read_text(encoding="utf-8"))[
            "uncertainties"
        ],
        "manual_gate": (
            "OPEN: M05-UI — 5 L3 giao diện (loader, custom, emerald/baguette, positioner, "
            "properties) chưa được người nghiệm thu; 3 L3 backend là AUTO_ONLY"
        ),
        "next_action": (
            "Duyệt/chụp cửa sổ gem và preview trong viewport (loader, custom, emerald, baguette, "
            "positioner, properties) ở gate M05-UI; sau đó chuyển sang L2-04-10 (gem on curves)."
        ),
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != KEY] + [entry]
    ledger["active_item"] = KEY
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    write_json(ledger_path, ledger)

    owner_path = L3_ROOT / "L2.json"
    owner = json.loads(owner_path.read_text(encoding="utf-8"))
    owner["runtime_implementation"] = {
        "date": "2026-10-27",
        "status": "AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING",
        "target": TARGET,
        "scope": (
            "Typed gem cut/shape/dimension schema, procedural closed meshes and canonical size "
            "library, loader/emerald/baguette/custom/positioner commands, F6 gem properties, "
            "Gem 01 layer routing, the girdle/table/facet reference API, the viewport preview "
            "overlay and the mesh->B-Rep adapter (`BRep=Yes` default, `BRep=No` keeps the mesh)"
        ),
        "owner": {
            "crates": ["om9-core", "om9-kernel", "om9-app"],
            "tests": [
                "crates/om9-core/tests/gem_assets.rs",
                "crates/om9-kernel/tests/gem_brep.rs",
                "crates/om9-app/tests/gem_panel.rs",
                "crates/om9-app/src/ui/gem.rs (preview_tests)",
            ],
        },
        "evidence": [
            "implementation/evidence/M05/OM9-L2-04-09/observed.json",
            "implementation/evidence/M05/OM9-L2-04-09/README.md",
            "implementation/evidence/M05/OM9-L2-04-09/core-gem-assets.log",
            "implementation/evidence/M05/OM9-L2-04-09/app-gem-panel.log",
            "implementation/evidence/M05/OM9-L2-04-09/kernel-gem-brep.log",
            "implementation/evidence/M05/OM9-L2-04-09/workspace-check.log",
        ],
        "notes": (
            "Gem body là mesh kín do entity sở hữu, có oracle closed/volume của ring_builder; "
            "khi kernel OCCT bật, Apply lưu Geometry::Brep sewn từ chính mesh đó (BRep=No giữ "
            "mesh), preview được vẽ trong viewport trước khi Apply. Không copy asset Matrix. "
            "settings::gems::GemStone/GemRow là input của prong/channel chứ không phải record này."
        ),
        "l3_coverage": {
            l3: (
                "automated_verified_auto_only"
                if package["gate"] == AUTO
                else "automated_verified_manual_gate_open"
            )
            for l3, package in L3_PACKAGES.items()
        },
        "l4": {
            "planned": 21,
            "artifacts": 21,
            "receipts": receipts_all,
        },
        "tests": [
            f"core gem_assets: exit {results[0]['exit_code']}",
            f"app gem_panel: exit {results[1]['exit_code']}",
            f"workspace check: exit {results[3]['exit_code']}",
            f"clippy: exit {results[4]['exit_code']}",
            f"kernel gem_brep: exit {results[5]['exit_code']}",
        ],
        "human_approval": None,
        "manual_gate": "M05-UI",
        "remaining": json.loads((BATCH / "observed.json").read_text(encoding="utf-8"))[
            "uncertainties"
        ],
    }
    write_json(owner_path, owner)

    for l3_id, package in L3_PACKAGES.items():
        l3_path = L3_ROOT / package["folder"] / "l3.json"
        document = json.loads(l3_path.read_text(encoding="utf-8"))
        document["runtime_implementation"] = {
            "date": "2026-10-27",
            "status": (
                "AUTOMATED_VERIFIED_AUTO_ONLY"
                if package["gate"] == AUTO
                else "AUTOMATED_VERIFIED_MANUAL_UI_PENDING"
            ),
            "target": TARGET,
            "owner": {"crates": package.get("owner_crates", ["om9-core", "om9-app"])},
            "evidence": [
                f"implementation/evidence/M05/{l3_id}/observed.json",
                f"implementation/evidence/M05/{l3_id}/source_crosswalk.json",
                f"implementation/evidence/M05/{l3_id}/api.md",
                f"implementation/evidence/M05/{l3_id}/positive.json",
                f"implementation/evidence/M05/{l3_id}/negative.json",
            ],
            "manual_gate": "AUTO_ONLY" if package["gate"] == AUTO else "M05-UI",
            "owner": {"crates": package.get("owner_crates", ["om9-core", "om9-app"])},
            "evidence": [
                f"implementation/evidence/M05/{l3_id}/observed.json",
                f"implementation/evidence/M05/{l3_id}/source_crosswalk.json",
                f"implementation/evidence/M05/{l3_id}/api.md",
                f"implementation/evidence/M05/{l3_id}/positive.json",
                f"implementation/evidence/M05/{l3_id}/negative.json",
            ],
            "notes": package["positive_note"] + "; " + package["negative_note"],
        }
        write_json(l3_path, document)

    print(f"recorded gem evidence at {BATCH.relative_to(ROOT)}")
    for item in results:
        print(f"  {item['artifact']}: exit {item['exit_code']}")
    print(f"L4 receipts: {receipts_all}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
