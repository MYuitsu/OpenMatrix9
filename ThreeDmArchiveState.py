"""FreeCAD binding for immutable native 3DM snapshots and scoped identities."""
import hashlib
import json
import os
import uuid

MAX_ARCHIVE = 512 * 1024 * 1024
MAX_MANIFEST = 32 * 1024 * 1024

def _hash(path):
    if os.path.getsize(path) > MAX_ARCHIVE:
        raise RuntimeError('Source archive exceeds 512 MiB')
    digest = hashlib.sha256()
    with open(path, 'rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()

def _property(obj, kind, name, value):
    obj.addProperty('App::Property' + kind, name, 'Rhino source')
    setattr(obj, name, value)
    obj.setEditorMode(name, 1)

def archive_identity(obj):
    namespace = getattr(obj, 'OM9ImportNamespace', '')
    source = getattr(obj, 'OM9SourceUUID', '')
    return (namespace, source) if namespace and source else None

def validate_manifest(manifest):
    if not isinstance(manifest, dict):
        raise RuntimeError('Invalid source archive manifest')
    text = json.dumps(manifest, ensure_ascii=False, separators=(',', ':'))
    if manifest.get('schema_version') != 1 or len(text.encode('utf-8')) > MAX_MANIFEST:
        raise RuntimeError('Invalid source archive schema or manifest size')
    digest = manifest.get('archive_sha256', '')
    if len(digest) != 64 or any(c not in '0123456789abcdef' for c in digest):
        raise RuntimeError('Invalid source archive hash')
    seen = set()
    for category in ('records', 'components'):
        rows = manifest.get(category)
        if not isinstance(rows, list):
            raise RuntimeError('Invalid source record table')
        for row in rows:
            if not isinstance(row, dict) or row.get('capability') not in ('editable', 'display-retained', 'retained', 'incompatible'):
                raise RuntimeError('Invalid source record capability')
            value = row.get('source_uuid', '')
            try:
                identity = uuid.UUID(value)
            except (ValueError, TypeError, AttributeError):
                raise RuntimeError('Invalid source UUID') from None
            if not identity.int or str(identity) != value or value in seen:
                raise RuntimeError('Invalid or duplicate source UUID')
            seen.add(value)
    for category in ('resources', 'issues'):
        if not isinstance(manifest.get(category), list):
            raise RuntimeError('Invalid archive report table')
    if not isinstance(manifest.get('settings'), dict):
        raise RuntimeError('Invalid archive settings')
    return text

def validate_prepared(prepared):
    validate_manifest(prepared['manifest'])
    if _hash(prepared['snapshot']) != prepared['manifest']['archive_sha256']:
        raise RuntimeError('Source snapshot integrity check failed')

def load_archive_state(container):
    text = container.OM9ArchiveManifest
    if len(text.encode('utf-8')) > MAX_MANIFEST or container.OM9ArchiveSchema != 1:
        raise RuntimeError('Invalid source archive schema or manifest size')
    manifest = json.loads(text)
    validate_manifest(manifest)
    snapshot = container.OM9SourceArchive
    if manifest.get('schema_version') != 1 or _hash(snapshot) != container.OM9ArchiveHash or manifest.get('archive_sha256') != container.OM9ArchiveHash:
        raise RuntimeError('Source archive integrity check failed')
    return dict(manifest=manifest, snapshot=snapshot)

def bind_archive(document, prepared):
    import ThreeDm
    manifest = prepared['manifest']
    text = validate_manifest(manifest)
    validate_prepared(prepared)
    namespace = str(uuid.uuid4())
    container = document.addObject('App::FeaturePython', 'RhinoSourceArchive')
    container.Label = 'Rhino source archive (storage only)'
    _property(container, 'Integer', 'OM9ArchiveSchema', 1)
    _property(container, 'String', 'OM9ImportNamespace', namespace)
    _property(container, 'FileIncluded', 'OM9SourceArchive', prepared['snapshot'])
    _property(container, 'String', 'OM9ArchiveManifest', text)
    _property(container, 'String', 'OM9ArchiveHash', manifest['archive_sha256'])
    _property(container, 'Integer', 'OM9ArchiveMode', 1)
    objects = ThreeDm._insert_prepared(document.Name, prepared['host_geometry'])
    pairs = list(zip(objects, (item for item, geometry in prepared['host_geometry'])))
    for row in prepared['retained_records']:
        obj = document.addObject('App::FeaturePython', 'RhinoRetainedRecord')
        obj.Label = (row['name'] or row['class_name']) + ' [source retained]'
        objects.append(obj)
        pairs.append((obj, row))
    for obj, row in pairs:
        _property(obj, 'String', 'OM9SourceUUID', row['source_uuid'])
        _property(obj, 'String', 'OM9ImportNamespace', namespace)
        _property(obj, 'String', 'OM9SourceClass', row['class_name'])
        _property(obj, 'String', 'OM9Capability', row['capability'])
        signature = hashlib.sha256(json.dumps(row, sort_keys=True).encode()).hexdigest()
        _property(obj, 'String', 'OM9SourceSignature', signature)
    document.recompute()
    return objects
