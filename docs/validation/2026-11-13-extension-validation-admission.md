# Extension validation scratch admission

Document validation previously counted extension storage by calling
`Extension::validate`, which parses JSON, before admitting document validation
scratch. The scratch estimate also did not include extension parsing. This
affects trim metadata and other entity extensions, independently of the 16 KiB
default per-extension limit.

The document storage pass and three transaction request storage-count paths
now count the owned bytes without parsing JSON. Document validation reserves a
conservative 64× largest extension payload plus 4096 bytes alongside its other
scratch before parsing; extensions are validated sequentially. This is an
accounting estimate, not a measured bound on native allocations or all callers.
It can reject a tight-budget operation earlier than before. The later semantic
validation is retained. A malformed-payload fixture proves document validation
rejects on budget before parsing when the estimated peak cannot fit, and a
transaction fixture verifies malformed metadata still fails without changing
the document or Redo.

Checks: core library 377 passed/1 ignored; document-graph 35 passed before the
additional transaction fixture, which passed separately; semantic and retained
peak tests passed; full archive suite passed; seven scoped native 3DM fixtures
passed using the P1 Release adapter. Native transient memory, actual physical
allocation failure, other validation entry points and desktop remain open.
This change does not increase the trim metadata size limit or implement
large-trim save/open.

## Request-owned metadata — follow-up

New extension data can be parsed while planning a transaction, before the
candidate document exists. The planning reservation now includes the same
conservative scratch estimate for the largest request-owned extension on
insert, extension replacement, definition insertion or prepared output. In a
focused pressure fixture, insufficient managed budget rejects before parsing;
once pressure is removed, the intentionally malformed payload reaches semantic
validation and is rejected. Both outcomes keep the document and Redo. Core
library 377/377 (one ignored), document graph 37/37, five trim archive tests and
seven real native 3DM file fixtures passed. This verifies the selected paths,
not actual physical out-of-memory or all caller allocations.
