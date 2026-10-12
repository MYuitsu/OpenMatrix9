# RCORE-01 — Publication boundary before native transaction finalization

Source review: 2026-10-11. **Open; no native completion claim.**

The output-created callback fixtures expose real Builder/Surface Redo loss.
The caller corrections use the existing opt-in native API, check sources again
after booking, and retain existing solvers. They do not close the full atomic
publication requirement.

The final source/output checks still precede `doc.commitTransaction()` in
`Gui/BuilderHistory.cpp`, `Gui/SurfaceGeometry.cpp` and `Gui/EditGeometry.cpp`.
`src/App/Document.cpp::_commitTransaction` dispatches
`Application::TransactionSignaller(false, true)` while the native active journal
still exists. Its constructor emits `signalBeforeCloseTransaction` without the
reviewed property-observer scope. A same-thread global callback can therefore
change a staged source after the caller's final verdict. This is a source-backed
publication gap; a real native callback fixture is required before claiming a
specific runtime result or a correction.

Additional global lifecycle questions remain in the same native boundary:

- Opt-in `_abortTransactionChecked` does not create the signaller used by default
  `_abortTransaction`; the outer application close signaller has `signal=false`.
  Verify global before/close notification counts for one and multiple documents,
  empty bookings and failed rollback.
- The signaller constructor changes static counters before dispatch. An observer
  exception can leave those counters changed because a failed constructor has no
  destructor. Isolated historical RED12 witnesses retained the resulting test
  process contamination; verify native recovery with a subsequent transaction.
- Commit clears `bookedTransactionID` before that dispatch. If the callback
  throws, the active journal can remain while a later application abort cannot
  find its document by the cleared booking. Capture every participant before
  notifications and attempt rollback through that vector.
- The signaller destructor catches `boost::exception` only. Native standard or
  unknown exceptions need a bounded, non-terminating atomic verdict and genuine
  history cleanup; determine the default compatibility contract separately.
- A thread-local observer guard does not transfer through worker-to-main-thread
  signal dispatch. Same-thread tests cannot certify that path or arbitrary
  virtual callbacks.

The next native witnesses must exercise before-close source/property mutation,
throwing before/close observers, exact global notification counts and subsequent
transaction recovery. Preserve real native Redo and checked multi-document
rollback. Correct the native finalization boundary and verify its default behavior
before treating caller publication as atomic. Native App changes require a fresh
target-only shared SDK rebuild; an OM9-only plugin change does not.

The proposed correction needs explicit checked begin/finish notifications in the
application path, with aggregate exceptions and exact counter ownership. Before
dispatch must precede booking removal; close dispatch must precede final held-Redo
disposition. If a close observer cancels commit, preserve the exact native
journals and postpone Undo trimming/Redo disposal until that verdict, then use
checked rollback to undo the staged publication. Calling close before native
commit would misrepresent lifecycle ordering. Keep public stack-object layouts
and existing exported constructors compatible, or explicitly account for every
affected SDK consumer; adding fields to `TransactionSignaller` can change its ABI.

All original RCORE-01 representation families, consumers, conversion/global
deviation and maps, attributes, persistence/export, ownership/cache and aggregate
budgets remain required. This review adds evidence to the existing objective.

## Native witness follow-up — API11 / 2026-10-11

The isolated `AtomicPublicationCloseTest.*` unit now has actual runtime evidence:
`build/atomic-host-tests-close-red2-20261011/isolated-boundary.xml`, exit1,
one pass/two failures. It uses a fresh publication-close unit and SDK bootstrap
with cached other App dependencies; it is separate from the passing root
Document/App executable.

- `DefaultAbortKeepsGlobalBeforeClosePair` passes.
- `AbortSignalsOnceAfterAllParticipants` fails. Actual events are
  `abortA, stableA, abortB, stableB`: neither global before nor global close runs.
  Actual native Redo IDs and subsequent replay still pass; that does not make
  the missing lifecycle notifications acceptable.
- `BeforeCommitCannotOverwriteValidatedSource` fails. The before-close callback
  writes `Late observer value`, the participant booking/application description are
  already inactive, and Redo retains that late value instead of the validated
  value. This is now runtime-proven, beyond the original source-only finding.

The refined witnesses also require booked IDs during before notification and
registered native object identity after rollback/Redo. Their deterministic A/B
ordering follows `DocMap`; the requirement is the group lifecycle and ordering
relative to history disposition. Before/close exceptions, counter recovery,
empty-booking lifecycle and deferred commit retirement remain unproven and
must be added before finalizing a native correction.

The independently fixed unbooked-deletion/default/group booking regressions are
verified by root native46/46 and App602pass/2skip/0fail of604. Runtime21/API11
has462 checks across16 actual host exits0, including unchanged Builder History43/43
and Builder18/Surface30 callback checks. These passing tests do not supersede
the two global-boundary failures. [Evidence and remaining scope](../validation/2026-10-11-rhino-core-publication-booking.md).

## API12 witness follow-up and unverified API13 candidate

Eight original global cases pass API12; four corrected admission/history and four refined nested/global cases fail. API13 source objects compile, but DLL/runtime remain unverified while the shared Gumball host is open. API11 close-commit fatal process exit is3 (wrapper exit is not the process result). See [the current evidence](../validation/2026-10-11-rhino-core-global-close.md); all original requirements remain open.

### Additional source-only witnesses required after API13 linkage

These are review hypotheses, not executed native evidence or accepted fixes:

- Compose a protected close of a prior default global transaction with a nested
  two-document preserving move abort. Roll the receiving document back first,
  then the original property source. Checked definition on both documents must
  let native reverse-move preflight access the completed receiver despite the
  still-closing outer global ID. The new global-admission predicate currently
  does not distinguish that checked internal definition; verify exact property
  ownership/value and both executable native Redo histories.
- `signalNewObject` and delete lifecycle callbacks currently lack an explicit
  `_atomicObserverScope`. During checked object restoration, attempt to modify
  an untouched attached object from the new-object callback. Require rejection,
  unchanged unrelated properties and exact restored native identity/Redo. The
  per-document baseline must not turn an actual structural observer into an
  allowed native rollback write.
- Invoke an independent inner abort from a property observer during outer
  checked rollback; make the inner rollback fail and handle its false return.
  `Transaction::applyChecked` propagates failure into a previous thread-local
  verdict. Verify whether distinct transaction IDs retain independent verdicts
  when the inner failure has not altered the outer participant, rather than
  silently disposing outer Redo. Nested-before witnesses do not cover this
  nested-during-rollback boundary.

Do not label the API13 candidate accepted before these structural/native
ownership interactions and the existing regression cohorts are verified.

## Isolated API19 native follow-up

The structural, activation, receiver-first nested move, independent nested verdict, interleaved same-ID verdict and default nested apply hypotheses now have actual native RED → GREEN evidence. Final global28/28, root46/46 and App602pass/2skip/0fail of604 pass with actual selected DLL telemetry. Shared SDK remains API12; matching OM9 GUI/runtime and full chapter requirements remain open. See [isolated native evidence](../validation/2026-10-11-rhino-core-isolated-publication.md).
