# OpenMatrix9 cross-platform release

[The release workflow](../../.github/workflows/release.yml) builds Windows x64, macOS 15 Apple Silicon and Ubuntu 24.04 x86_64 in parallel from one commit. A tagged run publishes one GitHub Release after the runners verify the extracted archives. It checks each platform's SHA-256 manifest and uploads a shared manifest for all three all-in-one archives and the standalone Windows ZIP.

## Published result

[Run #8](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45149973934) completed all six jobs at commit `c2b91d0d67c70caa34dea846bbfdfeef9b845599` and published one [GitHub prerelease](https://github.com/MYuitsu/OpenMatrix9/releases/tag/preview-v0.3.6-202610211407-cross-platform.8). All four archives were downloaded anonymously after publication. Their sizes and SHA-256 digests matched the public [checksum manifest](https://github.com/MYuitsu/OpenMatrix9/releases/download/preview-v0.3.6-202610211407-cross-platform.8/SHA256SUMS.txt) and release metadata.

| Archive | Bytes | SHA-256 |
| --- | ---: | --- |
| Windows x64 all-in-one | 42,890,658 | `3cde26870b18f1f616a28b44ad4a8721186bb7ee098350f467a97c90da0ba255` |
| macOS arm64 all-in-one | 41,411,892 | `307e404650818a42b8cda400e5880446c68ec1c7f17590b9ccadca552613dd2c` |
| Linux x64 all-in-one | 53,582,505 | `35d95953360903a0c5e3d0a48ed4395be2af2e140bc2abb13dcd3911ee47e0bc` |
| Windows x64 standalone | 38,879,920 | `27d0e9a6b6173c00fdfe544cb51593a70ad774790201eeb9393d81072b21431a` |

All three extracted all-in-one packages passed a Box BRep save and reopen, a native 3DM round-trip, and Matrix and Ring Sizes loading. Apple Silicon also passed Copy and Copy as 3DM followed by Paste, Undo and Redo. Linux passed 9 process-owner, 4 verified-file and 6 atomic-save tests. Its Xvfb probe reached the renderer and confirmed the expected software GPU rejection. Interactive use on Linux requires a supported hardware GPU. The macOS bundle is ad-hoc signed.

## Linux package

The Linux tar.gz carries the pinned OCCT runtime, native 3DM exchange, both isolated BRep workers, OM9 Matrix and Ring Sizes. ELF runpaths locate bundled libraries relative to the executables and add-on. Linux uses the distribution's display and Vulkan drivers, OpenSSL 3 and Zenity file dialogs. The Zenity adapter supplies paths to the existing application owners.

Rust owns Linux worker startup, cancellation, output limits, archive staging and validation. Linux uses a child-only data limit, post-exec resident memory sampling and process-group cleanup. The native exchange uses the system OpenSSL SHA-256 implementation. The rest of the C++ changes are limited to the existing kernel and exchange adapters.

## Checks

| Validation | Result |
| --- | --- |
| Workflow and shell syntax | All four workflow files passed Actionlint 1.7.12. Both Unix build scripts passed Bash syntax checks. |
| Public source | Audit passed with 16,326 source files, 602 icon bindings and 601 SVG files. |
| Windows ownership and archive | Process owner 9/9, verified file 10/10 and atomic save 6/6 passed locally. |
| Python packaging | Package test selection passed 15/15. Release test selection passed 8/8. |
| Linux Rust cross-check | Native owner and test targets compiled. The application compiled with developer fallback features. |
| Release staging | The stage script verified actual archives from run #7 and rejected a fixture with a corrupted Linux checksum. |

[Run #1](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135048235) identified an unsupported use of the runner context in job-level environment configuration. Linux now sets its Cargo target directory after the runner has started.

[Run #2](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135159114) showed that Linux's final wait statistic could count the parent's pre-exec mappings against the worker. The Linux owner now sets the allocation limit before exec and samples the worker's post-exec resident peak. The cancellation test admits the fixture before arming cancellation, so it exercises an already-starting worker.

[Run #3](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45135645007) passed Linux process ownership, verified file and atomic-save tests. GNU ld then reported zlib's callbacks into the openNURBS allocator during the native exchange link. Linux now rescans the two pinned archives for the exchange executable and controls their order in the Rust-linked workers.

[Run #4](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45136221742) linked the Linux exchange executable and built and installed the pinned OCCT SDK. GCC then flagged a missing complete Geom2d_TrimmedCurve type in the Rust-linked measurement bridge. A local Ubuntu 24.04 GCC check showed that including the owning header alone still left the implicit OCCT handle conversion unresolved. The bridge now constructs the base handle from the typed segment pointer while the original handle still owns it.

[Run #6](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45139926929) built the full Linux app and passed the extracted BRep, 3DM and add-on tests. The Xvfb startup check then found the XKB X11 library missing. The tarball now carries the XKB runtime libraries and their distribution notices. Ubuntu 24.04 GCC also passed syntax checks for all 15 OCCT and openNURBS bridge sources in a local Linux container.
[Run #7](https://github.com/MYuitsu/OpenMatrix9/actions/runs/45145124836) passed the extracted Linux BRep, 3DM and add-on smoke again. With XKB in the package, X11 initialization reached wgpu. The GitHub-hosted runner offers only a software adapter, which OM9 rejects under its existing hardware-GPU requirement. The workflow now checks that classified startup outcome on Xvfb with Mesa software Vulkan. A full interactive Linux desktop session needs a hardware GPU.
