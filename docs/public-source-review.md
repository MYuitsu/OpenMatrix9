# Publication source review — 2026-10-06

This is a technical preparation of the current OpenMatrix9 worktree for a new
public snapshot. It is not legal clearance, a complete security audit, a
clean-room claim, or evidence that every catalog command is implemented.

## Exclusions and replacement

- All 510 icon bindings now point to 510 independently authored, self-contained
  SVG files. Replaced the remaining 142 extracted PNG bindings and 12 generic
  auxiliary symbols with 154 command-specific core/sidebar SVGs.
- Preserved the previously authored Curve, Surface, Solid, Transform, Art,
  Builder, SubD, Tools, Gems, Setting, Cutters and Render families and approved
  color choices. Reused catalog precedence preserves shared assets.
- Inspected all 154 original/new pairs at 24px/large size, plus gray/dark sheets.
  The 154 new icons have no opaque paint touching the 32px canvas boundary.
- Removed all unbound artwork, including 497 reference PNG files and obsolete
  category SVGs. No proprietary bitmap data or logos are embedded in the SVGs.
- Moved `ref/`, `analysis/`, imported/derived research documentation, inventory
  dumps, private 3DM fixtures, research skills/assets and extraction tools to a
  private backup outside the repository. Transient build/cache artifacts are
  also kept outside the publication tree.
- Replaced imported F6 XML with a small independently declared configuration
  using functional command identifiers. Unsupported entries remain disabled;
  proprietary class metadata, translation IDs and the original table are absent.
  The context-menu label is SubD. Empty/unimplemented categories keep the host's
  existing disabled placeholder. This is a deliberate reduction of the imported
  context configuration, not a full reconstruction of its original contents.
- Replaced the asset exporter with authored-catalog-only export. External
  bitmap/archive sources are rejected, including when legacy flags are supplied.
  Unknown command keys fail instead of receiving generic symbols.
- Removed private developer path fallbacks and private fixture-dependent ring
  tests. Preserved the existing LGPL-2.1-or-later project declaration and added
  the complete license, third-party notices and pinned upstream openNURBS license.

Private reference records remain only in the external backup. Public manifests
retain authored design explanations, functional IDs, color roles and file hashes.

## Verification

| Check | Result |
|---|---|
| Source publication audit | 510 bindings, 510 SVGs, zero errors |
| Authored exporter coverage | 510 resolved, zero missing; byte-idempotent |
| Python unit suite | 23 passed |
| Rust suite | 68 passed |
| Native FreeCAD build | RelWithDebInfo OpenMatrix9Gui built successfully |
| Native catalog/sidebar/F6 smoke | 3,093/3,093 passed |
| Qt SVG rendering | All 510 loaded at 32px and rendered at 523px |
| Actual menu/auxiliary images | Order, pixels and disabled color roles checked |

The successful native run used an isolated FreeCAD SDK runtime with a separately
linked OpenMatrix9 module built from this source. A concurrent build from an older
development copy was observed overwriting resources/native outputs in the shared
SDK. Its stale output is excluded from this snapshot. The isolated runtime also
required FreeCAD's standard module support files; binaries are not redistributed
here. Native validation artifacts remain in the external private backup.

Gitleaks 8.30.1 checked all 16 original commits with no secret findings. The old
tree scan flagged three command-identifier records in ignored private build
metadata; those files were excluded with the entire transient build tree. Final
clean-tree and rewritten-history scan results are recorded in
`public-source-checks.json` without secret values or private filesystem paths.

## Git history and publication limits

At the owner's request, the local Git repository was replaced with one clean
root commit containing the reviewed current source. The former `.git`, Git
bundle, working-tree snapshot and quarantined files are stored outside the repo.
No old branches, tags, reflogs, alternates or objects are carried into the new
repository. The original remote configuration is kept privately rather than
attached to the cleaned repository.

No GitHub push, force-push, repository deletion or visibility change was made.
The old remote history is **not** cleaned by replacing local history. Do not make
the old remote public unchanged. Publish the reviewed snapshot to a fresh remote,
or deliberately replace the old remote's refs and address any retained copies.
GitHub explains the limits of history removal and cached/forked copies in
[Removing sensitive data from a repository](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository).

## Remaining provenance and behavior limits

Development used Matrix9 as a reference. Removal of distributed proprietary
materials does not establish rights in every implementation detail. Independent
review of code provenance and applicable agreements is still needed before
claiming legal clearance. Compatibility identifiers and historical functional
cues remain, with no vendor affiliation claim.

User Library/Add, View Manager and Knot Snap lack dedicated local command
specifications and use functional/reference cues. Twelve Layer/Project/Isometric
controls had no original PNG and retain their previously authored blue/gold
palette. WebViewer and the Render Styler parent retain their previously documented
semantic uncertainties. Visual checks do not prove user recognition.

No new CAD algorithms, manufacturing integrations or commercial SDK adapters
were enabled by the icon replacement. Tests needing private design inputs are
not represented as reproducible public-fixture checks.
