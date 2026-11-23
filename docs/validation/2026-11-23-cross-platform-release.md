# OpenMatrix9 cross-platform release

[The release workflow](../../.github/workflows/release.yml) builds Windows x64, macOS 15 Apple Silicon and Ubuntu 24.04 x86_64 in parallel from one commit. A tagged run publishes one GitHub Release after the runners verify the extracted archives. It checks each platform's SHA-256 manifest and uploads a shared manifest for all three all-in-one archives and the standalone Windows ZIP.

The Linux tar.gz carries the pinned OCCT runtime, native 3DM exchange, both isolated BRep workers, OM9 Matrix and Ring Sizes. ELF runpaths locate bundled libraries relative to the executables and add-on. Linux uses the distribution's display and Vulkan drivers, OpenSSL 3 and Zenity file dialogs. The Zenity adapter supplies paths to the existing application owners.

Rust owns Linux worker startup, cancellation, output limits, archive staging and validation. Linux uses a child-only data limit, post-exec resident memory sampling and process-group cleanup. The native exchange uses the system OpenSSL SHA-256 implementation. The rest of the C++ changes are limited to the existing kernel and exchange adapters.

## Checks

| Validation | Result |
| --- | --- |
| Workflow and shell syntax | All four workflow files passed Actionlint 1.7.12. Both Unix build scripts passed Bash syntax checks. |
| Public source | Audit passed with 16,325 source files, 602 icon bindings and 601 SVG files. |
| Windows ownership and archive | Process owner 9/9, verified file 10/10 and atomic save 6/6 passed locally. |
| Python packaging | Package test selection passed 15/15. Release test selection passed 8/8. |
| Linux Rust cross-check | Native owner and test targets compiled. The application compiled with developer fallback features. |

[Run #1](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135048235) identified an unsupported use of the runner context in job-level environment configuration. Linux now sets its Cargo target directory after the runner has started.

[Run #2](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135159114) showed that Linux's final wait statistic could count the parent's pre-exec mappings against the worker. The Linux owner now sets the allocation limit before exec and samples the worker's post-exec resident peak. The cancellation test admits the fixture before arming cancellation, so it exercises an already-starting worker.

[Run #3](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135645007) passed Linux process ownership, verified file and atomic-save tests. GNU ld then reported zlib's callbacks into the openNURBS allocator during the native exchange link. Linux now rescans the two pinned archives for the exchange executable and controls their order in the Rust-linked workers.

[Run #4](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45136221742) linked the Linux exchange executable and built and installed the pinned OCCT SDK. GCC then flagged a missing complete Geom2d_TrimmedCurve type in the Rust-linked measurement bridge. A local Ubuntu 24.04 GCC check showed that including the owning header alone still left the implicit OCCT handle conversion unresolved. The bridge now constructs the base handle from the typed segment pointer while the original handle still owns it.
