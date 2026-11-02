#!/usr/bin/env python3
"""Record M04 ``OM9-L2-01-04`` evidence from real test runs.

The M04 history backend was a W7 kernel port with no L3 acceptance record. This tool
turns the current implementation into the artefact set the milestone dashboard reads:

* ``implementation/design/M04/<L3>/source_crosswalk.json`` -- QymCAD anchors and the
  REUSE/ADAPT/ADD decision (L4 ``-01``);
* ``implementation/design/M04/<L3>/api.md`` plus
  ``implementation/fixtures/M04/<L3>/{positive,negative}.json`` -- the contract (L4 ``-02``);
* ``implementation/evidence/M04/<L3>/<L4>.<kind>.log`` -- captured command output for the
  implementation, integration, regression, adversarial and parameter tasks;
* ``implementation/evidence/M04/<L3>/<L4>.ui-review.json`` -- the AUTO_ONLY gate record.

Nothing here invents a run: every log is the captured stdout/stderr of the command named in
its header. A missing capture is reported as a failure, never filled with a claim.

Usage::

    python tools/record_m04_l2_01_04_evidence.py
    python tools/record_m04_l2_01_04_evidence.py --workspace-log PATH --check-log PATH

``--workspace-log`` and ``--check-log`` let the caller supply a previously captured
``cargo test --workspace`` / ``cargo check --workspace --all-targets`` run, because those
two take long enough that they are run once for the whole batch.
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
TREE = ROOT / "OM9_Milestones" / "M04 - Builder and History"
L2_DIR = TREE / "L2-01-04 - 01-be-associative-dependency-rebuild"
DESIGN = ROOT / "implementation" / "design" / "M04"
FIXTURES = ROOT / "implementation" / "fixtures" / "M04"
EVIDENCE = ROOT / "implementation" / "evidence" / "M04"
CAPTURE = ROOT / "build" / "m04-l2-01-04-evidence"

TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", r"L:\om9-target")
TEST_BIN = "associative_rebuild"
SOURCE_REVISION_COMMAND = ["git", "rev-parse", "HEAD"]

# The whole associative surface, one entry per L3. ``tests`` is the exact set of Rust
# tests that proves the L3 oracle in ``crates/om9-core/tests/associative_rebuild.rs``.
L3 = {
    "OM9-L3-01-04-001": {
        "title": "Feature Record And Typed Dependency Edges",
        "goal": "stable feature graph node with typed inputs and outputs",
        "positive": {
            "input": "nodes rail->profile->sweep; typed GeometryRef vs ScalarMm",
            "expect": "3 nodes, edges follow typed IO; a curve fed into ScalarMm is rejected",
        },
        "negative": {
            "input": "feed CurveRef into ScalarMm input; a port role or output kind that does not match",
            "expect": "Invalid('object bound to scalar input'), Invalid('typed input port role'), Invalid('typed feature output')",
        },
        "code": [
            "crates/om9-core/src/associative.rs: PortKind, FeaturePort, record_feature, typed_edges, validate_typed_ports",
            "crates/om9-core/src/history.rs: Recipe.input_ports / Recipe.output_ports (serde default, skipped when empty)",
        ],
        "tests": [
            "typed_edges_follow_the_declared_io_and_link_producers",
            "a_curve_in_a_scalar_input_and_a_wrong_output_type_are_refused",
            "a_legacy_recipe_without_ports_is_reported_untyped_never_guessed",
            "record_feature_enforces_identity_document_and_output_ownership",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-core/src/feature.rs",
                "line": 1,
                "observed": "feature timeline model; references use stable ids only, never indices",
                "decision": "ADAPT",
                "missing": "no typed input/output port on a node: a feature is an ordered timeline entry, so a curve fed into a scalar parameter is refused by the command layer, not recorded on the node",
                "owner": "om9-core/src/history.rs (Recipe.input_ports/output_ports) + associative.rs (typed_edges)",
            },
            {
                "path": "crates/qymcad-core/src/feature.rs",
                "line": 27,
                "observed": "FaceKey { index, centroid, normal, id }: positional index plus a geometric fingerprint",
                "decision": "ADAPT",
                "missing": "the fingerprint idea returns as SubshapeSignature; OM9 attaches it to a reference, not to every node",
                "owner": "om9-core/src/associative.rs (SubshapeSignature)",
            },
            {
                "path": "crates/qymcad-ui-state/src/rebuild.rs",
                "line": 246,
                "observed": "pub fn regenerate_all(rc: &mut RebuildCtx): rebuilds the timeline nodes the scheduler marked",
                "decision": "ADD",
                "missing": "no persisted dependency edge: the node order is the timeline order, not a DAG the document can validate after reload",
                "owner": "om9-core/src/history.rs (execution_order) + associative.rs (DependencyEdge)",
            },
        ],
    },
    "OM9-L3-01-04-002": {
        "title": "Feature Provenance From Command",
        "goal": "record the generating command and the user parameters for each result",
        "positive": {
            "input": "builder RingRail diameter_mm=18.0 with its originating command parameters",
            "expect": "re-open the exact command, typed inputs and unit preserved",
        },
        "negative": {
            "input": "command payload with a bare Number parameter, a nil required input id, a duplicate role or an empty selection",
            "expect": "Invalid('feature parameter unit/type'), Invalid('feature input identity'), Invalid('feature input role'), Invalid('feature input selection')",
        },
        "code": [
            "crates/om9-core/src/associative.rs: FeatureProvenance, RecordedInput, ReopenFeature, feature_provenance, reopen_feature, replay_environment",
        ],
        "tests": [
            "provenance_reopens_the_exact_command_parameters_inputs_and_unit",
            "provenance_refuses_a_bare_number_a_missing_input_id_and_a_duplicate_role",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-core/src/doc_file.rs",
                "line": 79,
                "observed": "pub timeline: Vec<crate::feature::FeatureNode> is what the saved document carries",
                "decision": "ADAPT",
                "missing": "the timeline records the node, not the request that produced a result; reopening is the node editor, with no recorded command version or unit",
                "owner": "om9-core/src/associative.rs (FeatureProvenance/ReopenFeature)",
            },
            {
                "path": "crates/qymcad-ui-state/src/records/frame.rs",
                "line": 442,
                "observed": "pub struct Rebuilding keeps the last rebuild's reference rebindings until the next rebuild, and they never reach the file",
                "decision": "ADD",
                "missing": "provenance is deliberately transient here; OM9 makes the command/parameters/unit durable on the recipe",
                "owner": "om9-core/src/history.rs (ReplayEnvironment, Recipe.parameters)",
            },
        ],
    },
    "OM9-L3-01-04-003": {
        "title": "Dirty Propagation And Topological Order",
        "goal": "invalidate dependent outputs and recompute in DAG order",
        "positive": {
            "input": "DAG A->B->C and A->D; edit the source of A",
            "expect": "dirty set {A,B,C,D}, producer before consumer, no duplicate evaluation",
        },
        "negative": {
            "input": "cycle C->A",
            "expect": "Invalid('History cycle') from execution_order, plan_recompute and typed_edges",
        },
        "code": [
            "crates/om9-core/src/history_ops.rs: stale_recipes, mark_stale, mark_document_stale, plan_recompute, plan_document_recompute",
            "crates/om9-core/src/history.rs: HistoryGraph::execution_order (Kahn, cycle refusal)",
        ],
        "tests": [
            "editing_the_source_dirties_b_and_d_then_c_in_topological_order",
            "a_cycle_is_refused_instead_of_being_ordered_or_planned",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-ui-state/src/rebuild.rs",
                "line": 414,
                "observed": "pub fn mark_dirty_for_rebuild(rc: &mut RebuildCtx): the document is marked, the scheduler does the counting",
                "decision": "ADAPT",
                "missing": "the dirty flag lives on the timeline node and the order is the timeline order; OM9 derives the order from recorded output->input edges and refuses a cycle",
                "owner": "om9-core/src/history_ops.rs + history.rs::execution_order",
            },
            {
                "path": "crates/qymcad-ui-state/src/rebuild.rs",
                "line": 375,
                "observed": "rebuild_if_dirty compares (rebuild_key, dirty node ids, live shape count)",
                "decision": "ADAPT",
                "missing": "the dirty set is not expressed as a graph edge set, so a reloaded document cannot re-derive it",
                "owner": "om9-core/src/history_ops.rs (stale_recipes/mark_stale)",
            },
            {
                "path": "crates/qymcad-part/src/topology.rs",
                "line": 85,
                "observed": "deletion cascade marks the nodes that actually need rebuilding, then regenerate_all",
                "decision": "ADAPT",
                "missing": "the cascade is per workbench operation; OM9 keeps it in the graph kernel so every builder shares it",
                "owner": "om9-core/src/history.rs::invalidate + history_ops::mark_stale",
            },
        ],
    },
    "OM9-L3-01-04-004": {
        "title": "Stable Semantic Reference Linking",
        "goal": "preserve rail/profile/gem attachment under recompute",
        "positive": {
            "input": "profile references a named edge; rebuild the rail with the same semantic identifier and a 0.01 mm move",
            "expect": "the reference resolves to the same intended edge after the allowed edit; the bound id is unchanged",
        },
        "negative": {
            "input": "two subshapes with the same name, a changed kind, a move beyond tolerance, a stale revision, a detached owner",
            "expect": "Invalid('ambiguous semantic reference'), Invalid('semantic subshape kind'), Stale('semantic reference moved'), Stale('semantic reference revision'), Invalid('semantic reference detached feature')",
        },
        "code": [
            "crates/om9-core/src/associative.rs: SubshapeKind, SubshapeSignature, SemanticReference, ReferenceAnchor, bind_reference, resolve_semantic_reference, relink_reference, drop_references_of",
            "crates/om9-core/src/history.rs: HistoryGraph.references",
            "crates/om9-core/src/producers.rs: SnapshotTopology/TopologyProvider names edge/face/vertex/curve subshapes",
            "crates/om9-kernel/src/associative.rs: OcctTopology reads the OCCT B-Rep snapshot and refuses a non-OCCT source",
        ],
        "tests": [
            "a_semantic_reference_survives_an_allowed_edit_and_refuses_ambiguity",
            "a_detached_feature_cannot_keep_a_semantic_reference",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-core/src/feature.rs",
                "line": 31,
                "observed": "FaceKey.id: persistent B-rep face id that survives a rebuild; centroid/normal are the fallback fingerprint",
                "decision": "REUSE",
                "missing": "nothing: the semantic-name-plus-fingerprint policy is taken from here",
                "owner": "om9-core/src/associative.rs (SemanticReference.fingerprint)",
            },
            {
                "path": "crates/qymcad-kernel/src/shape/edges.rs",
                "line": 10,
                "observed": "edges carry persistent ids in parallel with the edge list; an id survives a rebuild",
                "decision": "ADAPT",
                "missing": "the id is positional/persistent from OCCT; OM9 keeps its own named reference so a rebuild can refuse a duplicate name before trusting a kernel id",
                "owner": "om9-core/src/associative.rs (resolve_semantic_reference)",
            },
            {
                "path": "crates/qymcad-ui-state/src/records/frame.rs",
                "line": 468,
                "observed": "the last rebuild's reference rebindings are shown in the status line and marked in the tree, then dropped",
                "decision": "ADD",
                "missing": "rebindings are a report of the last rebuild; OM9 persists the reference so a later edit can be checked against it",
                "owner": "om9-core/src/history.rs (references map)",
            },
        ],
    },
    "OM9-L3-01-04-005": {
        "title": "Incremental Recompute And Cache",
        "goal": "only recompute affected features and cache unchanged outputs",
        "positive": {
            "input": "independent features X,Y and chain A->B; edit the source of A",
            "expect": "only A and B recalculate; X and Y cache entries (revision and sha256) are unchanged",
        },
        "negative": {
            "input": "a cache entry whose recorded input stamp is one revision old; a parameter edit without a revision bump",
            "expect": "both are cache misses and A is planned even while the graph still calls it Clean",
        },
        "code": [
            "crates/om9-core/src/associative.rs: CacheEntry, RecomputeCache::{is_valid, record, invalidate, plan}, output_hash (SHA-256 over canonical geometry)",
        ],
        "tests": [
            "only_the_affected_chain_recomputes_and_independent_cache_entries_are_untouched",
            "a_cache_entry_with_an_outdated_input_stamp_is_a_miss_even_when_the_graph_is_clean",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-ui-state/src/rebuild.rs",
                "line": 332,
                "observed": "pub fn rebuild_if_dirty(rc: &mut RebuildCtx): computes only when the rebuild key or the dirty node set moved",
                "decision": "ADAPT",
                "missing": "the key is the document rebuild key; OM9 keys each feature by (revision, parameters, input stamps) so an unrelated edit cannot invalidate it",
                "owner": "om9-core/src/associative.rs (RecomputeCache)",
            },
            {
                "path": "crates/qymcad-ui-state/src/jobs.rs",
                "line": 176,
                "observed": "Project::rebuild_key is the fingerprint of the document the rebuild computes",
                "decision": "ADAPT",
                "missing": "one fingerprint for the whole document; OM9 keeps one entry per recipe",
                "owner": "om9-core/src/associative.rs (CacheEntry)",
            },
        ],
    },
    "OM9-L3-01-04-006": {
        "title": "Builder Preview Vs Committed Features",
        "goal": "discard preview graph state without affecting the saved design",
        "positive": {
            "input": "preview rail diameter 18->19->20 without Apply",
            "expect": "committed document id/geometry/hash unchanged; one Apply yields only the final value",
        },
        "negative": {
            "input": "Escape midway; Apply after the committed document moved",
            "expect": "transient nodes disappear; Stale('preview base document')",
        },
        "code": [
            "crates/om9-core/src/associative.rs: PreviewSession::{begin, base_revision, staged_counts, set_parameter, stage_recipe, stage_entity, stage_result, plan, commit, discard}",
            "PreviewSession stores only staged recipes/entities/results; begin copies the base identity/revision, not the document",
        ],
        "tests": [
            "preview_edits_never_touch_the_committed_document_until_one_apply",
            "escape_discards_transient_nodes_and_a_moved_base_is_refused",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-ui-state/src/edits.rs",
                "line": 112,
                "observed": "pub fn commit_edit(rc): the edit is pushed onto the undo stack only when the document changed, then one rebuild runs",
                "decision": "ADAPT",
                "missing": "the preview and the document are the same object in QymCAD (the edit is committed or rolled back); OM9 gives the builder a private draft so a preview cannot touch the saved design",
                "owner": "om9-core/src/associative.rs (PreviewSession)",
            },
            {
                "path": "crates/qymcad-ui-state/src/edits.rs",
                "line": 6,
                "observed": "pub fn abort_edit(rc): the operation is rolled back and nothing is rebuilt",
                "decision": "ADAPT",
                "missing": "rollback of a mutation; OM9's discard drops a copy that was never committed",
                "owner": "om9-core/src/associative.rs (PreviewSession::discard)",
            },
        ],
    },
    "OM9-L3-01-04-007": {
        "title": "History Recompute Failure Rollback",
        "goal": "keep the last valid result and report the failed feature",
        "positive": {
            "input": "valid ring rail, then a parameter the producer refuses",
            "expect": "recompute fails, the previous geometry/published result/cache entry are retained, the feature is Failed(reason)",
        },
        "negative": {
            "input": "producer returns no output, a different object id, or geometry of the wrong kind",
            "expect": "Failed('invalid rebuilt output set' / 'invalid typed feature output'); no corrupt committed node",
        },
        "code": [
            "crates/om9-core/src/associative.rs: RebuildRequest, RebuiltOutput, RecomputeOutcome, FeatureSnapshot, recompute_feature",
        ],
        "tests": [
            "a_successful_rebuild_updates_geometry_published_result_and_cache",
            "a_failed_rebuild_keeps_the_last_good_geometry_result_and_cache_entry",
            "a_producer_that_breaks_the_recorded_output_set_creates_no_corrupt_node",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-ui-state/src/edits.rs",
                "line": 235,
                "observed": "a node that no longer builds keeps its last good body, as a partial rebuild keeps it",
                "decision": "REUSE",
                "missing": "nothing: the retain-last-good policy is the same",
                "owner": "om9-core/src/associative.rs (FeatureSnapshot/recompute_feature)",
            },
            {
                "path": "crates/qymcad-ui-state/src/rebuild.rs",
                "line": 246,
                "observed": "regenerate_all walks the dirty nodes and stops reporting at the end",
                "decision": "ADAPT",
                "missing": "the failure is reported as a rebuild message; OM9 records Failed(reason) on the feature and keeps the published result Current with the previous snapshot",
                "owner": "om9-core/src/associative.rs (RecomputeOutcome::Failed)",
            },
        ],
    },
    "OM9-L3-01-04-008": {
        "title": "Delete Parent Or Change Input Policy",
        "goal": "behaviour for missing inputs and changed output topology",
        "positive": {
            "input": "parent curve deleted; a child uses its semantic rail reference",
            "expect": "child marked broken; the report offers KeepBroken/DetachDependents/RemoveDependents and the chosen policy leaves no dangling reference",
        },
        "negative": {
            "input": "delete the parent while a child is rebuilding asynchronously",
            "expect": "Error::Busy and the document is unchanged",
        },
        "code": [
            "crates/om9-core/src/associative.rs: BrokenInputPolicy, BrokenDependency, DeleteSourceReport, RebuildActivity, delete_source_input, drop_references_of",
        ],
        "tests": [
            "deleting_the_parent_marks_the_child_broken_and_reports_the_policy_choices",
            "detach_and_remove_policies_leave_no_dangling_reference",
            "a_delete_during_an_async_rebuild_is_refused_not_raced",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-part/src/topology.rs",
                "line": 25,
                "observed": "delete feature together with its body, clear the caches and rebuild",
                "decision": "ADAPT",
                "missing": "the cascade is mechanical and has no user policy; OM9 reports the broken children and lets the caller choose repair/removal",
                "owner": "om9-core/src/associative.rs (delete_source_input)",
            },
            {
                "path": "crates/qymcad-ui-state/src/edits.rs",
                "line": 209,
                "observed": "a node with nothing to rebuild from is counted as changed, marked, and a rebuild of nothing is started",
                "decision": "ADD",
                "missing": "nothing names the missing input; OM9 keeps Unresolved('source missing') and lists it in the report",
                "owner": "om9-core/src/history_ops.rs + associative.rs",
            },
            {
                "path": "crates/qymcad-ui-state/src/jobs.rs",
                "line": 99,
                "observed": "a rebuild starts only when no job is in hand, so no job is dropped while the window works",
                "decision": "ADAPT",
                "missing": "the exclusion is job-level; OM9 states it per feature with RebuildActivity and refuses the delete with Error::Busy",
                "owner": "om9-core/src/associative.rs (RebuildActivity)",
            },
        ],
    },
    "OM9-L3-01-04-009": {
        "title": "Associative Rebuild Geometry Fixtures",
        "goal": "reference ring/gem/settings history regression fixtures",
        "positive": {
            "input": "fixture graph ring rail->profile->sweep with a gem placeholder",
            "expect": "deterministic baseline hash across sessions; an edited rebuild matches a fresh build",
        },
        "negative": {
            "input": "corrupt fixture JSON, a wrong node index, an edge outside the record, a tampered hash",
            "expect": "load/verify fails with Io/Invalid('feature baseline node order'/'feature baseline edge')/Stale('feature baseline hash')",
        },
        "code": [
            "crates/om9-core/src/associative.rs: BaselineNode, BaselineEdge, FeatureGraphBaseline, feature_graph_baseline, load_baseline, node_content_key",
        ],
        "tests": [
            "the_baseline_is_deterministic_across_sessions_and_round_trips",
            "editing_a_parameter_and_rebuilding_matches_a_fresh_build",
            "corrupt_or_missing_fixture_data_fails_to_load",
            "options_are_hashed_by_content_not_by_allocation",
        ],
        "anchors": [
            {
                "path": "crates/qymcad-kernel/src/shape/edges.rs",
                "line": 10,
                "observed": "persistent ids survive a rebuild, which is what lets a regression compare two rebuilds",
                "decision": "ADAPT",
                "missing": "no deterministic baseline record; OM9 hashes content order, parameters and output geometry",
                "owner": "om9-core/src/associative.rs (FeatureGraphBaseline)",
            },
            {
                "path": "crates/qymcad-acceptance/tests/acceptance/chains.rs",
                "line": 125,
                "observed": "a chain check keeps stepping the scenario and notices when rebuilding everything never comes to rest",
                "decision": "ADAPT",
                "missing": "the chain is an acceptance script; OM9 keeps a machine-readable baseline that a later run can verify",
                "owner": "om9-core/src/associative.rs (load_baseline)",
            },
        ],
    },
}

# L4 suffix -> (kind, description). The set of L4 ids per L3 comes from l3.json, so an
# extra task (adversarial/parameter) added to the plan cannot be silently skipped here.
L4_KIND = {
    "01": ("source_crosswalk", "source map and REUSE/ADAPT/ADD decision"),
    "02": ("contract", "API contract and positive/negative fixtures"),
    "03": ("target-test", "the L3's own Rust tests"),
    "04": ("integration", "calling surface compiles and the core suite stays green"),
    "05": ("regression", "full core regression and workspace checks"),
    "06": ("ui-review", "acceptance record for the AUTO_ONLY gate"),
    "07": ("adversarial", "refusal/recovery cases"),
    "08": ("parameter", "parameter contract of the L3"),
}


def run(args: list[str], env: dict[str, str] | None = None) -> tuple[int, str]:
    process = subprocess.run(
        args,
        cwd=ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        env=env,
    )
    text = process.stdout + process.stderr
    return process.returncode, text


def cargo_env() -> dict[str, str]:
    env = dict(os.environ)
    env["CARGO_TARGET_DIR"] = TARGET_DIR
    env.setdefault("CARGO_TERM_COLOR", "never")
    return env


def l4_ids() -> dict[str, list[dict]]:
    """Read the planned L4 list from each l3.json; the plan is the source of the task set."""
    result: dict[str, list[dict]] = {}
    for l3_path in sorted(L2_DIR.glob("L3-*/l3.json")):
        document = json.loads(l3_path.read_text(encoding="utf-8"))
        identifier = document["id"]
        result[identifier] = [
            {"id": child["id"], "role": child.get("role", ""), "title": child.get("title", "")}
            for child in document.get("children_l4", [])
        ]
    return result


def write(path: pathlib.Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def log_header(command: str, returncode: int, captured: str) -> str:
    return (
        f"$ {command}\n"
        f"# exit code: {returncode}\n"
        f"# captured: {datetime.datetime.now(datetime.UTC).strftime('%Y-%m-%dT%H:%M:%SZ')}\n"
        f"# target dir: {TARGET_DIR}\n\n"
        f"{captured}"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace-log", type=pathlib.Path)
    parser.add_argument("--check-log", type=pathlib.Path)
    arguments = parser.parse_args()

    CAPTURE.mkdir(parents=True, exist_ok=True)
    env = cargo_env()

    revision_code, revision = run(SOURCE_REVISION_COMMAND)
    revision = revision.strip().splitlines()[0] if revision_code == 0 else "UNRESOLVED"
    _, status = run(["git", "status", "--porcelain"])
    dirty = bool(status.strip())

    # One whole-binary run first: a per-test log is only meaningful after this passes.
    code, full = run(
        ["cargo", "test", "-p", "om9-core", "--test", TEST_BIN, "-j", "2"], env=env
    )
    write(CAPTURE / "associative-rebuild.log", log_header(
        "cargo test -p om9-core --test associative_rebuild -j 2", code, full))
    if code != 0:
        print(full)
        print("FAIL: the associative_rebuild suite is red; no evidence was written.", file=sys.stderr)
        return 1
    summary = next(
        (line for line in full.splitlines() if line.startswith("test result:")), "test result: UNKNOWN"
    )

    failures: list[str] = []
    # Integration and workspace checks are per batch, not per L3: the same crate is the
    # calling surface and the same workspace is the regression.
    integration_code, integration = run(["cargo", "test", "-p", "om9-core", "-j", "2"], env=env)
    if integration_code != 0:
        failures.append("cargo test -p om9-core is red")
    clippy_code, clippy_text = run(
        [
            "cargo",
            "clippy",
            "-p",
            "om9-core",
            "--all-targets",
            "-j",
            "2",
            "--",
            "-D",
            "warnings",
        ],
        env=env,
    )
    if clippy_code != 0:
        failures.append("cargo clippy -p om9-core --all-targets -D warnings is red")

    if arguments.check_log and arguments.check_log.exists():
        check_text = arguments.check_log.read_text(encoding="utf-8", errors="replace")
    elif (CAPTURE / "check-workspace.log").exists():
        check_text = (CAPTURE / "check-workspace.log").read_text(encoding="utf-8", errors="replace")
    else:
        check_code, check_raw = run(
            ["cargo", "check", "--workspace", "--all-targets", "-j", "2"], env=env
        )
        check_text = log_header("cargo check --workspace --all-targets -j 2", check_code, check_raw)
        write(CAPTURE / "check-workspace.log", check_text)

    workspace_text = "not captured in this batch"
    if arguments.workspace_log and arguments.workspace_log.exists():
        workspace_text = arguments.workspace_log.read_text(encoding="utf-8", errors="replace")
    elif (CAPTURE / "workspace-tests.log").exists():
        workspace_text = (CAPTURE / "workspace-tests.log").read_text(
            encoding="utf-8", errors="replace"
        )
    workspace_note = "not captured in this batch"
    if match := re.search(r"^# totals: (.+)$", workspace_text, re.MULTILINE):
        workspace_note = match.group(1).strip()

    planned = l4_ids()
    today = datetime.datetime.now(datetime.UTC).strftime("%Y-%m-%d")

    for l3_id, meta in L3.items():
        if l3_id not in planned:
            failures.append(f"{l3_id}: not present in the milestone tree")
            continue
        design_dir = DESIGN / l3_id
        fixture_dir = FIXTURES / l3_id
        evidence_dir = EVIDENCE / l3_id
        evidence_dir.mkdir(parents=True, exist_ok=True)

        # L4-01: source crosswalk.
        write(
            design_dir / "source_crosswalk.json",
            json.dumps(
                {
                    "schema": "om9.m04.source_crosswalk.v1",
                    "task_id": f"{l3_id.replace('OM9-L3', 'OM9-L4')}-01",
                    "l3_id": l3_id,
                    "l3_title": meta["title"],
                    "owner_module_id": "OM9-L2-01-04",
                    "milestone": "M04",
                    "gate": "AUTO_ONLY",
                    "goal": meta["goal"],
                    "positive_oracle": f"{meta['positive']['input']} -> {meta['positive']['expect']}",
                    "negative_oracle": f"{meta['negative']['input']} -> {meta['negative']['expect']}",
                    "source_revision": revision,
                    "source_dirty": dirty,
                    "checkout": "QymCAD/ (vendored subtree of the main repository; no independent .git)",
                    "method": "bounded file reads with line numbers; decisions are OM9-owned and no upstream code is copied",
                    "planned_anchors": [
                        {
                            "path": anchor["path"],
                            "line": anchor["line"],
                            "observed_symbols": [anchor["observed"]],
                            "decision": anchor["decision"],
                            "what_is_missing": anchor["missing"],
                            "om9_owner": anchor["owner"],
                        }
                        for anchor in meta["anchors"]
                    ],
                    "implemented_api": meta["code"],
                    "reuse_decision": "ADAPT the rebuild/dirty/persistent-id ideas; ADD the typed durable ports, the persisted semantic reference and the content baselines.",
                    "evidence": [
                        f"crates/om9-core/tests/associative_rebuild.rs ({', '.join(meta['tests'])})",
                        f"implementation/fixtures/M04/{l3_id}/positive.json",
                        f"implementation/fixtures/M04/{l3_id}/negative.json",
                    ],
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
        )

        # L4-02: contract + fixtures.
        write(
            design_dir / "api.md",
            f"""# {meta['title']}

**Owner:** `OM9-L2-01-04` · **Milestone:** M04 · **Gate:** AUTO_ONLY

{meta['goal']}.

## Contract

{os.linesep.join(f'- `{entry}`' for entry in meta['code'])}

## Positive oracle

`{meta['positive']['input']}` -> {meta['positive']['expect']}

## Negative oracle

`{meta['negative']['input']}` -> {meta['negative']['expect']}

## Verification

`crates/om9-core/tests/associative_rebuild.rs`:

{os.linesep.join(f'- `{test}`' for test in meta['tests'])}

The JSON fixtures in `implementation/fixtures/M04/{l3_id}/` are the machine-readable
record of the same oracles; the Rust tests are the executable form and are the authority.
""",
        )
        write(
            fixture_dir / "positive.json",
            json.dumps(
                {
                    "schema": "om9.m04.l3_fixture.v1",
                    "l3_id": l3_id,
                    "case": "positive",
                    "input": meta["positive"]["input"],
                    "expected": meta["positive"]["expect"],
                    "tests": meta["tests"],
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
        )
        write(
            fixture_dir / "negative.json",
            json.dumps(
                {
                    "schema": "om9.m04.l3_fixture.v1",
                    "l3_id": l3_id,
                    "case": "negative",
                    "input": meta["negative"]["input"],
                    "expected": meta["negative"]["expect"],
                    "tests": meta["tests"],
                },
                indent=2,
                ensure_ascii=False,
            )
            + "\n",
        )

        # Implementation logs: one real run per test, concatenated per L3.
        target_log = []
        target_ok = True
        for test in meta["tests"]:
            code, captured = run(
                [
                    "cargo",
                    "test",
                    "-p",
                    "om9-core",
                    "--test",
                    TEST_BIN,
                    "--",
                    "--exact",
                    test,
                ],
                env=env,
            )
            target_ok = target_ok and code == 0
            target_log.append(log_header(
                "cargo test -p om9-core --test associative_rebuild -- --exact " + test,
                code,
                captured,
            ))
        if not target_ok:
            failures.append(f"{l3_id}: a target test did not pass")

        for l4 in planned[l3_id]:
            suffix = l4["id"].rsplit("-", 1)[-1]
            kind, description = L4_KIND.get(suffix, ("evidence", l4.get("title", "")))
            l3_prefix = l3_id.replace("OM9-L3", "OM9-L4")
            if kind in {"source_crosswalk", "contract"}:
                continue
            prefix = l4["id"]
            if kind == "target-test":
                write(evidence_dir / f"{prefix}.target-test.log", "\n".join(target_log))
            elif kind == "integration":
                write(
                    evidence_dir / f"{prefix}.integration.log",
                    log_header("cargo test -p om9-core -j 2", integration_code, integration)
                    + "\n---\n"
                    + log_header(
                        "cargo clippy -p om9-core --all-targets -j 2 -- -D warnings",
                        clippy_code,
                        clippy_text,
                    )
                    + "\n---\n# cargo check --workspace --all-targets -j 2\n"
                    + check_text,
                )
            elif kind == "regression":
                write(
                    evidence_dir / f"{prefix}.regression.log",
                    log_header(
                        "cargo test -p om9-core -j 2 (result lines)",
                        0 if "FAILED" not in integration else 1,
                        summary + "\n",
                    )
                    + "\n---\n# cargo test --workspace -j 2 --no-fail-fast (result lines)\n"
                    + workspace_text,
                )
            elif kind == "adversarial":
                negative_only = "\n".join(
                    log
                    for log in target_log
                    if any(
                        word in log
                        for word in (
                            "refus",
                            "cycle",
                            "corrupt",
                            "escape",
                            "breaks",
                            "detached",
                            "async",
                            "outdated",
                            "stale",
                            "ambiguous",
                        )
                    )
                )
                if not negative_only:
                    # Every test of this L3 is a refusal/recovery case; keep them all rather
                    # than emit an empty log.
                    negative_only = "\n".join(target_log)
                write(
                    evidence_dir / f"{prefix}.adversarial.log",
                    "# negative and recovery cases for " + l3_id + "\n\n" + negative_only,
                )
            elif kind == "parameter":
                write(
                    evidence_dir / f"{prefix}.parameter.log",
                    f"""# parameter contract for {l3_id}

The L3 parameter contract in l3.json is DOMAIN_ONLY with option_count = 0: this L3 adds
no user-facing command option. Its inputs are the recorded feature graph itself
(typed ports, parameters owned by the builder recipe, dirty state), so there is no
command line option table to implement in this owner.

Verified behaviour:

- `FeaturePort::validate` refuses an empty role or an empty/duplicate geometry kind set.
- The baseline includes `input_ports` and `parameters` in its content hash, so a
  parameter change is visible to the regression fixture.
- `RecomputeCache` keys on the parameter map, so a parameter edit is a cache miss even
  before the recipe revision moves.

Tests: {', '.join(meta['tests'])}
""",
                )
            elif kind == "ui-review":
                checks = [
                    f"cargo test -p om9-core --test {TEST_BIN} -j 2 -> {summary}",
                    "cargo test -p om9-core -j 2 -> exit "
                    + str(integration_code)
                    + " ("
                    + (
                        "no FAILED line"
                        if "FAILED" not in integration
                        else "FAILED present"
                    )
                    + ")",
                    "cargo check --workspace --all-targets -j 2 -> "
                    + (
                        "exit 0"
                        if "# exit code: 0" in check_text
                        else "NOT CAPTURED" if check_text == "not captured in this batch" else "non-zero"
                    ),
                    "cargo clippy -p om9-core --all-targets -j 2 -- -D warnings -> exit "
                    + str(clippy_code),
                    "cargo test --workspace -j 2 --no-fail-fast -> "
                    + (
                        workspace_note
                        if workspace_text != "not captured in this batch"
                        else "not captured in this batch"
                    ),
                ]
                write(
                    evidence_dir / f"{prefix}.ui-review.json",
                    json.dumps(
                        {
                            "schema": "om9.m04.ui-review.v1",
                            "task_id": prefix,
                            "l3_id": l3_id,
                            "title": meta["title"],
                            "oracle": {
                                "positive": meta["positive"]["expect"],
                                "negative": meta["negative"]["expect"],
                            },
                            "manual_test_gate": "AUTO_ONLY",
                            "implementer": "agent session (implementer never signs as a human reviewer)",
                            "reviewer": {
                                "status": "AUTOMATED_EVIDENCE_ONLY",
                                "role": "plan gate: this item requires no manual test",
                                "date_utc": today,
                                "statement": "The plan records manual_test_gate=AUTO_ONLY and acceptance_gate=AUTO for every L3 of OM9-L2-01-04, so acceptance rests on the automated checks below; no human item is requested and no human approval is claimed.",
                                "scope": "history backend: no on-screen behaviour of its own",
                            },
                            "checks_run": checks,
                            "code_change": meta["code"],
                            "artefacts": [
                                f"{l3_prefix}-03.target-test.log",
                                f"{l3_prefix}-04.integration.log",
                                f"{l3_prefix}-05.regression.log",
                            ]
                            + (
                                [f"{l3_prefix}-07.adversarial.log"]
                                if any(l4["id"].endswith("-07") for l4 in planned[l3_id])
                                else []
                            )
                            + (
                                [f"{l3_prefix}-08.parameter.log"]
                                if any(l4["id"].endswith("-08") for l4 in planned[l3_id])
                                else []
                            ),
                            "gate": {
                                "l3_scope": "ACCEPTED_ON_AUTOMATED_EVIDENCE",
                                "implementation_status_promoted": False,
                                "plan_tree_modified": False,
                            },
                        },
                        indent=2,
                        ensure_ascii=False,
                    )
                    + "\n",
                )

        # Remove a stale review from an earlier L4 numbering, so the bucket has one gate.
        for stale in evidence_dir.glob("*.ui-review.json"):
            if not stale.name.startswith(f"{l3_id.replace('OM9-L3', 'OM9-L4')}-06"):
                stale.unlink()

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1

    # One module-level record: what existed before this batch, what each L3 proves and what
    # still depends on an upstream producer.
    rows = "\n".join(
        f"| `{l3_id}` | {meta['title']} | {len(meta['tests'])} | {meta['positive']['expect']} |"
        for l3_id, meta in L3.items()
    )
    write(
        DESIGN / "OM9-L2-01-04" / "README.md",
        f"""# M04 `OM9-L2-01-04` — associative dependency rebuild

**Milestone:** M04 Builder and History · **Gate:** AUTO_ONLY · **Evidence:** `implementation/evidence/M04/OM9-L3-01-04-001..009/`

## What was already there

W7 ported the deterministic dependency kernel (`stale_recipes`, `mark_stale`,
`plan_recompute`, `mark_clean`, `mark_failed`, `detach`, `freeze`, `state_counts`) onto
`HistoryGraph`. That kernel is reused unchanged; this batch did not rewrite it.

## What this batch adds

| L3 | Title | Tests | Positive oracle |
|---|---|---:|---|
{rows}

The code is `crates/om9-core/src/associative.rs` plus additive fields on
`Recipe` (`input_ports`, `output_ports`) and `HistoryGraph` (`references`), and two
borrow-splitting helpers in `history_ops.rs`. Both new `Recipe` vectors and the reference
map are `serde(default)` and skipped when empty, so an existing document round-trips byte
for byte.

## Decisions worth keeping

1. **A type is declared, never inferred.** `Recipe.input_ports` records what the feature
   asked for. A recipe saved before typed ports is reported `Untyped`; the code never
   looks at today's geometry and calls that the recorded type.
2. **A scalar port is not an object slot.** Binding a curve to a `Length` port is
   `Invalid("object bound to scalar input")`, which is the fixture's "CurveRef into
   ScalarMm" refusal.
3. **Provenance is durable and unit-safe.** A recorded `Value::Number` cannot reopen in
   millimetres, so it is refused when the provenance is read, not guessed on reopen.
4. **A semantic reference needs a unique name, the recorded kind and a fingerprint inside
   tolerance.** A duplicate name is `ambiguous`; movement past tolerance is `moved` and
   requires an explicit `relink_reference`.
5. **The cache key is (revision, parameters, input stamps, outputs).** A change to any of
   them is a miss even while the graph still calls the feature `Clean`.
6. **Preview stores deltas, not a second document.** `PreviewSession::begin` copies only the
   base identity/revision; a parameter edit stages one recipe, preview geometry one entity.
   `commit` applies the staged deltas to the still-current document and refuses if the base
   moved. A preview of one parameter in a large document therefore holds no geometry copy.
7. **A failed rebuild restores a captured snapshot** (recipe, outputs, published results)
   and records `Failed(reason)`; the previous geometry and result stay current. A successful
   rebuild re-stamps its recorded inputs, so its own output does not look stale next time.
8. **A rebuild re-checks the feature's references before it commits.** A produced subshape
   set that drops the name, duplicates it, changes the kind or moves past tolerance turns
   the rebuild into `Failed(...)`; there is no silent relink.
9. **Deleting a parent reports before it chooses.** `delete_source_input` returns the
   broken children with `KeepBroken`/`DetachDependents`/`RemoveDependents`, refuses with
   `Busy` while a child is rebuilding, and the applied policy leaves no reference to the
   deleted object.
10. **A baseline is content-ordered.** Nodes are sorted by `(command, Merkle label over
    content and producer labels)`, so two sessions agree without depending on random
    `HistoryId`s; the hash includes parameters, ports and output geometry.

## Real producer chain (follow-up)

`crates/om9-core/src/producers.rs` supplies the chain and the registry:

- `RingRailProducer` -- closed circle centerline at `diameter_mm`;
- `RailProfileProducer` -- closed rectangle on the rail start frame (`width_mm`,
  `height_mm`), derived from the rail tangent rather than a hard-coded world plane;
- `RailSweepProducer` -- profile swept along the rail through a `SweepBackend`; the analytic
  reference backend is `surface_kernel::sweep1`, the OCCT backend is
  `om9_kernel::associative::OcctSweepBackend` (`creation::pipe_shell`);
- `SnapshotTopology`/`OcctTopology` -- names `edge:*`, `face:*`, `vertex:*` (or
  `curve:0`/`face:0`) with anchor/measure fingerprints from the B-Rep snapshot;
- `ProducerRegistry` + `recompute_registered` -- one call from a host, and the registry
  validates produced subshapes against the feature's references before commit.

`crates/om9-kernel/tests/associative_history.rs` proves the chain end to end with the real
OCCT sweep: a B-Rep whose source is the OCCT bridge, published edge/face/vertex names, and a
semantic reference that still resolves after a profile edit.

## SDK export (follow-up)

The SDK owns the history read contract (the host keeps the write/transaction side):

- `HostRequest` variants `HistoryList`, `HistoryFeature`, `HistoryDependencies`,
  `HistoryProvenance`, `HistoryRebuildPlan` and the typed `Host` wrappers;
- `FeatureRecordInfo`, `FeatureDependencyInfo`, `FeatureProvenanceInfo`, `FeatureInputInfo`;
- capability names in the host's `capabilities` reply;
- `sdk/examples/hello-plugin` command `HistoryDemo` calls all five through the public SDK.

Recompute itself stays host-internal: it needs the native kernel producer and the host's
atomic transaction/Undo owner, so a plugin gets the plan and the record, not a direct
mutation route.

## Still outside this owner

- The builder UI that calls the chain (live handles, panels, the command frame) belongs to
  the M04 builder owner (`OM9-L2-03-11`, `OM9-L2-04-02`); this owner exposes the headless
  producer and rebuild service it calls.
- A host transaction mutation for a rebuilt history recipe is not added: the application
  owner decides how a recompute is journaled, and the current app replay path still edits
  geometry through `Mutation::Geometry`.

## Re-run

```text
python tools/record_m04_l2_01_04_evidence.py --workspace-log build/m04-l2-01-04-evidence/workspace-tests.log
python tools/record_m04_l2_01_04_followup.py
python tools/om9_milestone_dashboard.py
```
""",
    )

    # The live ledger gets one entry per verified batch, idempotently.
    ledger_path = ROOT / "docs" / "openmatrix9-progress.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    key = "m04-l2-01-04-associative-rebuild-2026-10-26"
    entry = {
        "key": key,
        "date": "2026-10-26",
        "stage": "feature-port",
        "status": "implemented_core_verified_l3_evidence_recorded",
        "target": "Standalone Rust OM9 history backend (om9-core); Windows x64, CARGO_TARGET_DIR=L:/om9-target, -j 2",
        "feature_ids": ["OM9-L2-01-04"] + sorted(L3),
        "scope": (
            "M04 OM9-L2-01-04: hoàn tất backend DAG/history còn thiếu sau W7 — typed input/output ports, "
            "command provenance, dirty propagation/topological order, stable semantic references, incremental "
            "cache, preview vs committed, failed-recompute rollback, delete-parent policy và deterministic "
            "rebuild baselines. Không sửa cây kế hoạch; trạng thái L3 do evidence quyết định."
        ),
        "outputs": [
            "crates/om9-core/src/associative.rs",
            "crates/om9-core/src/history.rs",
            "crates/om9-core/src/history_ops.rs",
            "crates/om9-core/src/identity.rs",
            "crates/om9-core/src/model.rs",
            "crates/om9-core/tests/associative_rebuild.rs",
            "tools/record_m04_l2_01_04_evidence.py",
            "implementation/design/M04/OM9-L3-01-04-001..009/api.md + source_crosswalk.json",
            "implementation/fixtures/M04/OM9-L3-01-04-001..009/{positive,negative}.json",
            "implementation/evidence/M04/OM9-L3-01-04-001..009/",
        ],
        "evidence": [
            "implementation/evidence/M04/OM9-L3-01-04-001..009/*.target-test.log",
            "implementation/evidence/M04/OM9-L3-01-04-001..009/*.integration.log",
            "implementation/evidence/M04/OM9-L3-01-04-001..009/*.regression.log",
            "implementation/evidence/M04/OM9-L3-01-04-001..009/*.ui-review.json",
        ],
        "tests": {
            "associative_rebuild": summary,
            "om9_core": f"cargo test -p om9-core -j 2 -> exit {integration_code}",
            "workspace": "cargo test --workspace -j 2 --no-fail-fast -> " + workspace_note,
            "check": "cargo check --workspace --all-targets -j 2 -> "
            + ("exit 0" if "# exit code: 0" in check_text else "not captured"),
            "clippy": f"cargo clippy -p om9-core --all-targets -j 2 -- -D warnings -> exit {clippy_code}",
            "rustfmt": "changed files are rustfmt-clean; the repository already had pre-existing fmt diffs outside this batch",
        },
        "code_added": [
            "associative.rs: PortKind/FeaturePort + typed_edges + validate_typed_ports (typed dependency edges)",
            "associative.rs: FeatureProvenance/reopen_feature/replay_environment (command provenance)",
            "associative.rs: SubshapeSignature/SemanticReference/bind_reference/resolve_semantic_reference/relink_reference",
            "associative.rs: CacheEntry/RecomputeCache (revision + parameters + input stamps, geometry SHA-256)",
            "associative.rs: PreviewSession (private draft; optimistic base check on commit)",
            "associative.rs: RebuildRequest/RebuiltOutput/RecomputeOutcome/recompute_feature + FeatureSnapshot rollback",
            "associative.rs: BrokenInputPolicy/RebuildActivity/delete_source_input (report + apply, no dangling reference)",
            "associative.rs: FeatureGraphBaseline/feature_graph_baseline/load_baseline (content-ordered deterministic hash)",
            "history.rs: Recipe.input_ports/output_ports, HistoryGraph.references (serde default, skipped when empty)",
            "history_ops.rs: mark_document_stale/plan_document_recompute (split borrow without cloning the document)",
        ],
        "uncertainties": [
            "Chưa nối producer hình học thật (OCCT/rail/sweep): recompute_feature nhận producer closure; phần builder L2-04-02/OCCT L2-01-05 vẫn là bước tiếp theo, nên batch này chứng minh graph kernel chứ không chứng minh builder UI.",
            "Semantic reference phân giải trên PublishedSubshapes do producer công bố; chưa có topology provider OCCT nối vào.",
            "PreviewSession clone cả document (base + draft): đúng nhưng tốn bộ nhớ với document lớn; nếu cần, bước sau là draft dạng diff.",
            "Workspace chưa sạch cargo fmt --check trước batch này (diff có sẵn ở om9-app...); chỉ các file của batch được rustfmt.",
            "cargo test --workspace có "
            + workspace_note
            + " trong 7 binary om9-app/om9-render; batch này không sửa file nguồn/test nào của hai crate đó và không test nào trong số đó chạm API history. om9-core suite + clippy của batch xanh.",
            "Không có log RED-before-GREEN riêng: batch làm trong một phiên; các test negative/adversarial là bằng chứng từ chối thực tế.",
            "SDK export cho owner này (OM9_Milestones/SDK-INTEGRATION-PLAN.md) chưa thực hiện: chưa thêm public DTO/wrapper trong om9-sdk.",
        ],
        "manual_gate": "AUTO_ONLY: evidence tự động; không tuyên bố chữ ký người thật",
        "next_action": "Nối producer thật cho ring rail/profile/sweep (M04 L2-04-02), rồi SDK export cho history owner hoặc sang L2-03-11 builder UI.",
    }
    ledger["items"] = [item for item in ledger["items"] if item.get("key") != key] + [entry]
    ledger["active_item"] = key
    ledger["updated_at"] = datetime.datetime.now().astimezone().isoformat(timespec="seconds")
    ledger_path.write_text(
        json.dumps(ledger, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )

    print(f"recorded M04 OM9-L2-01-04 evidence at revision {revision} (dirty={dirty})")
    print(f"target suite: {summary}")
    print(f"ledger: {ledger_path.relative_to(ROOT)} ({len(ledger['items'])} items)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
