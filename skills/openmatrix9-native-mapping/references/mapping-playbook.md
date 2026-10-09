# Address-driven reading

Paths are relative to the selected OpenMatrix9 checkout. Read original Guide files under `ref/matrix9/Matrix90_Codex_Guide_MD/` for details, only at the applicable stage.

| Need | Document |
|---|---|
| Input identity/build/source fingerprint | `00_PROJECT_FACTS.md` plus actual current input hashes |
| Source precedence | `01_SOURCE_OF_TRUTH.md` |
| Procedure VA and Ghidra identity | `02_ADDRESS_MAPPING.md` |
| VB `.frm/.bas/.cls/.ctl/.123` parser issue | `03_PARSING_VB_OUTPUT.md` |
| Ghidra definition/body boundaries | `04_PARSING_GHIDRA_C.md` |
| Selected procedure workflow | `05_MAPPING_WORKFLOW.md` |
| Mapping and behavioral confidence | `07_CONFIDENCE_AND_VALIDATION.md` |
| Timeout, missing body, thunk | `08_FAILURE_CASES.md`, relevant example in `09_MATRIX90_EXAMPLES.md` |
| Ledger field details | `11_PROGRESS_LEDGER_TEMPLATE.md` |

## Canonical identity

The recovered procedure comment `14DE3A0` is VA `0x014DE3A0`, directly matched to `FUN_014de3a0`. Generic `Proc_3_0_14F56E0` embeds VA `0x014F56E0`. Body `loc_` labels corroborate the header.

Do not subtract ImageBase from these recovered VAs. Only if another tool explicitly supplies an RVA, add the verified binary ImageBase to obtain VA. This dataset's documented ImageBase is `0x00400000`; recheck for different binaries/exports.

Find the named procedure, then candidate Ghidra occurrences:

```powershell
rg -n 'tmrProfileBrowser_Timer' ref/matrix9/vb6-lite/Matrix90/frmMaster.frm
rg -n -m 8 'FUN_014de3a0' ref/matrix9/ghidra/Matrix90.exe.c
```

Inspect the declaration and body at the returned definition, not an arbitrary first hit. Extract a bounded section with line numbers; verify function boundaries including braces inside strings/comments. If using a script, require lexical awareness rather than a naive brace count. Follow a callee only to answer the current unresolved question.

## Per-procedure output contract

Record container/name/signature, original input paths and SHA-256, exact VA and function declaration line, selected evidence ranges, direct calls/strings, unresolved branches and source feature ID. Use two separate confidence fields:

| Mapping status | Mapping confidence | Meaning |
|---|---|---|
| EXACT | A | Exact address with body; behavior still may be uncertain |
| EXACT_DECOMP_FAIL | B | Exact address exists but usable pseudocode is absent |
| NO_EXPORTED_FUN | Unestablished; C only with corroboration | No exported definition; do not infer identity from a string alone |
| THUNK_OR_WRAPPER | A for exact entry; assess body separately | Preserve canonical entry and record delegated address |
| AMBIGUOUS | C or unestablished | Keep candidate evidence and ambiguity explicit |

Behavioral confidence is High/Medium/Low based on supported control flow, types, object members and error handling. Record validation separately. Guide coverage totals are historical baseline measurements, not proof of this export's current coverage.

Example anchors to reverify from the current inputs:

- `frmMaster.tmrProfileBrowser_Timer`: `0x014DE3A0`; exact mapping, COM member meaning may remain unresolved. Compare the VB/native error path with Ghidra, not only the happy path.
- `frmMaster.Form_Load`: `0x014D1DA0`; documented decompile timeout means EXACT_DECOMP_FAIL, not an unmatched procedure.

Preserve existing names. Any inferred rename has its own evidence/confidence entry. Hidden `Me`, calling convention, 16-bit VB Integer versus 32-bit Long, and Boolean True `-1` matter when describing semantics. COM vtable offsets do not themselves identify a named property. Runtime cleanup is not automatically application behavior, but error handling must remain represented or unresolved.

Write analysis outside the original source packages. Only an explicit VB6 recovery request activates reconstruction rules and a recovered VB listing; the normal output is a verified behavior description for Rust.
