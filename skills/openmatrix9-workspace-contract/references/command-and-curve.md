# Command and Curve input

## Shared, copyable transcript

Place Command above the viewports with Matrix-green background and black text. History, live prompt and input belong to one selectable/copyable plain-text transcript. Protect history and prompt from editing; only the draft suffix is editable. Ctrl+A/C can copy the transcript. Pasting and IME input remain on one unwrapped input line.

Default height is **two history rows plus one live input row**, based on font metrics. Drag the lower dock separator to change history height; retain the user's resized height through workbench switches. The live prompt/input remains pinned at the bottom at every supported height, including minimum height and while scrolling old history. Draw a horizontal separator immediately above it. History scrolls independently without discarding or moving the draft.

References: [default rows and options](../assets/matrix-command-options.png), [separator, pinned input and suggestions](../assets/matrix-command-completion.png).

## Idle command-name suggestions

While idle, typing original command names opens suggestions below the input. Use local spec names and supported native mappings; rank exact matches, prefixes, then contained matches. An available exact typed name takes priority over a previously selected contained match. Unavailable commands remain visibly disabled and cannot execute.

- Up/Down selects available suggestions, skipping disabled rows.
- Tab fills the selected name without executing it.
- Enter or a mouse click executes through the shared CMD/menu handler.
- First Esc dismisses the list and preserves the draft; further Esc cancels/clears as usual.

Hide named-command suggestions during point/interactive options or coordinate input, on no match, focus loss and workbench exit. When the popup is hidden, Up/Down recalls command history while preserving the current draft. Empty Enter repeats the last supported command; viewport typing uses the same Command input path.

## Straight Line/Polyline

Polyline accepts successive viewport clicks as points and displays the transient point chain. **Enter finishes and commits the entire wire in one transaction**. Menu/mouse and typed `Polyline` use the same command session and produce the same geometry. Undo within the running command removes the last accepted point and updates its transient preview without adding a document Undo transaction.

Supported options appear inside `( )` with the initial character underlined. Typed initials are case-insensitive; full option names also work. A plain click on a supported option submits through the same input handler only when the draft is empty. Dragging selects transcript text; preserve nonempty drafts.

| Command | Initial / input | Supported behavior |
|---|---|---|
| Polyline | P, P=Y, P=N | PersistentClose setting |
| Polyline | C | Close once at least three points exist |
| Polyline | M, M=L | Existing straight Line mode |
| Polyline | L, L=value | Length after a point |
| Polyline | U | Remove last accepted point |
| Line | B, B=Y, B=N | BothSides before the first point |

Do not infer Arc, Helpers or continuously closed PersistentClose preview support from these names; those remain unfinished. Full core/Curve completion is separate from this implemented slice. Validate actual geometry, finish/cancel, shared option clicks/typed aliases and protected transcript behavior.
