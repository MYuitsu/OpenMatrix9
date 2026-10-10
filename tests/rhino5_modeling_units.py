# Focused actual Rhino5 mm/cm diagnostics; opens only an empty unsaved document.
MANUAL = 'H:/FreeCAD-src/build/om9-dev/tests/rhino5_modeling_manual.py'
scope = {'__name__': 'manual_helpers', '__file__': MANUAL}
execfile(MANUAL, scope, scope)
scope['run'](['mm', 'cm-converted-mm'])
