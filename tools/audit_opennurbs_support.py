"""Current coverage/gap audit; preserves the historical audit JSON."""
from pathlib import Path
from datetime import datetime
from zoneinfo import ZoneInfo
import argparse
import collections
import json
from opennurbs_coverage import normalize, sha

def main():
    project=Path(__file__).resolve().parents[1]
    parser=argparse.ArgumentParser()
    parser.add_argument('--sdk-root',type=Path,default=project.parent/'dependencies/opennurbs')
    parser.add_argument('--runtime',type=Path,default=project.parent/'3dm-preserve-native/coverage-runtime.json')
    parser.add_argument('--output',type=Path,default=project/'docs/3dm-current-support-audit.json')
    args=parser.parse_args()
    coverage_path=project/'docs/3dm-coverage.json'
    coverage=json.loads(coverage_path.read_text(encoding='utf-8'))
    result=normalize(project,args.sdk_root,coverage,json.loads(args.runtime.read_text(encoding='utf-8')))
    anchors=[
        ('Gui/ThreeDmSolidShells.cpp','Inward multi-shell cavity has no verified valid editable OCCT solid'),
        ('Gui/ThreeDmNativeReferences.cpp','not certified global correspondence'),
        ('Gui/ThreeDmCurveOnSurface.cpp','export profile is not yet verified'),
        ('Gui/ThreeDmCurveOnSurface.cpp','Rhino 5 discards CurveOnSurface objects containing m_c3'),
        ('Gui/ThreeDmArchive.cpp','Rhino 5 would reverse an inward solid'),
        ('Gui/ThreeDmArchive.cpp','external linked blocks require embedded geometry'),
        ('Gui/ThreeDmMerge.cpp','Object rendering/mapping dependencies are not implemented yet'),
        ('Gui/ThreeDmMerge.cpp','RDK content dependencies are not implemented yet'),
        ('Gui/ThreeDmMerge.cpp','Different source document metadata requires an explicit document merge policy'),
        ('Gui/ThreeDmMerge.cpp','View unit normalization requires verified viewport handling'),
        ('Gui/ThreeDmMerge.cpp','Copied linked definition resource semantics are not implemented yet'),
        ('Gui/ThreeDmMerge.cpp','Unsafe opaque userdata dependency')]
    gaps=[]
    for name,needle in anchors:
        file=project/name
        hits=[i+1 for i,line in enumerate(file.read_text(encoding='utf-8').splitlines()) if needle in line]
        if not hits: raise ValueError('Guard changed; audit required: '+needle)
        gaps.append(dict(path=name,sha256=sha(file),lines=hits,needle=needle))
    result.update(recorded_at=datetime.now(ZoneInfo('Asia/Ho_Chi_Minh')).isoformat(timespec='seconds'),
                  scope='Source and linked registry normalization; field exchange gaps remain explicit.',
                  historical_coverage_sha256=sha(coverage_path),runtime_report_sha256=sha(args.runtime),
                  code_gaps=gaps,classification_counts=dict(collections.Counter(r['classification'] for r in result['classes'])))
    args.output.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(source_classes=len(result['classes']),runtime_classes=result['runtime_count'],
                          excluded_comments=len(result['excluded_comment_registrations']),
                          classification=result['classification_counts'],full_support_proven=False)))
if __name__=='__main__':
    main()
