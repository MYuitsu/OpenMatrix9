"""Persistent TextDot fields and derived Coin screen-text display.

The double anchor and strings are authoritative current fields. Coin's float
coordinates/font rasterization are a preview, never the export geometry.
"""
import json
import math

PROPERTIES = dict(point='OM9TextDotPoint', primary_text='OM9TextDotText',
                  secondary_text='OM9TextDotSecondaryText', font_face='OM9TextDotFont',
                  height_in_points='OM9TextDotHeight', bold='OM9TextDotBold',
                  italic='OM9TextDotItalic', always_on_top='OM9TextDotAlwaysOnTop',
                  transparent='OM9TextDotTransparent')

def is_adapter(obj):
    return 'OM9TextDotSchema' in obj.PropertiesList

def fields(obj, record=None):
    if not is_adapter(obj) or obj.OM9TextDotSchema != 1 or not obj.isDerivedFrom('App::FeaturePython'):
        raise RuntimeError('TextDot adapter has no supported current-field schema')
    if record is not None and (record.get('class_name') != 'ON_TextDot' or
                               json.loads(obj.OM9SourceRecord) != record):
        raise RuntimeError('TextDot native source record differs from verified archive')
    result = {key: getattr(obj, prop) for key, prop in PROPERTIES.items()}
    result['point'] = list(result['point'])
    if not all(math.isfinite(v) for v in result['point']):
        raise RuntimeError('Invalid TextDot anchor')
    # Reading/persisting newer secondary text is valid. Target-version
    # compatibility belongs to native export, not the import signature.
    return result

def changed(obj, record, manifest):
    current = fields(obj, record)
    original = dict(record['text_dot'])
    original['point'] = [v * manifest['scale_mm'] for v in original['point']]
    return current != original

def bind(obj, record, scale):
    import FreeCAD as App
    from ThreeDmArchiveState import _property
    data = record['text_dot']
    _property(obj, 'Integer', 'OM9TextDotSchema', 1)
    for key, name in PROPERTIES.items():
        value = data[key]
        kind = 'String'
        if key == 'point':kind, value = 'Vector', App.Vector(*[v*scale for v in value])
        elif key == 'height_in_points':kind = 'Integer'
        elif isinstance(value, bool):kind = 'Bool'
        _property(obj, kind, name, value)
        obj.setEditorMode(name, 0)
    # Preview is reconstructed from persistent fields when the FCStd is read.
    obj.ViewObject.Proxy = ViewProvider(obj.ViewObject)

class ViewProvider:
    def __init__(self, view):self.attach(view)
    def attach(self, view):
        from pivy import coin
        self.Object = view.Object
        self.root = coin.SoSeparator()
        self.translation = coin.SoTranslation()
        self.color = coin.SoBaseColor()
        self.font = coin.SoFont()
        self.text = coin.SoText2()
        self.text.justification = coin.SoText2.CENTER
        self.content = coin.SoSeparator()
        for node in (self.translation, self.color, self.font, self.text):self.content.addChild(node)
        self.top = coin.SoAnnotation();self.top.addChild(self.content)
        view.addDisplayMode(self.root, 'TextDot')
        self.update()
    def update(self):
        if not hasattr(self, 'Object'):return
        obj = self.Object
        try:
            data = fields(obj)
            point = obj.OM9BlockMemberPlacement.multVec(obj.OM9TextDotPoint) if hasattr(obj, 'OM9BlockMemberPlacement') else obj.OM9TextDotPoint
            self.translation.translation.setValue(*point)
            self.text.string.setValues(0, data['primary_text'].split('\n'))
            self.font.name = data['font_face']
            self.font.size = data['height_in_points']
            self.color.rgb.setValue(*list(getattr(obj, 'OM9Color', (0.7, 0.7, 0.7)))[:3])
            self.root.removeAllChildren()
            self.root.addChild(self.top if data['always_on_top'] else self.content)
        except (AttributeError, RuntimeError, ValueError):
            # Invalid current data remains visible in the property editor and
            # fails export; never substitute original source data for it.
            self.root.removeAllChildren()
    def updateData(self, obj, prop):self.update()
    def getDisplayModes(self, obj):return ['TextDot']
    def getDefaultDisplayMode(self):return 'TextDot'
    def setDisplayMode(self, mode):return mode
    def __getstate__(self):return None
    def __setstate__(self, state):pass
