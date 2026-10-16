# Manuals split into OM9_Milestones function folders: verification review

The four manuals (Matrix 8 book 1 and 2, Matrix 9 addendum, Rhino 5) were split into per-section text and images and placed as `reference/` subfolders under 785 of 893 function folders. A top-level `MANUALS_REFERENCE_INDEX.md` lists coverage, unmatched functions, unassigned sections and OCR status. The evidence in `fix_report.md` covers every required check, and the samples I opened agree with it.

Watch for: reference folders are not git-ignored and add about 448 MB of Matrix manual excerpts and bitmaps, which conflicts with the AGENTS.md publication rule (non-blocking, see below). Longest created path is 249 chars, 1 under the cap.

**Verdict**: APPROVED

## High-level view

Placement is relevance-driven rather than bulk-copied. Each function folder gets a `REFERENCE.md` with source PDF, page range, section title, confidence, a text link and an image count. Text lives under `text/` and figures under `images/`. L2 folders hold shared copies and sibling L3 rows with `link_to_L2` point back to them instead of duplicating. The sampled mappings were on-topic.

Coverage is 785 of 893 function folders. The 93 unmatched functions are listed with reasons (mostly backend or framework functions with no manual counterpart). 42 of 715 sections are unassigned, each with a reason (mostly generic tutorials). Nothing needed OCR because all four PDFs have 0 scanned pages.

The change is purely additive. The report shows all 1799 pre-existing files still present with no later mtime, `git status` shows only `??` entries under OM9_Milestones, and the `manuals/` PDFs are untouched.

<details>
<summary>Issues (3)</summary>

1. **Reference folders not git-ignored (likely, non-blocking)** — `git check-ignore` returns no match for a `reference/` path, and `.gitignore` has no pattern for them. AGENTS.md says never import Matrix manuals or commercial bitmaps into the repo and never stage ignored private references. The user explicitly asked for this placement, so it is not blocking. Add a `/OM9_Milestones/**/reference/` ignore (and the index if wanted) or confirm the folders are meant to be local only, before anyone runs `git add`.
2. **Path length headroom (confirmed, non-blocking)** — Longest created path is 249 chars against a 250 limit. Moving the repo to a deeper root could break Windows MAX_PATH. Keep the repo root short or enable long paths.
3. **Boundary-page text and a duplicate image (confirmed, non-blocking)** — `m8book1__p40-40__smarttrack.md` is the whole two-column page 40, so it also contains Selection Filter and Library Menu text. The SmartTrack row lists `m8book1_p34_img01.png`, a dedup artifact of a page-40 image. Both are labelled as shared-page in the headers, and the Rhino 5 SmartTrack row covers the topic cleanly. Consumers should treat boundary pages as approximate.

</details>

<details>
<summary>Details</summary>

## Sampled placements

I opened REFERENCE.md in six folders across M02, M04, M05, M06 and M07: SmartTrack Toggle (M02), Ring Builder Style Presets (M04), Baguette Channel Layout (M05), Bead On Surface (M06), and Gem Cutter Library Profiles (M07). Each maps to a clearly matching manual section: SmartTrack to the SmartTrack sections, Baguette Channel to Baguette Builder and Baguette Channel (m8book2 pp 214-218), Bead On Surface to m8book2 pp 263-264, and Gem Cutter Library to m8book2 pp 279-280. The M04 style-presets row (Styles, pp 30-31, medium confidence) is plausible and is correctly labelled medium. Links use relative `text/` and `images/` paths, and the files I checked exist on disk: Bead On Surface has 8 images and 1 text file, and SmartTrack has 6 images and 2 text files with sizes consistent with the listed counts.

REFERENCE.md formats differ from folder to folder (column order, naming, and whether text links are clickable). This is cosmetic.

## SmartTrack boundary page

The M02 SmartTrack case is the one place I dug into. The m8book1 source section is a single page that starts shared with the previous topic, so the extractor emitted the full page 40 text. The SmartTrack body is present further down the page. The p34 image entry comes from the extractor marking a deduplicated image on page 40.

## Publication risk

AGENTS.md forbids importing Matrix manuals and commercial bitmaps into the repository and tells agents never to stage ignored private references. `.gitignore` covers only `/private-references/`, so the new `reference/` folders are not ignored and would show as untracked. Several REFERENCE.md files carry a private-reference warning, so the intent is clear. A `.gitignore` entry was not added, since the task rules out modifying existing files.

</details>

<details>
<summary>File map</summary>

- `OM9_Milestones/**/reference/REFERENCE.md`, `text/*.md`, `images/*.png`: new per-function manual extracts (7793 added files)
- `OM9_Milestones/MANUALS_REFERENCE_INDEX.md`: new coverage and navigation index
- `.agents/tasks/manuals-split/fix_report.md`: coder verification evidence

</details>

