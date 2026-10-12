"""Validate an explicit paired isolated App override for the shared SDK.

Source/output hashes are candidate evidence, not a complete compiler/ABI or
production-deployment certificate.
"""
from pathlib import Path
import argparse,hashlib,json,sys
def validate(path,sdk,source):
 path=Path(path);sdk=Path(sdk).resolve();source=Path(source).resolve()
 if path.stat().st_size>16*1024*1024:raise ValueError('Native App manifest exceeds byte budget')
 value=json.loads(path.read_text(encoding='utf-8-sig'))
 if value.get('schema_version')!=1 or Path(value['sdk_root']).resolve()!=sdk or Path(value['source_root']).resolve()!=source:raise ValueError('Native App manifest belongs to another SDK/source')
 def file(row):
  item=Path(row['path']).resolve()
  if not item.is_file()or item.stat().st_size!=row['bytes']or len(row['sha256'])!=64:raise ValueError('Invalid native App file row')
  if hashlib.sha256(item.read_bytes()).hexdigest()!=row['sha256']:raise ValueError('Stale native App file: '+str(item))
  return item
 dll=file(value['native_dll']);lib=file(value['native_import'])
 if dll.name!='FreeCADApp.dll'or lib.name!='FreeCADApp.lib'or dll.parent!=lib.parent:raise ValueError('Native App DLL/import pair must share its isolated directory')
 rows=value['sources']
 if not isinstance(rows,list)or not rows or len(rows)>4096:raise ValueError('Invalid source candidate budget')
 seen=set()
 for row in rows:
  item=file(row);item.relative_to(source/'src/App')
  if item in seen:raise ValueError('Duplicate native source candidate')
  seen.add(item)
 for relative in ['Document.h','Transactions.h','private/DocumentP.h','private/PropertyReplay.h']:
  if (source/'src/App'/relative).resolve()not in seen:raise ValueError('Native replay ABI header missing from candidate evidence')
 receipt=file(value['build_receipt'])
 if receipt.parent!=dll.parent or json.loads(receipt.read_text(encoding='utf-8-sig'))['exit_code']!=0:raise ValueError('Native App build has not completed successfully')
 return dict(native_dll=dll.as_posix(),native_import=lib.as_posix(),manifest=path.resolve().as_posix(),source_candidates=len(rows),compiler_dependency_certificate=False)
def main():
 p=argparse.ArgumentParser();p.add_argument('manifest');p.add_argument('--sdk',required=True);p.add_argument('--source',required=True);args=p.parse_args()
 try:print(json.dumps(validate(args.manifest,args.sdk,args.source)))
 except (ValueError,KeyError,OSError,TypeError)as error:print(str(error),file=sys.stderr);return 1
 return 0
if __name__=='__main__':sys.exit(main())
