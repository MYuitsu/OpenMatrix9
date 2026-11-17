# Display identity without a retained weak header

Prepared remesh buffers now identify their input using a process-local UUID
stored with the B-rep display version. Clones share that identity; a new or
decoded display and a validation edit receive a new one. The value contains
no Arc or Weak and cannot keep a display allocation header alive. It is not
serialized or treated as document equality, topology attestation, or a
replacement for the existing source and fingerprint checks.

The scene fixture rejects both an equal but independently allocated display
and a mutated clone whose old display hash remains unchanged. Core tests
verify the identity has no drop-held allocation, clone identity, different
decode identity, equality after decode and unchanged v4 bytes. The diagnostic
`display_liveness` witness still contains a Weak; a caller retaining it after
the display dies must account for the weak allocation header. Production
remesh buffer receipts use the value identity instead.

Validation: core B-rep 24, scene 34, app topology owner 49 passed/2 ignored,
Sphere/Gumball including v4 2 and GPU offscreen 29 passed. App no-default
check, public-source audit and diff check passed. Builds used
`L:/om9-target` and `-j 2`. This did not measure large-model latency, native
or process peak, or the running desktop. Document and Undo still own the
original display; history source/proof retention and actual eviction remain
open, so M0/M1 are not complete.
