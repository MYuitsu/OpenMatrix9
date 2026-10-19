# OM9-L3-01-08-005 — Builder Parameter Range Validation (typed API)

Owner module `OM9-L2-01-08` (units, tolerances, basic parameters; stage `E01`, gate **AUTO_ONLY**).
Module contract: `implementation/design/M01/OM9-L2-01-08/api.md` §3.
Implementation owner: `crates/om9-core/src/request.rs`. Legacy decision: **PORT**.

Oracle — positive: *Domain Number chỉ nhận Quantity cùng chiều, trong [min,max], đúng bước.*
Negative: *Số trần bị MissingUnit; sai bước là Invalid("numeric option step"); schema sai bị từ chối trước mọi giá trị.*

## 1. Typed API

```rust
OptionDomain::Number { dimension, min, min_inclusive, max, step: Option<f64> }
impl OptionDomain { pub fn validate(&self) -> Result<()>; pub fn check(&self, &Value, &Limits) -> Result<()>; }
request::admit(...) calls domain.validate() for every schema option before admission
```

## 2. Errors (exact strings)

- `Invalid("numeric option bounds") / Invalid("numeric option step")`
- `Invalid("missing unit") — a bare number never satisfies a Number domain`
- `Invalid("numeric option domain") — outside range / non-integral count`

## 3. Legacy `rust/` audit

- `rust/src/command_option_schemas.rs`
- `rust/src/command_options.rs`

Legacy schemas carry range/step through the C option layer; the rule is ported into the Rust request schema and is now enforced before any value is admitted.

## 4. Tests that carry the oracle

- `cargo test -p om9-core --test units_parameters l3_005`

## 5. Evidence

- `implementation/evidence/M01/OM9-L3-01-08-005/OM9-L4-01-08-005-03.target-test.log`
- `implementation/evidence/M01/OM9-L3-01-08-005/OM9-L4-01-08-005-05.regression.log`
- `implementation/evidence/M01/OM9-L3-01-08-005/OM9-L4-01-08-005-05.ui-review.json`
- `implementation/fixtures/M01/OM9-L3-01-08-005/positive.json`, `negative.json`
