# Tools original alias routing

The local milestone references identify `gvOutsideRingRail` and `gvImageTrace` as original command names. Both names are now explicit aliases in the standalone Tools and generated Matrix package manifests and help. The host previously resolved these names through the feature catalog; the explicit aliases remove that dependency when the feature-name table is absent.

Headless package tests installed each package with the in-process Matrix runtime. With an empty feature-name table, the menu icon and original alias reached the same add-on command and produced equal curve geometry: a default Outside Ring Rail and a trace of a generated dark-square PGM. Each result had one Undo. With the package disabled, the icons remained visible as built-in fallbacks: `OM9-TOOLS-003` was enabled and `OM9-TOOLS-016` disabled. These checks do not establish full Matrix behavior, real native DLL loading, desktop clicks, image formats beyond the PGM fixture, or exact-source geometry.

The standalone package manifest had a tagline longer than the package validator allows; shortening it allowed the same installation test to run. The focused Matrix package test verifies both manifests and both generated command-help pages, including alias uniqueness. The full `package` test target has an unrelated pre-existing generated README mismatch in Solid menu position numbers; no unrelated generated documentation was overwritten.

Validation used `L:/om9-target` and `-j 2`: the alias route fixture passed with default and no-default features; the default add-on session suite passed 16 tests with two ignored, and the no-default suite passed 14 with one ignored before the final fallback assertions were added. The focused package parity fixture passed.
