# OM9-L3-01-02-005 — Command Option Parser (typed API)

Owner module `OM9-L2-01-02` (backend framework, stage E01, gate **AUTO_ONLY**).
Implementation owner: `standalone/crates/om9-app/src/option_parser.rs`. Reuse
decision: **ADAPT** the keyed typed parameter table of `qymcad-ui-state`; **ADD**
the textual option language with units and named refusals
(see `source_crosswalk.json`).

Oracle — positive: *Option parser reads length=2.5mm angle=90deg from typed
descriptors.* Negative: *Unknown options, malformed numeric, NaN/Infinity report
parse error.*

## 1. Typed API

```rust
pub enum OptionValue {
    Flag(bool),                                        // BothSides=Yes
    Number { value_mm: f64, radians: Option<f64> },     // Length=2.5mm, Angle=90deg
}
pub struct ParsedOption { pub name: String, pub value: OptionValue }

/// None when the text is not an assignment (a point or a command name).
pub fn parse_option(operation: &Operation, text: &str) -> Result<Option<ParsedOption>>;
```

The declared option names come from the command's descriptor
(`Operation::options`, read without case by `Operation::option`), so the parser
and the descriptor cannot disagree about a name.

## 2. Rules

| Typed | Result |
| --- | --- |
| `BothSides=Yes` / `yes` / `true` / `y` / `on` | `Flag(true)` |
| `BothSides=No` / `false` / `n` / `off` | `Flag(false)` |
| ` BothSides = No ` (padding, any case) | `Flag(false)`, name reported as declared: `BothSides` |
| `Length=2.5mm` / `0.25cm` / `0.0025m` / `0.0984in` | `Number { value_mm: 2.5 }` |
| `Angle=90deg` | `Number { radians: π/2 }` |
| `12,6,0`, `Line` | `None`: not an option at all |
| `Angles=90` (not declared) | `Unsupported("command option")` |
| `Length=abc`, `BothSides=maybe`, `=Yes` | `Invalid("option value")` / `Invalid("option value")` / `Invalid("option name")` |
| `Length=NaN`, `inf`, `1e400` | `Invalid("finite option number")` — read as a number, refused as a non-number |
| `Length=2.5kg` | `Unsupported("option unit")` |
| `Length=` | `Missing("option value")` |
| `Length=10mm` on LINE, then a click 3,4,0 from 0,0,0 | the end point is 6,8,0: the length is fixed, the click chose the direction |
| any refusal | the command keeps running in the phase it was in, and nothing is written |

## 3. Ownership

* The **parser** is pure: descriptor in, typed value out, no document and no
  session.
* The **session** applies the value (`BothSides` sets the flag, `Length` fixes
  the distance); the geometry stays the session's business.
* An unknown option is refused **by name** before any geometry is considered.
