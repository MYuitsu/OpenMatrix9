# STL receipt scope and source identity

The open-surface receipt previously omitted the fact that self-intersection was unchecked. For authored meshes it could also omit an explicit global-deviation status. The reported edge/fan counts are calculated on a source-scoped indexed soup before binary encoding; STL does not preserve those indices or standardized units.

The user-facing message now reports the requested linear/angular settings, assumed millimetres, absent STL unit metadata, unverified global deviation, unchecked self-intersection and the scope of topology counts. The structured receipt additionally retains the captured document ID and epoch. MCP exposes the document ID, topology scope, unverified deviation and absence of STL unit metadata, retaining its existing fields for compatibility.

The open coincident-objects fixture failed before the message change and passes after it. All 29 STL integration tests passed, including source capture followed by editing/deletion; the capture's document ID and epoch are asserted in the resulting receipt. The MCP profile test verifies the added fields and existing destination preservation after an invalid profile. This does not check self-intersection, establish a global deviation bound, certify encoded topology or verify desktop presentation. Background export and aggregate scratch reservation remain incomplete.
