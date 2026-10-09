# Extended numeric units review — 2026-10-09

Reviewed the uncommitted `rust/src/units.rs` increment against `349203d`,
`rust/tests/numeric_units.rs`, existing `rust/tests/units.rs`, and
`docs/superpowers/plans/2026-10-09-rhino-core-extended-input.md`.
The initial reviewed units diff had working blob prefix `162760d`; the historical
finding below describes that pre-repair snapshot. The subsequent `angle_scalar`
repair was independently reviewed against working blob prefix `8a296f4`.
This review does not establish completion of P0.

## Disposition

The P2 finding is resolved by the reviewed repair. No outstanding concrete
finding remains within the selected units scope.

## Historical finding — resolved

- **[P2] `rust/src/units.rs:372`: Exact out-of-range angle tokens can round to the inclusive limit before the bound check. Validate the bounded token/components before reducing them to `f64`, and add positive, negative, `d`-suffixed, decimal, and scientific boundary regressions.**

`scalar` combines a mixed fraction at line 213, while decimal/scientific input
is parsed directly into `f64`. `parse_angle` subsequently checks only that
rounded value. For example, the following exact quantities exceed the documented
OM9 absolute-total-degree limit of `1e9`, but all return the same allowed angle:

```text
parse_angle("1000000000-1/9007199254740991")
  -> Ok(17453292.519943297)
parse_angle("-1000000000-1/9007199254740991")
  -> Ok(-17453292.519943297)
parse_angle("1000000000-1/9007199254740991d")
  -> Ok(17453292.519943297)
parse_angle("1000000000.00000001")
  -> Ok(17453292.519943297)
parse_angle("100000000000000001e-8")
  -> Ok(17453292.519943297)
```

Expected: `Err(UnitError::Overflow)` for every case above. This is the same
rounding concern already handled explicitly for DMS at lines 356–362:

```text
parse_angle("1000000000d0'0.000001\"") -> Err(Overflow)
parse_angle("1000000000-0/1") -> Ok(17453292.519943297)
```

The issue permits tokens outside the selected strict magnitude policy and makes
the bound depend on notation. Keep the intentional legacy decimal length path
and its accepted parsing-underflow behavior unchanged when repairing it.

## Repair review

`angle_scalar` now checks the exact validated token when its parsed `f64`
magnitude equals the inclusive degree bound. This is sufficient: the bound is
exactly representable, so an exact quantity above it can round to the bound or
above it, not to a lower representable value. The existing final check rejects
values rounded above the bound. The helper is used for bare and `d`-suffixed
angles only, preserving the legacy length parser's behavior.

For fractions, prior `scalar` validation limits every unsigned component to
`2^53-1` and rejects zero denominators. Re-parsing into `u128` and comparing
`whole * denominator + numerator` with `1_000_000_000 * denominator` is exact.
The left side is below `2^106`; the right side is below `2^83`, so neither
unchecked product/addition can overflow `u128`.

For decimals, normalized significand length minus decimal places plus exponent
determines the base-10 order. At order 9, the exact bound requires leading digit
`1` and all remaining digits `0`; any larger significand exceeds it. The prior
successful finite, nonzero `f64` parse at magnitude `1e9` guarantees a nonempty
ASCII digit string, making the first-byte and byte-1 suffix accesses safe.
The 128-byte token bound limits mantissa displacement, so a token parsing to
`1e9` cannot have an effective exponent near an `i32` endpoint; the order
calculation stays small. Leading exponent zeros, positive/negative signs,
omitted integer digits, and trailing decimal points preserve the comparison.

Independent follow-up verification:

- Fresh `rtk cargo test --test numeric_units --test units`: 20 tests passed,
  zero failures, including the new exact-bound regression.
- A temporary Rust probe imported the repaired source and compared 5,175
  decimal/scientific and mixed-fraction boundary cases against exact integer
  or rational expectations. All passed, including positive/negative signs,
  `d` suffixes, significands near the bound with 0–100 decimal places, maximal
  allowed fraction denominators, and normalization variants. The temporary
  executable was removed.

## Verification and limits

- `rtk cargo test --test numeric_units --test units` completed successfully:
  19 tests passed across two suites, zero failures.
- A temporary standalone Rust probe imported the actual reviewed `units.rs`
  and reproduced every result above; the temporary executable was removed.
- Reviewed mixed signs, zero denominators, bounded integer components,
  malformed/duplicate delimiters, whole-token consumption, whitespace policy,
  remainder ranges, final finite/zero-conversion checks, degrees-to-radians
  conversion, and the unchanged dimension-scaling and legacy length paths.
- No additional concrete bug was identified in this scope. The exclusive
  inch/second/minute upper bounds reject values rounded to those bounds;
  compound feet and fractional integer limits are checked before conversion.
- No source files were edited and no Git state was mutated. No UI/native
  runtime, native build, full Rust suite, exhaustive fuzzing, or complete Rhino
  parity verification was performed by this reviewer.

## Final source rereview — degree/radian split

Final reviewed `rust/src/units.rs` SHA256:

```text
60b488fff3969cf54a103a6988684a2b810c0dbac165187e9874cd852f5f2b89
```

The final source factors the existing bounded angle grammar, error mapping,
exact magnitude-bound checks, and DMS component checks into
`pub(crate) angle_degrees` at line 361. `parse_angle` at lines 417–418 still
applies the identical checked `PI / 180` conversion to that result, preserving
finite/underflow rejection at conversion, signed zero, radians output, and
independence from length scales. The fractional/decimal exact-bound helper and
the legacy length path retain the previously reviewed behavior.

Disposition: the historical P2 remains resolved; no additional concrete units
finding arose from this extraction or formatting. This final pass was a source
and fingerprint rereview only, with no build, UI, or test rerun. The independent
20-test/5,175-case evidence above belongs to the prior repaired source; the
integrator reports the focused 20 tests remain green after this extraction.
Coordinate consumer singularity behavior is outside this units-only rereview.
