# Gumball approved core completion — 2026-10-12

2026-10-12: OM9-INFO-020/021/022 `implemented` / `validated_supported_slice`;
`approved_scope_complete=true` for the approved OM9 core scope. Exact30 native
processes / 955 checks passed against one plugin/host.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Stock GitHub Actions CI is pending.

Version label: `0.3.4-202610120539`; this evidence record does not create a tag or publish a GitHub Release.

Plugin SHA256: `77fa3d086a1ed9a388d81e3dd3fa1fe6ff813bad758a5954e4e9d1880c28fda3`.
Host SHA256: `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`.
Suite SHA256: `3ccc42b7848404a193e6407749e39e4a86c1e2f52c5986234786a70ee2fd0943`.

| Native case | Checks | DPI | Report SHA256 |
|---|---:|---|---|
| geometry | 15 | 1 | `f025abe6bd088477634e222d452badf21be53e288c782acab518e9168101294d` |
| commands | 16 | 1 | `b95d5db8a0a9fb335ef37a4252d3512a7aaa830b23f1e4ea32a1395da5483489` |
| ownership | 24 | 1 | `0b8cc9e16228891478e2197e8646f0cc53171a9a69bcc2285e039b52fd220cf7` |
| review | 9 | 1 | `dcd28d23c3f8bb1e301e221e17480e3cf882bae19e15a911dda599b05c267b6f` |
| lifecycle | 18 | 1 | `40a301c9fe059db15927b75025340e5a43cac9c23f450723d91e2588c9173ca1` |
| reentrancy | 3 | 1 | `fd467cbee2b2f4979289b7edd6dfcfd6b882d4b35e1bffa7dc5762137a7a6d42` |
| cancel-publication | 2 | 1 | `7f7886e9f75d1e4cc0c6ca6f0cee6e9d26f0debf217e8caa7c437625a704400d` |
| core-dpi1 | 68 | 1 | `a9879c7348216f55456b9e09282f1c76356e389332b927f7aed0eb06f1ca2d0e` |
| core-dpi150 | 68 | 1.5 | `1e3c2c9db94d31c28eeb51b755e35da9dd197c47eee2db78cc0100a31adeb77e` |
| four-view-dpi1 | 65 | 1 | `f49db0ec76720e3d545664606b8cd406070fe6ab96bac29019465042316802d6` |
| view-lifecycle-dpi1 | 19 | 1 | `50adfc7a1c185a53e7e5ba1821d7fb4127ca23795545e1b3c5e7d355f019377d` |
| four-view-dpi1.5 | 65 | 1.5 | `dfe99dd0878c941616f0b9e8e7d6f0ed517d8724c176bf0b71c3fd2f7dfa1e21` |
| view-lifecycle-dpi1.5 | 19 | 1.5 | `99f5fe438dcf8669ff51dc9b9358d0073f11f012c5580a42c6543c531abd52a8` |
| four-view-dpi2 | 65 | 2 | `bc1821c4644725faefd170de4f702b6d76a8882aab2207aca886aa417d256f82` |
| view-lifecycle-dpi2 | 19 | 2 | `60e9e93d351f06bcddf7ba2472f1e3da8eb9f165dbe8be8d33986c3027e51222` |
| settings | 10 | 1 | `59090546f1a4de39c936f170557363145c8d98d3bbe7add2d6002aeb611914a2` |
| snappy | 6 | 1 | `a33bcf87a00cb202d746bd681b33cbeebf482a3169ff3cdb0947e831427b5859` |
| relocate_pointer | 25 | 1 | `b037a34863346276ef5d43f92da85a99b4d5322d648b89cee69584c8c63521c6` |
| owner_scale | 27 | 1 | `01a57437b8a4d77d3ebcc3df3a50511c24728abf452a0eb3344ef48973f28390` |
| owner_callback | 2 | 1 | `d70c723432d7324261f55ebe08630098ce5a5bc34c38778a317b803654dcb8ce` |
| mesh_frame | 13 | 1 | `ed94181cfa95c56002f3caf35ebbb7803f3a011ebecad0deee8a7a21f3122b9b` |
| visual | 23 | 1 | `fd40b1c6e23a678f229ff4094451785ccd6fe2dfebfc2d9b4376629ff5f7308e` |
| warm | 20 | 1 | `f964b03f019b71fd36e8a9fd3148136298dbb469f7c92bf8b0c18956544fb632` |
| cold | 17 | 1 | `21bb0d4826e8fe0fc647e1ceb6b3d01ce285f685a71334c6990db09161bbe3d5` |
| parametric | 22 | 1 | `04059787f03d19ba862604c5645996017720d3f039203198705366716ddc5f54` |
| component-pointer | 85 | 1 | `6fb373753db659bd3bea0e4acc8354c6fbcbb7795ab25a309007cb673a5cb737` |
| associative-warm | 127 | 1 | `69c6f79e3b35f1135fc8a9daad1424252be107506b5d353916d8fb544b5d5f6e` |
| associative-cold | 63 | 1 | `bcc8e0732b8a985129cff11a359a5535acfde66b25cfa0393b3a1486383fe294` |
| frame-objects-warm | 24 | 1 | `baa4b9c42813494ecde3f069233f6b512aa1a58f880c598e4a347fc7e3a76a83` |
| frame-objects-cold | 16 | 1 | `5e58e518c6d837d79a59e515f0f4ddda982e5236763d96635daaccaf69df23ae` |

[Detailed exact30 evidence](2026-10-12-gumball-core-completion.json).
[Design, applicability and approved limits](2026-10-12-gumball-core-release-candidate.md).

Native Camera/Light and associative graphs survive fresh-process FCStd. Pivot cache remains process-local by design. FreeCAD failed posttransaction callbacks can restore model/Undo but lose old Redo; pretransaction rejection retains it. Unsupported graph/geometry/ambiguous frame admission remains explicit. These limits do not imply complete Rhino/Matrix representation parity.

Stock FreeCAD1.1.4 GitHub Actions build/replay is **pending**. Earlier v0.0.3 Actions results do not validate this source or binary. The local suite establishes only the recorded host/plugin combination.

Direct native Cylinder radial/axial Scale is admitted; it is not an unsupported-owner rejection fixture. Protected incoming native links reject before publication. Alt Copy selects the new copy; Undo can remove that selection, so Ctrl Relocate fixtures explicitly reselect the original before the view-only operation.

Stock-header regression: the first candidate Action failed before native verification because the stock compatibility prelude left Windows FAR pointing to the removed far macro, and the bundled openNURBS zlib include path shadowed FreeCAD’s system header. The old prelude reproduced the native compiler failure. The corrected prelude and private bundled-header boundary now compile and pass the independent unprefixed CRC32 oracle. The 30-case cohort above was rerun after this plugin rebuild. The exact stock Action result remains separate from this local evidence.
