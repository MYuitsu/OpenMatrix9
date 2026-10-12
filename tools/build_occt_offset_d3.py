# SPDX-License-Identifier: LGPL-2.1-or-later
"""Build only patched OCCT TKG3d, importing the existing SDK dependencies.

Run from a VS2022 native-tools environment. The official V8_0_1 archive is
https://codeload.github.com/Open-Cascade-SAS/OCCT/zip/refs/tags/V8_0_1
All source/license notices remain in the isolated output; the shared SDK is
read-only. This is not a full FreeCAD or OCCT build.
"""
import argparse, hashlib, json, subprocess, zipfile
from pathlib import Path

ARCHIVE_SHA='7c033d917ee8f040c0512d289dcc5f02c148889d5bac17c3e25639accb44f0da'

def fingerprint(path):
    raw=path.read_bytes()
    return dict(path=path.as_posix(),bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--archive',type=Path,required=True)
    parser.add_argument('--sdk',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--workers',type=int,default=14)
    args=parser.parse_args()
    assert args.output.is_absolute() and not args.output.exists(),'Preserve existing outputs'
    assert fingerprint(args.archive)['sha256']==ARCHIVE_SHA,'Pinned archive mismatch'
    args.output.mkdir(parents=True)
    source=args.output/'upstream';rows=[];headers=[]
    with zipfile.ZipFile(args.archive)as archive:
        for item in archive.infolist():
            relative=Path(*Path(item.filename).parts[1:])
            if item.is_dir():continue
            if relative.parts[:3]!=('src','ModelingData','TKG3d') and relative.as_posix()not in ('LICENSE_LGPL_21.txt','OCCT_LGPL_EXCEPTION.txt','LICENSE'):continue
            assert not relative.is_absolute() and '..'not in relative.parts
            path=source/relative;path.parent.mkdir(parents=True,exist_ok=True)
            path.write_bytes(archive.read(item));rows.append(fingerprint(path))
            if path.suffix in ('.hxx','.lxx'):
                installed=args.sdk/'include/opencascade'/path.name
                assert installed.exists() and installed.read_bytes()==path.read_bytes(),installed
                headers.append(fingerprint(installed))
    libraries=['TKMath','TKernel','TKG2d']
    unchanged=[fingerprint(args.sdk/group/(name+suffix))for name in libraries+['TKG3d']for group,suffix in [('lib','.lib'),('bin','.dll')]]+headers
    patch=Path(__file__).parent/'patches/occt-8.0.1-offset-d3.patch'
    subprocess.run(['rtk','proxy','git','-C',str(source),'apply','--ignore-space-change',str(patch.resolve())],check=True)
    driver=args.output/'driver';driver.mkdir()
    root=(source/'src/ModelingData/TKG3d').as_posix();sdk=args.sdk.as_posix();output=args.output.as_posix()
    (driver/'CMakeLists.txt').write_text(f'''cmake_minimum_required(VERSION 3.20)
project(OCCTOffsetD3 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(root "{root}")
include("${{root}}/PACKAGES.cmake")
set(sources)
set(includes)
foreach(package IN LISTS OCCT_TKG3d_LIST_OF_PACKAGES)
  include("${{root}}/${{package}}/FILES.cmake")
  list(APPEND includes "${{root}}/${{package}}")
  foreach(file IN LISTS OCCT_${{package}}_FILES)
    if(file MATCHES "[.]cxx$")
      list(APPEND sources "${{root}}/${{package}}/${{file}}")
    endif()
  endforeach()
endforeach()
add_library(TKG3d SHARED ${{sources}})
target_include_directories(TKG3d PRIVATE ${{includes}} "{sdk}/include/opencascade")
target_compile_definitions(TKG3d PRIVATE _CRT_SECURE_NO_WARNINGS)
target_link_libraries(TKG3d PRIVATE "{sdk}/lib/TKMath.lib" "{sdk}/lib/TKernel.lib" "{sdk}/lib/TKG2d.lib")
set_target_properties(TKG3d PROPERTIES RUNTIME_OUTPUT_DIRECTORY "{output}/bin" ARCHIVE_OUTPUT_DIRECTORY "{output}/lib")
''',encoding='utf-8')
    receipt=dict(scope='TKG3d only; imported existing SDK libraries; SDK not modified',archive=fingerprint(args.archive),patch=fingerprint(patch),original_sources=rows,sdk_headers_exact=headers,shared_outputs_before=unchanged)
    for stage,command in [('configure',['cmake','-G','Ninja','-S',str(driver),'-B',str(args.output/'build'),'-DCMAKE_BUILD_TYPE=Release']),('build',['cmake','--build',str(args.output/'build'),'--target','TKG3d','--parallel',str(max(1,args.workers))])]:
        with(args.output/(stage+'.log')).open('w',encoding='utf-8')as log:
            result=subprocess.run(['rtk','proxy']+command,stdout=log,stderr=subprocess.STDOUT)
        receipt[stage+'_exit_code']=result.returncode
        (args.output/'build-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
        if result.returncode:raise SystemExit(result.returncode)
    for row in unchanged:assert fingerprint(Path(row['path']))==row,row['path']
    receipt.update(native_dll=fingerprint(args.output/'bin/TKG3d.dll'),native_import=fingerprint(args.output/'lib/TKG3d.lib'),patched_helper=fingerprint(source/'src/ModelingData/TKG3d/Geom/Geom_OffsetCurveUtils.pxx'),shared_outputs_unchanged=True)
    (args.output/'build-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(dll=receipt['native_dll'],headers=len(headers),build_exit_code=0)))

if __name__=='__main__':main()
