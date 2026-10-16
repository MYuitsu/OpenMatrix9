"""Authored Rust sketch templates used by tools/apply_implementation_samples.py.

Ground rules for every template here:

- It is a *skeleton*, not verified code. The checker flag and the runbook section
  say so; nothing in a sketch may be read as evidence that a feature works.
- It only ever names things that already appear in the task record (the proposed
  API signature, the implementation algorithm text, the source anchors) or Rust
  standard-library items. No invented project symbol, no invented type.
- Reference code from the pinned QymCAD checkout and the recovered Matrix9 sources
  is described, never copied: the sketch says where to look and what to reuse.

The templates are functions of a small context dict:
    purpose, api, algorithm, algorithm_source, owner, positive, negative,
    oracle, negative_oracle, fixture, sources, anchor_label
"""

import re

HEADER_NOTE = ("Author-written skeleton derived from this task's own record (proposed API / "
               "implementation_algorithm / source anchors). It is not compiled, not tested and "
               "proves nothing about upstream behaviour.")

PROVENANCE = {
    "derived_from": "task record fields, authored reference sheets and pinned upstream source paths",
    "upstream": ("QymIs-Tech/QymCAD at 746a17f127897e921a57118b6caad08968602bf5 - read the file at the "
                 "pinned revision before reusing anything"),
    "matrix9_reference": ("recovered Matrix9 surfaces referenced in the task are behaviour references "
                          "only; no recovered source body is copied"),
    "not_evidence": ("a sketch is a starting scaffold. PASS still requires a command receipt with "
                     "exit 0 plus positive and negative oracle measured from the real run"),
}


def _api_signature(ctx):
    """Return (signature, is_real) for a Rust doc comment or a plain comment."""
    api = (ctx.get("api") or "").strip()
    if api:
        return api, True
    return ("TODO(API): no verified signature in this record yet; freeze the type before coding", False)


def _algorithm_lines(ctx):
    """Algorithm content for the sketch: the record's own text plus cited domain steps.

    The domain steps come from the authored algorithm notes for this L3 (see
    DOMAIN_NOTE_MAP in the applier). They are added as their own numbered block so a
    reader can tell the planning record's text apart from the extracted behaviour.
    """
    text = (ctx.get("algorithm") or "").strip().rstrip(".")
    if not text:
        text = (ctx.get("purpose") or "implement the capability").strip()
    # keep the source text as a comment block so the sketch cannot be mistaken for design
    parts = [p.strip() for p in text.replace("; ", "\n").split("\n") if p.strip()]
    domain = [str(s).strip() for s in (ctx.get("domain_steps") or []) if str(s).strip()]
    if domain:
        parts.append("--- bước thuật toán domain đã trích (phải đo lại hằng số trước khi dùng) ---")
        for i, step in enumerate(domain):
            # notes label their own steps ("B1.", "G2."); drop that label so the sketch
            # does not show "D1. B1." as a doubled prefix
            cleaned = re.sub(r"^[A-Z]{1,2}\d+\.\s*", "", step)
            parts.append(f"D{i + 1}. {cleaned}")
    return parts


def _contract_lines(ctx, todo_prefix="//"):
    """Render the contract line so it stays valid Rust whichever form it takes."""
    api, has_api = _api_signature(ctx)
    if has_api:
        return f"/// Contract on record: {api}"
    return (f"{todo_prefix} TODO(API): no verified signature in this record yet; "
            f"freeze the type before coding\n/// Contract on record: (not frozen yet)")


def source_map(ctx):
    """SOURCE_AUDIT: an inventory skeleton, no implementation."""
    api, has_api = _api_signature(ctx)
    steps = "\n".join(f"    // - {line}" for line in _algorithm_lines(ctx))
    sources = "\n".join(f"// source: {s}" for s in ctx.get("sources", []))
    return f"""// {ctx['title']}
// ROLE: SOURCE_AUDIT - this task produces an inventory, not code.
{sources}
//
// Capability: {ctx['purpose']}
// Proposed contract on record: {api if has_api else '(none frozen yet)'}
//
// Inventory to fill from the pinned checkout (record file, symbol, line range, verdict):
{steps}
//
// Decision skeleton to complete with real evidence:
//   reuse_decision: REUSE | ADAPT | ADD | BLOCKED_SOURCE
//   evidence: <path>:<line-range> at pinned SHA
//   license: <reviewed scope from M00>
//
// Do not claim "implemented", "works" or a PASS from this file."""


def implement_sketch(ctx):
    """IMPLEMENT: module skeleton around the frozen contract."""
    api, has_api = _api_signature(ctx)
    steps = "\n".join(f"//   {i}. {line}" for i, line in enumerate(_algorithm_lines(ctx), start=1))
    contract = _contract_lines(ctx)
    return f"""// {ctx['title']}
// ROLE: IMPLEMENT - skeleton only; body is deliberately unimplemented.

{contract}
///
/// Owner: {ctx['owner']}
/// Positive fixture: {ctx['positive']}
/// Negative fixture: {ctx['negative']}
///
/// Algorithm notes carried from the planning record:
{steps if steps else '//   (no algorithm text on record)'}
pub fn {ctx['fn_name']}(/* typed inputs from the contract */) -> Result</* Output */, {ctx['error_type']}> {{
    // TODO({ctx['task_id']}): implement exactly one owner's behaviour here.
    //   1. validate typed inputs; reject empty/degenerate input with a typed error.
    //   2. do the work through the owning kernel/domain service, never a second engine.
    //   3. return a typed result; never report success on a partial write.
    unimplemented!("{ctx['task_id']} not implemented")
}}

#[cfg(test)]
mod tests {{
    // Positive: {ctx['positive']}
    #[test]
    fn positive_{ctx['fn_name']}() {{
        // RED until implemented: assert the measured oracle, not a smoke check.
        // Oracle: {ctx['oracle']}
        unimplemented!()
    }}

    // Negative: {ctx['negative']}
    #[test]
    fn negative_{ctx['fn_name']}() {{
        // Oracle: {ctx['negative_oracle']}
        // Assert a typed error and unchanged state (no partial document mutation).
        unimplemented!()
    }}
}}"""


def contract_test_sketch(ctx):
    """CONTRACT_TEST: freeze the signature and write the intentionally failing test."""
    api, has_api = _api_signature(ctx)
    contract = _contract_lines(ctx)
    return f"""// {ctx['title']}
// ROLE: CONTRACT_TEST - freeze the contract, then write a test that fails first.
{contract}
// Owner: {ctx['owner']}
//
// 1. Freeze the typed signature and the error enum in the owner crate.
// 2. Write the fixture harness for {ctx['fixture']}:
//      input:  {ctx['positive']}
//      expect: {ctx['oracle']}
//      negative input: {ctx['negative']}
//      negative expect: {ctx['negative_oracle']}
// 3. Run it: it must fail for the right reason (missing behaviour), not a compile accident.
// 4. Only then hand the frozen type to the IMPLEMENT task of the same L3.

#[test]
fn {ctx['fn_name']}_contract_is_red_first() {{
    // Arrange: build the fixture input from authored data, not from the implementation.
    // Act: call the frozen entry point.
    // Assert: expected == actual for the positive fixture, and a typed error for the negative one.
    // Record the failing output as the RED receipt for this task.
    unimplemented!()
}}"""


def integrate_sketch(ctx):
    """INTEGRATE: composition order across owners, with rollback."""
    return f"""// {ctx['title']}
// ROLE: INTEGRATE - wire owners together; do not reimplement their logic.
//
// Purpose: {ctx['purpose']}
// Boundary: this task may only orchestrate; each step below must already be owned
//           and tested by the module named in the record's external_owner_interfaces.
//
// Order: {ctx['algorithm'] or ctx['purpose']}
//
// Integration checklist:
//   1. resolve owners through the parent service contract, not through UI handles
//   2. keep one transaction boundary: either the whole sequence commits or nothing does
//   3. on any step error, roll back what this sequence already changed and report the failing step
//   4. assert identity survives: same stable IDs before and after, no stale scene reference
//
// Contract under test: {ctx['api'] or '(see owner contracts)'}
// Oracle: {ctx['oracle']}
// Negative oracle: {ctx['negative_oracle']}"""


def acceptance_sketch(ctx):
    """ACCEPTANCE: assertion matrix over the emitted oracle."""
    return f"""// {ctx['title']}
// ROLE: ACCEPTANCE - assert the emitted evidence; this task adds no behaviour.
//
// Fixture / oracle on record: {ctx['fixture']}
// Positive case: {ctx['positive']}
//   expect: {ctx['oracle']}
// Negative case: {ctx['negative']}
//   expect: {ctx['negative_oracle']}
//
// Assertion matrix to fill from the real run (no placeholder may remain):
//   id | case      | input          | expected            | actual | verdict
//   ---+-----------+----------------+---------------------+--------+--------
//    1 | positive  | <fixture input>| <oracle>            | <meas> | PASS/FAIL
//    2 | negative  | <bad input>    | <typed error>       | <meas> | PASS/FAIL
//    3 | unchanged | after failure  | document identical  | <diff> | PASS/FAIL
//
// Rules:
//   - every 'actual' must come from a command receipt with exit 0 and a captured log path
//   - a missing log, an empty log or an unexecuted placeholder is FAIL, never PASS
//   - geometry claims need a measured quantity plus its tolerance, not a screenshot"""


def regression_stress_sketch(ctx):
    """REGRESSION_STRESS: long-run loop with a stop criterion."""
    return f"""// {ctx['title']}
// ROLE: REGRESSION_STRESS - long-run / adversarial behaviour of an existing capability.
//
// Purpose: {ctx['purpose']}
// Algorithm under stress: {ctx['algorithm'] or ctx['purpose']}
//
// Loop skeleton (choose the parameters from the fixture, then record them):
//   for round in 0..N {{
//       apply the same operation to an increasingly large/pathological input
//       measure: wall time, allocation, handle/leak count, geometry drift vs tolerance
//       stop criterion: <max rounds> or <drift > tolerance> or <first failure>
//   }}
//
// Assertions:
//   - results stay within the declared tolerance for every round
//   - no leaked OCCT/GPU resource: create/destroy counts balance
//   - the first failing round is reported with its input, not summarised away
// Positive oracle: {ctx['oracle']}
// Negative oracle: {ctx['negative_oracle']}"""


ROLE_TEMPLATES = {
    "SOURCE_AUDIT": source_map,
    "CONTRACT_TEST": contract_test_sketch,
    "IMPLEMENT": implement_sketch,
    "INTEGRATE": integrate_sketch,
    "ACCEPTANCE": acceptance_sketch,
    "REGRESSION_STRESS": regression_stress_sketch,
}

# Which reference families the sketches may lean on, stated per role.
ROLE_REFERENCE_NOTE = {
    "SOURCE_AUDIT": "read the pinned file and record symbol + line range; no runtime claim",
    "CONTRACT_TEST": "the fixture is authored data; upstream code is read, never executed as proof",
    "IMPLEMENT": "adapt the pinned algorithm, do not copy it; licence scope comes from M00",
    "INTEGRATE": "compose owners; re-check each owner's contract at its own revision",
    "ACCEPTANCE": "oracles come from this record and the authored sheets, not from the sketch",
    "REGRESSION_STRESS": "parameters are chosen here and recorded with the run",
}
