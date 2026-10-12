# Shared Rhino5 document enumeration; read-only and IronPython2 compatible.
import Rhino


def document_settings():
    settings = Rhino.DocObjects.ObjectEnumeratorSettings()
    settings.NormalObjects = True
    settings.LockedObjects = True
    settings.HiddenObjects = True
    settings.VisibleFilter = False
    settings.SelectedObjectsFilter = False
    settings.DeletedObjects = False
    settings.ActiveObjects = True
    settings.ReferenceObjects = False
    settings.IdefObjects = False  # True would enumerate ONLY definition members.
    settings.IncludeLights = False
    settings.IncludeGrips = False
    return settings


def document_objects(doc):
    objects = {str(obj.Id): obj for obj in doc.Objects.GetObjectList(document_settings())
               if not obj.IsDeleted}
    for index in range(doc.InstanceDefinitions.Count):
        definition = doc.InstanceDefinitions[index]
        if definition is not None and not definition.IsDeleted:
            for obj in definition.GetObjects():
                objects[str(obj.Id)] = obj
    return list(objects.values())
