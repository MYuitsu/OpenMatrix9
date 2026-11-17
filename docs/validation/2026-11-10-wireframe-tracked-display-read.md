# Wireframe display read ownership

DupEdge and ExtractWire read one B-rep display across multiple geometry
commits. Previously they cloned its display Arc without retaining the
document owner used by coordinator accounting. They now hold a tracked
immutable document capture for the entire read and borrow its display. The
capture can be refused before any geometry commit when admission fails.

The OCCT box fixture passed for both command names. A simulated-pressure
fixture confirmed that tracked capture is refused and the command leaves the
document snapshot and existing Redo unchanged. The app no-default check,
public-source audit and diff check passed. Builds used `L:/om9-target` and
`-j 2`.

This addresses one reader lifetime; it does not make document or Undo display
reclaimable. These commands still produce sampled curves from display edges,
so their exact source-geometry work remains open. The tests do not establish
native/process peak, desktop behavior, or atomicity of failures after a later
per-edge commit.
