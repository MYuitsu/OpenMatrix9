# Register Command Line Options for a command

RCORE-04 and OM9-BUILDERCORE-001 use one Rust reducer and one Qt binding layer.
OffsetBezier is not the implementation scope or a special case in that engine.
Every supported command supplies its own schema, current values, availability
and native construction adapter. Registering a schema does not enable an
unimplemented geometry command.

## Native adapter

`CommandOptionsBridge::openFeature(featureId, definitions, rules, mmPerUnit)`
copies descriptors into Rust and seals them before accepting input. Use the
command's exact feature ID and current values from its existing solver/settings.
There is no toggle default when no current value has been supplied.

```cpp
using D = CommandOptionDefinition;
bridge.openFeature("existing-feature-id", {
    D::toggle("Reverse", currentReverse).alias("R"),
    D::number("Distance", minimum, maximum, false, currentDistance, "mm").alias("D"),
    D::action("Preview").alias("P")
}, {}, documentMmPerUnit);

bridge.bind(reverseWidget, "Reverse", controller, handleOptionEvent);
bridge.bind(distanceWidget, "Distance", controller, handleOptionEvent);
```

For a choice, set `kind=2`, `choices`, `initialKind=2`, `initialChoice` and optional
aliases. Each choice may name a child. That child must declare the choice's key
in `parent`; cycles and invalid identities reject at registration. `scope` is
1 for CMD, 2 for Builder, or 3 for both. A `CommandOptionGreater` declares a
numeric relation; Rust schemas can also declare a forbidden value pair.
Canonical keys are unique ignoring ASCII case. A choice whose child is
Builder-only is disabled in CMD; typing it rejects before changing navigation.
During a choice phase, its bare choice names take precedence over global keys.

Commands do not add switches to the common reducer or name-based rules.
The built-in schemas in `command_option_schemas.rs` are existing consumers,
while Rebuild demonstrates registration through `openFeature` without adding
a built-in family branch.

## Accepting events

Use the existing console's `CommandOptionSnapshot` spans to render/click options.
The descriptor carries the canonical identity and typed input, including disabled
choices and their reason. Do not parse displayed prompt text to find options.

`event` returns Unhandled (0), navigation (1), candidate (2), action (3), or error
(4). A candidate is provisional. Construct/validate a temporary native result
from `value(key, true)`, then `accept()` and render the accepted state. Reject
with `accept(false)` on native failure. Keep the previous preview and raw input.
CoreRebuild and SurfaceController are real examples; Curve acceptance validates
a cloned solver session and publishes the option and solver state atomically
inside Rust.

`bind` routes checkbox, combo and numeric controls through the same typed events;
`render` uses signal blocking and reads Rust accepted values. Use
`CommandOptionSpinBox` / `CommandOptionDoubleSpinBox` for numeric controls so Qt
does not clamp or replace invalid raw text. Enter parses/accepts the option and
does not activate the dialog's default OK button. Units, finite/range/integer
checks and relations use Rust. `discardDrafts` plus a fresh render handles Esc.

Before a Builder event, synchronize the current CMD draft using `draft`.
A change to the same drafted key rejects and restores the control. An unrelated
change preserves the draft and staged choices. Commit readiness must reject an
open option phase or unresolved input error. Close the bridge when the owning
document/command ends, and retain existing native source and publication guards.
An unrelated CMD assignment also preserves the pending branch. Only completing
the active child accepts its staged ancestors. Opening another named branch
replaces navigation without accepting the abandoned branch. Candidates capture
an input epoch as well as generation/revision; Escape, navigation, new input
and draft replacement retire old admission. The ABI token is single-use.

## Current coverage

The first adapters cover Polyline, Line, InterpCrv, Rectangle, Circle, Ellipse,
Box, Sphere, native Rebuild, and scalar options on Sweep1, Sweep2 and Loft. Surface input
identity controls (directions, seams, slashes) retain their existing native
binding. Circle/Ellipse geometric size input keeps the existing point-construction
semantics. Gem builders and other unimplemented solvers remain gated.

Run `rust/tests/command_options.rs` and `command_options_ffi.rs`, then the native
macros `command_line_options_smoke.FCMacro`, `builder_command_options_smoke.FCMacro`
and `rebuild_command_options_smoke.FCMacro` against the selected standalone plugin
on the shared FreeCAD host. A schema-only test is not native geometry evidence.

## Phase prompts, links and transcript — 2026-10-12

`OptionSession::set_question` / `CommandOptionsBridge::question` supplies the
single-line question prefix. Snapshot `questionLength` and option spans use
UTF-16 offsets; Qt renders the question bold and only its declared single-character shortcut underlined, preserving full-name click spans.
`.hidden()` / definition `shown=false` controls presentation independently of
scope/gate. Hidden legacy keys still validate; visible enabled aliases take
priority over inactive hidden aliases, while multiple active matches reject.

For phase-driven commands, derive the schema from accepted native state and use
`refresh_presentation` to advance revisions without discarding the live draft.
The Box adapter `Session::command_options` and `syncSolid` demonstrate this:
first corner → only current mode links → Diagonal Cube → actual Width/Height
defaults. Family14 is a convenience consumer; other commands still register
their exact feature schema through the same generic API. Do not parse prompt
text to recover state or duplicate the solver in the console.

Consume/remember input only after native admission. Record it with the preceding
question, including click actions; accepted viewport/F4 input records that phase.
Preserve a rejected draft and the preceding geometry. Use the bounded Rust
invocation formatter plus `logInvocation` for each real command start, so
repeated starts/cancellations survive message dedup. A typed command's ordinary
input transcript already records its invocation; avoid recording it twice.

[Box native evidence and limits](validation/2026-10-12-box-matrix-command-options.md).

## Focus, origin input and readable errors

The reusable Qt `focusInput(current)` queues focus after icon/menu release;
provide an active-session generation guard so cancelled/replaced starts cannot
steal focus. Focus belongs in command start, not hover/preview refresh.
PointInput accepts bare0 as CPlane0,0,0; scalar-vs-point phase admission remains
the consumer's responsibility. Preserve raw token and resolved world-point
evidence. Box demonstrates numeric Enter explanations and separate error
transcript rows while retaining invalid drafts and a clean phase question.
[Native correction evidence](validation/2026-10-12-box-command-usability.md).

Snapshot mnemonic metadata is a declared unambiguous one-character alias found
in the key. Register it explicitly; do not infer a working shortcut from an
underline or shorten the click span. item_text field3 owns/copies the mnemonic.
[Single-character native evidence](validation/2026-10-12-command-mnemonics.md).

## Shared command and suboption mnemonics

Every current schema declares single-character shortcuts through the same Rust
engine. There is no Box-specific underline path. Qt decorates the copied
mnemonic inside the exact displayed span; its whole label remains clickable.
Disabled or ambiguous shortcuts have no underline. Exact full names remain
available, and a pending choice resolves its own shortcuts before global keys.

Rust registrations use `OptionSpec::alias` and `Choice::alias`. Dynamic native
registrations use definition `.alias("R")` and `CommandOptionChoice::aliases`.
The existing owned ABI accepts choice aliases via
`om9_options_alias(handle, "Parent=Choice", alias)` before sealing. There are
no new pointer lifetimes or changes to the event/item ABI structures.

Shortcuts are declarations, not automatic initials. For example, Rebuild uses
P/D/I/V for PointCount/Degree/DeleteInput/Preview. Ellipse's Mode choices use
E for Center and C for Corner. Mode shortcuts apply only in that option phase,
so a choice such as 2Point does not capture numeric dimension input later.
The actual Sphere construction session supplies its phase schema just as Box
does; supported 2Point/3Point/4Point shortcuts appear before the first point.
Existing native reference solvers still own geometry and solution selection.
Native diagnostics remain visible in CMD history while phase options refresh.

The registrations cover Line, Polyline, InterpCrv, Rectangle, Circle, Ellipse,
Rebuild, Box, Sphere, Sweep1/History, Sweep2/History and Loft. Commands without
option phases need no artificial options; future supported commands register
their own schema and reuse this renderer, resolver and CMD–Builder binding.
The shortcut mappings added here are OM9 declarations, not a claim that every
private Matrix9 shortcut has been recovered. Unsupported solvers stay gated.

[Cross-command native evidence](validation/2026-10-12-common-command-mnemonics.md).

## Box point input and mode examples

Box opens directly at `First corner of base ( Diagonal 3Point Vertical Center ):`.
Its default is two opposite base corners then Height: `Box → 0 → 10,20,0 → 5`.
The four methods are optional branches. Explicit 3Point (`3`) and Vertical (`V`)
still use two adjacent edge endpoints then Width/Height; Center (`C`) uses a
center then corner or full Length. A bare Length does not replace a point in
the default or explicit 3Point/Vertical branch.

Box previews its base and full wire box in all four views without document
objects. Parallel top/bottom height picks use screen-up drag scaled in world
units, anchored at the opposite base corner for the default. Signed Height,
Undo, cancellation and native output use the same Rust geometry.
[Current video correction](validation/2026-10-12-box-default-video.md) supersedes
mandatory method selection in the earlier documented-workflow report.
