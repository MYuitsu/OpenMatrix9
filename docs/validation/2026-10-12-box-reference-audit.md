# Box — comparison with the user-provided steps

The user supplied the Box description and asked whether the native command
already meets it. The excerpt is reference material; it is not permission to
invent original defaults or equate passing geometry tests with complete parity.

The current tested plugin is SHA-256
`cbe982b7f5129811f85544a2b4736cc5361bb56f53e9b22c30695be46191733d`.
A fresh native audit passes31 checks. Exact input, accepted prompts and native
parameters are in
`C:/Users/nguye/AppData/Local/Temp/om9-all-command-mnemonics-20261012/box-reference-audit03/results.json`.
The existing interactive user document/window was preserved.

| Referenced branch | Verified current behavior |
|---|---|
| Ordinary Box | First corner, other corner or Length, Width and Height; existing Enter defaults were separately verified by the usability fixture. |
| Diagonal | Two opposite base corners, then Height; rejects a scalar side length and offers Cube at the second corner. |
| 3Point | Start/end coordinates define the first edge, followed by numeric Width and Height. The native implementation also accepts an implicit-axis numeric Length, beyond the explicit endpoint steps in the supplied excerpt. |
| Vertical | Start/end coordinates, Width and Height; base axes are X and CPlane Z, with its normal perpendicular to CPlane Z in the audited frame. |
| Center | Center first, then other corner or full Length; a10×20×10 box has origin−5,−10,0 around the supplied center0,0,0. |
| Cube | Two opposite3D corners immediately construct equal edges and determine orientation; there is no separate third Height prompt. A separate Height step, if intended by the supplied prose, is not implemented by this current sequence. |

Distinguish point input, mode input and dimension input:

- `3` selects3Point when offered; it does not mean origin or Length.
- `0` at a point-only step denotes the current CPlane origin. It is accepted
  after clicking Center in this audit. Dimension0 remains invalid.
- `0 → 10 → 20 → 10` demonstrates ordinary Box or Center with a numeric Length.
  It does not demonstrate picking both edge endpoints in the3Point steps.
- To follow the supplied3Point steps explicitly: select3Point, enter
  `0,0,0`, then `10,0,0`, then Width20 and Height10. The same endpoint pattern
  is valid for Vertical, with the different native base orientation.

The earlier screenshot's reported rejection of0 after Center did not reproduce
in the fresh native `Center → 0` flow, including direct keyboard input after
clicking Center without repositioning the caret or explicitly refocusing CMD.
The active-window probe records CMD focus and successful point admission.
An earlier fixture had a false focus check before explicitly activating its
window; that witness is retained and is not proof of a command focus bug.
The exact rejected input/state in that
user interaction remains unconfirmed; do not claim that issue has been fixed.
The point error wording is currently terse and does not clearly explain the
origin shorthand. Prompts also advertise additional numeric Length behavior
in3Point/Vertical. These presentation/compatibility details require separate
correction or original behavior evidence before claiming exact Matrix9 parity.

This is a verified supported slice, not complete acceptance of the entire
reference. Existing Box status remains partially implemented.
