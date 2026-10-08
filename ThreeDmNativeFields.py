"""Dispatch current native fields without changing legacy TextDot signatures."""
def adapter(obj,record=None):
    import ThreeDmTextDot,ThreeDmPointCloud,ThreeDmHatch
    modules=[module for module in (ThreeDmTextDot,ThreeDmPointCloud,ThreeDmHatch) if module.is_adapter(obj)]
    if len(modules)>1:raise RuntimeError('Ambiguous native current-field schema')
    if not modules:return None
    module=modules[0]
    if record is not None:module.fields(obj,record)
    return module

def stage(obj,record,staging,index):
    import ThreeDmTextDot
    module=adapter(obj,record)
    if module is None:raise RuntimeError('Native geometry has no current-field adapter')
    if module is ThreeDmTextDot:return dict(text_dot=module.fields(obj,record))
    return module.stage(obj,record,staging,index)
