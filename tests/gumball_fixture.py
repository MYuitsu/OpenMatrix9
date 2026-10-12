"""Qt events for native Gumball fixtures; never writes transform output."""
import FreeCADGui as Gui
from PySide6 import QtCore, QtWidgets, QtTest, QtOpenGLWidgets


class GumballUI:
    def __init__(self, native):
        self.native = native
        self.gesture_viewport = None
        self.main_window = Gui.getMainWindow()
        # Native Qt widgets can outlive transient PySide ancestor wrappers.
        # Keep the complete lookup chain alive without caching a view lookup.
        self._viewport_refs = ()

    def settle(self):
        QtWidgets.QApplication.processEvents()
        QtTest.QTest.qWait(100)

    @property
    def viewport(self):
        area=self.main_window.findChild(QtWidgets.QMdiArea)
        sub=area.activeSubWindow()
        if sub is None:
            import FreeCAD as App
            state=self.native.gumballState()
            sub=next(s for s in area.subWindowList() if s.widget().property('om9ViewSlot')==state['source_slot'] and s.widget().property('om9DocumentName')==App.ActiveDocument.Name)
        container=sub.widget()
        viewport=next(w for w in container.findChildren(QtOpenGLWidgets.QOpenGLWidget)
                      if w.property('om9GumballViewport'))
        self._viewport_refs=(area,sub,container,viewport)
        return viewport

    def command(self, text):
        console = self.main_window.findChild(QtWidgets.QPlainTextEdit, 'OM9CommandTranscript')
        console.setFocus()
        QtTest.QTest.keyClicks(console, text)
        QtTest.QTest.keyClick(console, QtCore.Qt.Key_Return)
        self.settle()

    def select(self, *objects):
        Gui.Selection.clearSelection()
        for obj in objects:
            Gui.Selection.addSelection(obj)
        Gui.activeDocument().activeView().fitAll()
        self.settle()
        self.native.gumballState()
        self.settle()

    def target(self, name):
        self.native.gumballState()
        self.settle()
        rows = self.native.gumballHandles()
        row = next((h for h in rows if h['name'] == name and h['hit_verified']),None)
        if row is None:
            import os
            from pathlib import Path
            from pivy import coin
            output=coin.SoOutput();output.openFile(str(Path(os.environ['OM9_SMOKE_OUTPUT'])/'missing-handle.iv'))
            action=coin.SoWriteAction(output);action.apply(Gui.activeDocument().activeView().getSceneGraph());output.closeFile()
            raise AssertionError({'missing_handle':name,'state':self.native.gumballState(),'handles':rows,'viewport':[self.viewport.width(),self.viewport.height(),self.viewport.devicePixelRatioF()],'camera':Gui.activeDocument().activeView().getCamera()})
        return QtCore.QPoint(row['x'], row['y'])

    def numeric(self, name, value, modifiers=QtCore.Qt.NoModifier):
        pos = self.target(name)
        viewport=self.viewport
        viewport.setFocus()
        QtTest.QTest.mouseClick(viewport, QtCore.Qt.LeftButton, modifiers, pos)
        self.settle()
        field = next(w for w in self.main_window.findChildren(QtWidgets.QLineEdit, 'OM9GumballNumeric')
                     if w.isVisible())
        QtTest.QTest.keyClicks(field, value)
        QtTest.QTest.keyClick(field, QtCore.Qt.Key_Return)
        self.settle()

    def drag(self, name, modifiers=QtCore.Qt.NoModifier):
        pos = self.target(name)
        self.gesture_viewport = self.viewport
        self.gesture_viewport.setFocus()
        QtTest.QTest.mousePress(self.gesture_viewport, QtCore.Qt.LeftButton, modifiers, pos)
        end = pos + QtCore.QPoint(18, 12)
        QtTest.QTest.mouseMove(self.gesture_viewport, end)
        self.settle()
        assert self.native.gumballState()['phase'] == 3
        return end

    def release(self, pos, modifiers=QtCore.Qt.NoModifier):
        target = self.gesture_viewport or self.viewport
        self.gesture_viewport = None
        QtTest.QTest.mouseRelease(target, QtCore.Qt.LeftButton, modifiers, pos)
        self.settle()

    def escape(self):
        QtTest.QTest.keyClick(self.viewport, QtCore.Qt.Key_Escape)
        self.settle()

    def transcript(self):
        return self.main_window.findChild(QtWidgets.QPlainTextEdit, 'OM9CommandTranscript').toPlainText()


def durable_facts(obj):
    """Independent native model observations, independent of the Rust plan."""
    import Part
    facts = {'name': obj.Name, 'type': obj.TypeId, 'label': obj.Label,
             'members': [o.Name for o in getattr(obj, 'Group', [])],
             'source': getattr(getattr(obj, 'LinkedObject', None), 'Name', None),
             'copy_id': getattr(obj, 'OM9GumballIdentity', None),
             'source_uuid': getattr(obj, 'OM9SourceUUID', None),
             'layer': getattr(obj, 'OM9OutputLayerIndex', 0),
             'visible': obj.ViewObject.Visibility,
             'selectable': getattr(obj.ViewObject, 'Selectable', True)}
    if 'Placement' in obj.PropertiesList:
        facts['placement'] = [round(v, 9) for v in obj.Placement.toMatrix().A]
    if hasattr(obj.ViewObject, 'ShapeColor'):
        facts['color'] = [round(v, 6) for v in obj.ViewObject.ShapeColor[:3]]
    if hasattr(obj, 'Shape'):
        shape = Part.getShape(obj)
        b = shape.optimalBoundingBox(False,False)
        facts['geometry'] = [round(v, 9) for v in (b.XMin,b.YMin,b.ZMin,b.XMax,b.YMax,b.ZMax,shape.Volume,shape.Area)]
        facts['topology'] = [len(shape.Vertexes),len(shape.Edges),len(shape.Faces),len(shape.Solids)]
        facts['curve_type'] = type(shape.Edges[0].Curve).__name__ if shape.Edges else None
        facts['valid'] = shape.isValid()
    if hasattr(obj, 'OM9GumballReport'):
        import json
        facts['provenance'] = json.loads(obj.OM9GumballReport)
    return facts
