# Fix report (iteration 1, cross-part integration)

Checks were run with `work/verify_all.py` (re-runnable), `work/mtime.py`, `work/spot.py`.

(a) Page counts: all 4 PDFs fully processed. Real page count = index.json page_count = pages entries = text page files.
m8book1 246, m8book2 342, m9addendum 94, rhino5 284. scanned_pages is 0 for all, so no OCR flags are needed (OCR engine unavailable, but nothing needed it).

(b) Text: 0 empty, 0 non-UTF-8, 0 mojibake/U+FFFD files across all placed text files. Vietnamese/English spot-checks OK.

(c) Images: 5877 placed files (2367 distinct names), 0 zero-byte, 0 unopenable, 0 blank, no size mismatch between copies. Viewed 4 images (ring channel cutter, Clayoo emboss, etc.): real figures.

(d) Links: 4167 relative links across 785 REFERENCE.md + MANUALS_REFERENCE_INDEX.md. 3 were broken on first run
(`images/` pointing at a directory that does not exist because the row has 0 images, in ZAFTER L2-10-02 L3-001 and L3-002 REFERENCE.md).
FIXED by replacing "0 in [images/](images/)" with "0 (no figures)" in those two new files. Re-run: 0 broken.

(e) No pre-existing file changed: 1799 files in pre_placement_filelist.txt all still exist, none has an mtime after the snapshot,
and 7793 added files are all under `reference/` or are MANUALS_REFERENCE_INDEX.md. `git status --porcelain -- OM9_Milestones` shows only `??` entries (no M/D).
`git diff --name-status --diff-filter=MDRT c4ea369 HEAD` is empty (earlier commits 5170cfd/64836a0/b4a20b0 for M02/M06/M08 are pure additions). `manuals/` PDFs unmodified (git status clean for manuals).
Other repo changes visible in `git status` (.gitignore, AGENTS.md, OpenMatrix9_Codex_Spec_v1 deletions) are outside OM9_Milestones and not from this task; left untouched.
Note: I made no git commits this iteration.

(f) Longest created path: 249 chars (< 250), 0 paths >= 250. It is close to the limit; moving the repo to a deeper root could exceed Windows MAX_PATH.

(g) Relevance: spot-checked 36 function folders (two random samples) across M01-M08 and ZAFTER. Placements were relevant (e.g. Ring Rail, Scallop Bezel, Microprong Cutter,
Array Along Surface, Fillet Corners, Clayoo Bezel, Carbon Copy, F6 menu, Snaps). No wrong placement found, so none changed.
Observations, non-blocking: M03 L2-02-04 L3-007 has a low-confidence link to Rhino 5 "Penguin" (pp 173-214), and a few ZAFTER import/export rows are low confidence; they are labelled as such in REFERENCE.md.
