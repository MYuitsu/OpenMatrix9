# OM9-SOLID-029 — Shell (partial)

The original standalone command accepted a selected BRep, one numeric face index and a
positive thickness. The OCCT adapter additionally supports multiple removed
faces for one closed solid while retaining at least one face. Two opposite
box openings were measured with 1 mm inward walls; invalid selections and an
open input were rejected. See `docs/validation/2026-10-28-solid-shell-source-map.md`.

At the kernel checkpoint, multiple face picking and its command flow were not yet connected to the app;
arbitrary shapes, allocation peaks, desktop UI, History and human acceptance
remain unverified. The normalized M03 Shell reference is the target behavior;
the recovered registration establishes the external command binding only.

The subsequent app checkpoint connects multiple root-face picks and Thickness
to one transaction, with Undo/Cancel and retry on failure. Tests cover a box,
simulated viewport picking and keyboard/pointer/typed Thickness input. Original
default thickness, instances, general topology and actual desktop remain open.
