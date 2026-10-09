# Private reference policy

Recovered commercial source, raw decompiler output, native resource blobs,
commercial images, original Matrix manuals and private models belong in a private
archive outside this repository. The archive preserves the original inputs and
their applicable license notices for future reference work.

The user's Rhino 5 documentation exception is the exact original guide and
the three reference metadata files in [ref/rhino5](../ref/rhino5/README.md).
The public source audit verifies the PDF path, byte count and SHA256 rather
than allowing PDFs or reference directories generally. Its official download
source and original notices remain recorded in `ref/rhino5/SOURCES.json`.
Public availability is not a statement that the document is open-source.

Use the ignored local registry `docs/openmatrix9-reference-locations.json` to
resolve logical source names to their current private locations. This includes
the former `code/` inputs and the manuals `matrix8_book_1.pdf`,
`matrix_8_manual_book2.pdf` and `matrix9.0_addendum_manual.pdf`. Check that the
registered path exists before using a source. Keep machine-specific archive
paths in that registry.

Historical evidence citations retain the locations used when the evidence was
recorded. Resolve their logical source name or manual filename through the
registry when reading the source again; an old repository-relative citation
does not require restoring the private input inside the checkout.

Keep OpenMatrix9-authored implementations, behavioral summaries and engineering
specifications in the repository, with the applicable attribution and license
notices. Do not copy raw recovered source, decompiler listings, commercial
images, native resources or private manuals into those deliverables. Never
stage or push private references or the local registry. Keep public
documentation and package manifests free of private input contents.

Preserving an input does not mean all of its information has been read or
interpreted. Future work may read the relevant manual pages or a bounded
procedure and necessary context from the private archive. Record the source,
page or procedure identity, verified behavior and remaining uncertainty for
that work. Archive preservation and a local source audit do not establish
license clearance or complete command implementation.
