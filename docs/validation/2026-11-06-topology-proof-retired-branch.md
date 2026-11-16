# Retired-branch proof pruning (withdrawn)

**Withdrawn:** the pruning rule described in the original experiment was
removed after a subsequent regression exposed a false negative. A different
document allocation can still retain the same B-rep source and display after
the originally attested document has died. Pruning solely from the original
document's weak reference can discard a proof that a surviving history snapshot
could use. The cache now conservatively keeps same-session proof entries until
optional-memory reclamation; lookup and use still validate the current entity.

The optional proof cache formerly retained every proof from the same document
session until memory reclamation, even after its original document and Redo
branch were no longer reachable. The withdrawn experiment removed an entry
when its originally attested document has no strong owner and the current
entity does not match the proof's generation and display fingerprint. The
existing weak document reference supplies this retention hint; it does not
validate proof or grant access to source data. Normal full proof validation
still precedes remesh.

A Sphere fixture first undid creation and confirmed Redo kept its proof. A new
Point transaction then discarded the Redo branch. The new regression held a
separately allocated matching document and showed that the pruning rule removed
its proof. After withdrawal the proof survives while cached; an idle proof can
still be reclaimed under pressure, and an active registered source reader
keeps its own proof alive. The correction passed 45 app topology-owner tests
with 2 ignored, including this synthetic separate-document fixture, Undo/Redo,
pressure and reader lifetime. Tests used `CARGO_TARGET_DIR=L:/om9-target` and
`-j 2`. The fixture is not a complete history-trimming or desktop test.

Retired entries can remain until pressure reclaims them, and cache lookup is
linear in retained versions. The cache remains optional and can lose a proof
under pressure. This does not establish persistent attestation, source-only
document/Undo ownership, display eviction, bounded native peak or desktop
behavior.
