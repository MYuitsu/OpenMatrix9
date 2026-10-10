# OM9-FILE-008 / OM9-INFO-006 — Notes

Source contract: local 01-core Notes and Project Notes specs. File Notes saves when X closes its editor; Project Notes uses Done. Both retain notes with the project. No original command token is supplied in either spec; existing wrapper IDs remain OM9_FileNotes / OM9_ProjectNotes. Notes / ProjectNotes are host CMD aliases.

Host implementation decisions: UTF-8 multiline text, empty text removes the metadata value, 1 MiB storage limit and no NUL. Esc cancels; Project Notes X cancels. Both entry points share a per-document value. There is no geometry, selection input, units, geometry preview or geometry dependency history.

Native document storage uses a hidden App::FeaturePython data object with Notes and OM9NotesSchema properties, initialized to prior text outside the editing transaction. Only accepted text changes create an Undo transaction. Native Undo/Redo synchronize the Meta project-note cache; unrelated metadata is preserved. The data object has no Shape, is hidden from the tree/render, and OM9 Select All excludes it. Baseline initialization preserves preexisting metadata-only text on first-edit Undo. The data container is infrastructure, retained after undoing text to empty.

Menu/API and CMD use CoreNotes::execute. Notes cancels pending Curve/view tools before opening the modal editor. Originating document identity and editing permission are rechecked after it closes; closing that document discards the proposed edit. Successful history waits for changed text; unchanged/escaped edits return false.

Evidence so far: Rust two semantic tests; native expanded check run 7521d136b03f47f7b2e2531b2dfd6ea5 passed 26 checks including X/Done/Esc, Unicode, shared data, Undo/Redo, first-edit migration, FCStd reload, independent docs, selection preservation, task restrictions, originating-document closure, and Select All/Delete. See docs/validation/2026-10-05-core-view-controls.md for related modal/camera fixes. Final post-review regressions are pending.
