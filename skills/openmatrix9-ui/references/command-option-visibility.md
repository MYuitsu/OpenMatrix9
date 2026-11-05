# Command option presentation and visibility — accepted 2026-11-02

User decision 2026-11-02: command options are shown only inline in the Command prompt,
the current mode/value is the prompt word, the remaining options are stage-scoped
alternatives, and the Matrix manual references in the L3 folders are the behaviour source
(including Matrix overrides of the Rhino-style prompts). This reference records the rule and
the workflow so the next command migration does not re-invent or over-expose options.

## Presentation rules

- One presentation: prompt text then options in parentheses
  (`Center of sphere ( 2Point 3Point ... )`). No floating option-button row; a click, a
  typed name and the underlined accelerator all resolve against the same owner snapshot.
- The current mode, measure or parameter is the prompt word
  (`Center of sphere`, `Diameter`, `Next point of polyline`). It is not repeated inside
  the option list. This is the Rhino 5 p22 example and the approved Circle/Box prompts.
- Construction mode words stay plain (`Vertical`, `2Point`, `Corner`, `FromFoci`).
  Value-bearing flags and parameters keep the approved `Name=Value` form
  (`BothSides=No`, `Solid=Yes`, `Turns=3`, `Size=Area`) from the Polyline standard
  (SPEC §8). Do not mix the two styles inside one family.
- An option appears only in the stage that accepts it:
  - a later-stage option is hidden until its action happens
    (`Length` after the first Line point, `Length` after the first Rectangle corner,
    `Cube` after a Diagonal corner, `Radius`/`Diameter` after choosing Center Arc,
    `FromLastPoint`/`UseLastDistance`/`UseLastDirection` after the first Copy);
  - an option the parser rejects in this stage is not offered
    (Rectangle `Length=` before the first corner, `Radius=` when the Arc mode cannot use it);
  - mode choices are offered only before the first input that locks the flow.
- Cancel is Escape. The prompt advertises `Cancel` only where the owner contract already
  did; do not add it to a documented prompt that does not show it (Box, Circle).
- Hiding a token never removes the typed path. Assignments accepted by the parser keep
  working even when the option is hidden in that stage; hidden means "not advertised",
  not "rejected".

## Matrix authority and where it lives

- The Matrix book 1/book 2/addendum extracts are stored per L3 in
  `OM9_Milestones/**/L3-*/reference/text/*.md`; read those, not the PDFs. A command
  section gives `Steps` and `Command Line Options` — both matter: Steps define the
  stage, options define the names.
- Derived locator (machine-generated, needs a re-read before editing):
  `docs/validation/matrix-ref-command-options-2026-11-02.md` (command → option words →
  owning L3 reference path/page) and
  `docs/validation/matrix-ref-option-diff-2026-11-02.md` (option words vs the running
  command bar). The extractor can pull sub-options out of prose; treat a row as a pointer,
  not as a verified contract.
- Family-level notes with page citations: `docs/validation/2026-11-02-matrix-command-prompts.md`.
- The Matrix entries override Rhino-style prompts where they differ: e.g. Circle adds
  Radius/Diameter, Orientation, Circumference and Area after the center; Arc is
  center-based with StartPoint/Tangent/Extension; Rectangle has Rounded and a Shift square;
  Copy's From/Use-Last options only appear after the first copy.

## Workflow for one command or family

1. Route to the exact L3 and read its reference text section; collect `Steps` and
   `Command Line Options` (and the sub-options such as `3Point options`, `Tangent options`).
2. Write the stage table before editing: stage, prompt, required input, available
   options, current/default values, allowed transitions, finish/cancel/undo.
3. Compare the table with the provider token list, `describe_command_option` labels,
   prompt text and the owner parser. Where the Matrix text and the parser disagree,
   record both; do not silently pick one.
4. Implement: gate the tokens by stage, make the current mode/measure the prompt word,
   keep typed aliases, and leave one inline render path. Update the option values/hints
   only where the owner has a real value.
5. Tests: a stage fixture per prompt (text + clickable tokens), the keyboard/pointer/typed
   activation routes, invalid/retry, geometry or document effect, Undo. Update any M01
   fixtures that pin the old prompt.
6. Evidence: `docs/validation/...` record with the reference path/page and tests, an
   `implementation/design/.../EXECUTION.md` checkpoint, and the live ledger entry.

## Conflict policy

- Matrix manual/reference is the behaviour source for prompt words and stages, but a
  user-approved local decision (M01 fixtures, OWNER-CARDS, milestone contracts) is not
  overridden silently. When they conflict, record both and ask the user which contract
  wins before changing behaviour.
- Decision 2026-11-02: for the two known conflicts the user chose the Matrix direction.
  Line no longer offers `Length` (a bare number at the end prompt is the length; the typed
  `Length=` assignment stays an alias) and Rectangle no longer offers `Square` (Shift while
  picking the corner makes the square). The M01 `OM9-L3-01-02-005` fixture now declares
  `BothSides` only, and the owner card's `Square` entry is superseded by this decision.
- Do not mark a command complete from a screenshot or an icon click. Automated tests do
  not replace the manual M01-UI gate.

## Applied facts (2026-11-02)

- Polyline: the reference implementation (PersistentClose/Mode/Helpers; Close only after
  three points; Length only in Line mode; Direction/Center only in Arc mode).
- Box: Diagonal/3Point/Vertical/Center, Cube only after a Diagonal corner.
- Circle: the seven mode words at the center prompt, then the other measures plus
  Orientation; Orientation implemented (normal point, then size).
- Ellipse: current mode is the prompt word; `Approx` is a typed alias, not a token.
- Sphere: current mode is the prompt word; the constraint stages keep Point/Undo/Cancel.
- Arc: modes only before the first point; Radius/Diameter only in Center mode (the Matrix
  center-based Arc, Tangent and Extension remain open).
- Line: BothSides before the first point; after it the end point is picked or a bare number
  is typed as the length. No Length option (Matrix direction, 2026-11-02); `Length=` stays
  a typed alias, and the wider Matrix Line family (Normal/Angled/Vertical/FourPoint/
  Bisector/Perpendicular/Tangent/Extension) is not implemented.
- Rectangle: modes before the first corner; `Length`/`Undo` after it; a square is made by
  holding Shift while picking the corner (`set_rect_square`, not an option). Matrix
  `Rounded` remains open; the typed `Square` word stays a hidden alias.
- Top11 Duplicate: InPlace/Vertical before the first copy; FromLastPoint, UseLastDistance,
  UseLastDirection after it.
