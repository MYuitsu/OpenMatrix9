"""Authored structural 3DM dependency probe; no target feature completion."""
import argparse
import json
import math
import re
from pathlib import Path
import subprocess
import tempfile
import xml.etree.ElementTree as ET

from validate_standalone_foundation import ROOT, pe_imports, sha256


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"build/standalone-native/Release/om9-opennurbs-probe.exe")
    parser.add_argument("--runner",type=Path,default=ROOT/"standalone/target/release/om9-native-runner.exe")
    args=parser.parse_args()
    binary=args.binary.resolve(strict=True)
    runner=args.runner.resolve(strict=True)
    native_runs=[]
    def run_native(arguments,success=True):
        result=subprocess.run([str(runner),"--program",str(binary),"--sha256",sha256(binary),"--",*map(str,arguments)],capture_output=True,timeout=40)
        report=json.loads(result.stdout)
        assert report["stats"]["exited"] and report["stats"]["active_processes"]==0,"Native job not retired"
        native_runs.append(report)
        if success:
            if result.returncode or report["status"]!="native_owner_result":
                raise AssertionError(report)
            assert report["stats"]["peak_committed_bytes"]<=report["limits"]["native_committed_bytes"]
            return json.loads(report["stdout"])
        assert result.returncode and report["status"]=="native_owner_rejected","Invalid native result claimed success"
        return report
    with tempfile.TemporaryDirectory(prefix="om9-on-probe-") as temporary:
        model=Path(temporary)/"authored.3dm"
        written=run_native(["--write-fixture",model])
        def inspect():
            return run_native(["--inspect",model])
        data=inspect()
        assert data==written,"Cold process changed native inventory"
        assert data["archive_version"]==50
        objects={o["name"]:o for o in data["objects"]}
        assert set(objects)=={"Point","Line","RationalArc","Mesh","Cloud"}
        assert objects["Point"]["point"]==[1,2,3]
        assert objects["Line"]["start"]==[0,0,0] and objects["Line"]["end"]==[12,6,0]
        arc=objects["RationalArc"]
        assert arc["degree"]==2 and arc["rational"] and arc["cv_count"]==3
        assert arc["domain"]==[2,4]
        for value in arc["midpoint"][:2]:
            assert abs(value-10/math.sqrt(2))<1e-10
        assert arc["midpoint"][2]==0
        assert objects["Mesh"]["faces"]==[[0,1,2,2]]
        assert objects["Cloud"]["points"]==[[2,3,4],[5,6,7]]
        assert objects["Cloud"]["colors"]==[[255,0,0],[0,255,0]]
        before=model.read_bytes()
        model.write_bytes(b"corrupt")
        bad=run_native(["--inspect",model],success=False)
        assert "NativeExit" in bad["reason"],"Corrupt archive failure reason mismatch"
        model.write_bytes(before)
        imports=pe_imports(binary)
        forbidden=("freecad","qymcad","qt5","qt6","coin","python","tkernel","tkbrep")
        assert not any(any(x in n.lower() for x in forbidden) for n in imports)
        source=ROOT/"build/standalone-deps/opennurbs"
        commit=subprocess.check_output(["git","-C",str(source),"rev-parse","HEAD"],text=True).strip()
        assert commit=="eb92af3ba1806b0a34a99aba0d3bda83e3d46083"
        assert subprocess.run(["git","-C",str(source),"diff","--quiet","HEAD"],capture_output=True).returncode==0
        raw_license=subprocess.check_output(["git","-C",str(source),"show","HEAD:LICENSE"])
        import hashlib
        cmake_cache=(ROOT/"build/standalone-native/CMakeCache.txt").read_text(encoding="utf-8")
        compiler_file=next((ROOT/"build/standalone-native/CMakeFiles").glob("*/CMakeCXXCompiler.cmake"))
        compiler_text=compiler_file.read_text(encoding="utf-8")
        compiler_version=re.search(r'set\(CMAKE_CXX_COMPILER_VERSION "([^"]+)"\)',compiler_text).group(1)
        compiler_path=re.search(r'set\(CMAKE_CXX_COMPILER "([^"]+)"\)',compiler_text).group(1)
        project=ET.parse(ROOT/"build/standalone-native/om9-opennurbs-probe.vcxproj")
        sdk=project.find(".//{*}WindowsTargetPlatformVersion").text
        receipt={"scope":"authored_native_dependency_probe_only","status":"validated_probe_not_integrated",
                 "binary_sha256":sha256(binary),"fixture_sha256":sha256(model),"inventory":data,"pe_imports":imports,
                 "native_owner":{"runner_sha256":sha256(runner),"runs":native_runs,"policy":"suspended before job assignment, one process, explicit stdio-only inheritance, committed memory/output caps, cancellation/deadline, exclusive nonblocking pipe pumping and one teardown deadline for exit/EOF/observed job accounting", "sources":[{"path":p.relative_to(ROOT).as_posix(),"sha256":sha256(p)} for p in sorted((ROOT/"standalone/crates/om9-native-owner").rglob("*")) if p.is_file()]},
                 "source":{"repository":"https://github.com/mcneel/opennurbs","commit":commit,"working_tree_clean":True,
                           "license_git_blob_sha256":hashlib.sha256(raw_license).hexdigest(),"license_worktree_sha256":sha256(source/"LICENSE"),
                           "license_text_matches_existing_notice":(source/"LICENSE").read_text()==(ROOT/"Resources/licenses/openNURBS.txt").read_text(),
                           "key_files":[{"path":p.relative_to(source).as_posix(),"sha256":sha256(p)} for p in [source/"CMakeLists.txt",source/"opennurbs_extensions.h",source/"opennurbs_pointcloud.h"]]},
                 "build":{"generator":"Visual Studio 17 2022","compiler":"MSVC "+compiler_version,"compiler_path":compiler_path,"sdk":sdk,"architecture":"x64","configuration":"Release","runtime":"MD dynamic MSVC CRT",
                          "targets":["om9-opennurbs-probe","opennurbsStatic","zlib"],"libraries":["Shlwapi","Usp10","Windows/MSVC OS dependencies in actual PE imports"],
                          "cmake_cache_sha256":hashlib.sha256(cmake_cache.encode()).hexdigest(),
                          "authored_sources":[{"path":p.relative_to(ROOT).as_posix(),"sha256":sha256(p)} for p in [ROOT/"standalone/native/CMakeLists.txt",ROOT/"standalone/native/opennurbs_probe.cpp",Path(__file__).resolve()]]},
                 "limitations":["Not an OM9 import/export command or native memory-safety/global aggregate-budget gate", "Retained/edited/deleted merge, blocks and full family/field matrix pending", "No Rhino 5 open/edit/runtime acceptance", "Cloud per-point hidden flags are runtime-only in openNURBS and not serialized natively"]}
        output=ROOT/"docs/validation/2026-10-13-standalone-opennurbs-probe.json"
        output.write_text(json.dumps(receipt,indent=2)+"\n",encoding="utf-8")
        print(json.dumps({"status":receipt["status"],"objects":len(objects),"archive_version":data["archive_version"],"cold_equal":True,"corrupt_rejected":True,"native_peak_committed_bytes":[r["stats"]["peak_committed_bytes"] for r in native_runs],"active_processes":[r["stats"]["active_processes"] for r in native_runs],"imports":imports}))


if __name__=="__main__":
    main()
