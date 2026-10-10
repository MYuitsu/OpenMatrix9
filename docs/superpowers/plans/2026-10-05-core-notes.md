# Notes implementation plan — OM9-FILE-008 and OM9-INFO-006

Execute inline under the full 01-core goal using local specs, without PDF reads.

- Preserve catalog wrapper IDs OM9_FileNotes and OM9_ProjectNotes; specs supply no original command token. CMD accepts those IDs plus Notes / ProjectNotes as documented host aliases.
- Both entry points edit one Unicode, multiline per-document project note in Meta[OpenMatrix9.ProjectNotes]. File Notes saves when its X is clicked; Project Notes has Done. Esc cancels. Close/done commits one Undo transaction only if text changed, preserving unrelated metadata and selection. Geometry is unchanged. Reopening starts with saved notes; FCStd save/reload persists them.
- Rust validates UTF-8, embedded NUL and a 1 MiB host storage bound and decides changed/no-op/invalid. C++ owns QTextEdit, document permissions/lifetime, native transactions and storage. Capture originating document identity and recheck it after the modal editor.
- Test Rust Unicode/empty/no-op/invalid boundaries first, then actual menu and CMD dialogs, X/Done/Esc, shared text, independent documents, Undo/Redo, FCStd reload and restricted tasks.
- Record precise evidence in the core ledger. Continue all remaining requirements; these two specs do not redefine completion.
