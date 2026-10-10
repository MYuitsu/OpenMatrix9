# Remaining actual Rhino5 Phase1 types/workflow. Does not close the whole gate.
MANUAL = 'H:/FreeCAD-src/build/om9-dev/tests/rhino5_modeling_manual.py'
scope = {'__name__': 'manual_helpers', '__file__': MANUAL}
execfile(MANUAL, scope, scope)
scope['run'](['source-and-link', 'current-point', 'current-extrusion',
              'current-mesh', 'current-ring'], True)
