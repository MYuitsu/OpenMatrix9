# Focused remaining real-ring acceptance. Includes hidden/locked model objects.
MANUAL = 'H:/FreeCAD-src/build/om9-dev/tests/rhino5_modeling_manual.py'
scope = {'__name__': 'manual_helpers', '__file__': MANUAL}
execfile(MANUAL, scope, scope)
scope['run'](['current-ring'], True)
