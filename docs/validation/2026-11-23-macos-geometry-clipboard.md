# Geometry clipboard on macOS Apple Silicon

The public prerelease contains the all-in-one ZIP and its checksum. The ZIP was downloaded from the public release without credentials. Its 41,411,948 bytes matched the CI SHA-256 `70af1009157288747163519202ab2d03b712c45956257bc5447ba9cfac55bec6`. [GitHub Release](https://github.com/MYuitsu/OpenMatrix9/releases/tag/macos-v0.3.6-202610211407-clipboard.1) · [Apple Silicon ZIP](https://github.com/MYuitsu/OpenMatrix9/releases/download/macos-v0.3.6-202610211407-clipboard.1/OpenMatrix9-0.3.6-202610211407-All-in-One-macOS-arm64.zip) · [SHA256SUMS.txt](https://github.com/MYuitsu/OpenMatrix9/releases/download/macos-v0.3.6-202610211407-clipboard.1/SHA256SUMS.txt)

Edit → Copy uses the OM9 archive and Edit → Copy as 3DM uses the existing Rhino V5 export path. Paste reads the AppKit pasteboard and dispatches to the same Controller as Windows. Copy leaves the source document and Undo stack intact. Paste commits all imported objects in one transaction. The two AppKit pasteboard types are scoped to OM9.

The local AppKit event monitor observes only the OM9 window. It forwards Command+C, Shift+Command+C and Command+V to the existing geometry shortcut router. Active text editors, IME and modal command states retain keyboard ownership. AppKit and borrowed view access stay in `om9-native-owner`. The Controller owns geometry validation and history.

## Validation

- Windows x64 `clipboard_om9` passed 25/25 with the pinned 3DM adapter and `RUST_MIN_STACK=16777216` for the debug harness. The first run hit a debug thread stack overflow in a clean-history fixture. The regression suite passed with the larger test stack.
- Windows native clipboard unit tests passed four checks, including shortcut classification and detached clipboard payload capture. Three interactive clipboard tests were skipped.
- The application and native adapter cross-checked for `aarch64-apple-darwin`. [Apple Silicon run #14](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45122869384) passed the native shortcut preflight and found that Copy still used the v5 writer for canonical BReps. Copy now uses the existing v6/v7 encoder when the selection contains BReps. Paste restores their geometry and keeps the verified source packets so the pasted objects can be saved or copied again. The new Windows regression covers Box Copy, Paste, recopy, Undo and Redo.
- [Apple Silicon run #15](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45125957196) passed at commit `933e355a6f14d2ece5e060c45ad066ffc8bd615a`. The extracted app published both formats on a private AppKit pasteboard and restored a Box BRep and the point at (2, 3, 4) into distinct destination documents. Each Paste created new object IDs and one Undo entry. Undo removed both objects and Redo restored both. Shortcut mapping passed for Command modifiers and a non-Latin keyboard fallback. The standalone BRep Save/Open, 3DM exchange and two bundled add-ons also passed.

The clipboard test exercised the native pasteboard and command handlers in CI. Physical macOS desktop interaction and exchange with Rhino for Mac remain unverified. The bundle is ad hoc signed.
