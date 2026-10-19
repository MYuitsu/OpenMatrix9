"""Actual native schema3 source projection, always through the owned runner.

Authored archives and resolution packets stay private. This validates producer
projection, not host admission, native merge, or Rhino application acceptance.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import tempfile
from validate_native_source_inventory import ROOT, PIN, digest, uid, build_fixture, verify_circle_oracle


def close(actual, expected):
    if isinstance(expected, list):
        assert len(actual) == len(expected)
        for a,e in zip(actual,expected):
            close(a,e)
    elif isinstance(expected,(int,float)):
        assert math.isclose(actual,expected,rel_tol=2e-13,abs_tol=2e-13),(actual,expected)
    else:
        assert actual == expected,(actual,expected)


def resolution(model=None,page=None,definitions=None):
    return {"schema_version":1,"model_mm_per_unit":model,"page_mm_per_unit":page,
            "definition_mm_per_unit":definitions or {}}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--build-fixture",action="store_true")
    parser.add_argument("--binary",type=Path,default=ROOT/"build/standalone-native/Release/om9-exchange-native.exe")
    parser.add_argument("--fixture-binary",type=Path,default=ROOT/"build/native-source-fixture/Release/om9-source-fixture.exe")
    parser.add_argument("--runner",type=Path,default=ROOT/"target/release/om9-native-runner.exe")
    parser.add_argument("--receipt",type=Path,default=ROOT/"build/standalone-native/native-source-projection-validation.json")
    args=parser.parse_args()
    if args.build_fixture: build_fixture()
    binary=args.binary.resolve(strict=True)
    fixture=args.fixture_binary.resolve(strict=True)
    runner=args.runner.resolve(strict=True)
    runs=[]

    def run(program,arguments,success=True):
        process=subprocess.run(["rtk","proxy",str(runner),"--program",str(program),"--sha256",digest(program),"--",*map(str,arguments)],capture_output=True,timeout=40)
        report=json.loads(process.stdout)
        assert report["stats"]["exited"] and report["stats"]["active_processes"] == 0,report
        report["program_sha256"]=digest(program)
        report["arguments"]=[Path(str(a)).name if isinstance(a,Path) else str(a) for a in arguments]
        report["stdout_sha256"]=hashlib.sha256(report.get("stdout","").encode()).hexdigest()
        runs.append(report)
        if success:
            assert process.returncode == 0 and report["status"] == "native_owner_result",report
            return json.loads(report["stdout"])
        assert process.returncode != 0 and report["status"] == "native_owner_rejected",report
        assert not report.get("stdout"),report
        return report

    projections={}
    with tempfile.TemporaryDirectory(prefix="om9-native-projection-") as directory:
        directory=Path(directory)
        archive=directory/"authored.3dm"
        run(fixture,["mm",archive])
        packet=directory/"resolution.json"
        packet.write_text(json.dumps(resolution()),encoding="utf-8")
        projected=run(binary,["--project-source",archive,packet])
        assert projected["schema_version"] == 3 and projected["status"] == "source_projected"
        assert projected["decoder_policy"] == "om9-source-projection-v1"
        assert projected["archive"]["source_sha256"] == digest(archive)
        assert projected["unit_resolution"]["model"] == {"millimeters_per_unit":1,"resolved_by":"declared","original_native_enum":2}
        assert projected["admission_required"] is True
        records={o["id"]:o for o in projected["objects"]}
        assert records[uid(10)]["preservation"]["nondefault_fields"]["groups"] == [0]
        assert records[uid(10)]["preservation"]["nondefault_fields"]["material_source"] == 1
        assert records[uid(10)]["preservation"]["nondefault_fields"]["native_material_index"] == 0
        assert records[uid(22)]["geometry"] is None and records[uid(20)]["native_kind"] == "ON_Brep"
        projections["complex-mm"]=projected
        type_ids={"Image":1,"TextureMapping":2,"RenderMaterial":3,"LinePattern":4,"Layer":5,"Group":6,
                  "TextStyle":7,"DimStyle":8,"RenderLight":9,"HatchPattern":10,"InstanceDefinition":11,"ModelGeometry":12,"HistoryRecord":13}
        for mode,factor in [("mm",1),("inch",25.4),("custom",2.5),("unitless",25.4),("unset",25.4)]:
            path=directory/f"clean-{mode}.3dm"
            run(fixture,[f"clean-{mode}",path])
            original=run(binary,["--inventory-source",path])
            unknown=mode in {"unitless","unset"}
            requested=resolution(factor if unknown else None,7 if unknown else None,
                {uid(n):factor for n in [30,31,32]} if unknown else None)
            packet.write_text(json.dumps(requested),encoding="utf-8")
            data=run(binary,["--project-source",path,packet])
            assert data["archive"]["source_sha256"] == digest(path)
            assert data["units"] == original["units"],"source metadata must stay native truth"
            assert data["complete_read"] is True and data["admission_required"] is True
            assert len(data["objects"]) == 15 and len(data["definitions"]) == 3 and len(data["layers"]) == 1
            objects={o["id"]:o for o in data["objects"]}
            assert set(objects) == {uid(n) for n in [10,11,12,13,14,15,16,17,18,19,27,29,40,41,43]}
            assert sum(o["is_definition_member"] for o in objects.values()) == 3
            assert all(o["capability"] == "geometry_projection_exact" and o["admission_required"] and o["native_component_type"] == 12 for o in objects.values())
            for o in objects.values():
                assert o["preservation"]["admission_required"] is True
                assert o["preservation"]["nondefault_fields"] == {},o
                defaults=o["preservation"]["default_fields"]
                assert defaults["groups"] == [] and defaults["native_material_index"] == -1
                assert defaults["opaque_native_extensions"] == {"sdk_default_comparison_equal":True}
            close(objects[uid(10)]["geometry"]["Point"],[25.4,50.8,0])
            close(objects[uid(11)]["geometry"]["Curve"]["Line"]["end"],[3,4,0])
            curve=objects[uid(13)]["geometry"]["Curve"]["Nurbs"]
            close(curve["poles"],[[10,0,0],[10,10,0],[0,10,0]])
            close(curve["weights"],[1,math.sqrt(.5),1])
            assert curve["domain"] == [2,4] and curve["knots"] == [2,2,2,4,4,4]
            surface=objects[uid(14)]["geometry"]["Surface"]
            close(surface["poles"],[[0,0,0],[0,1,1],[0,2,2],[5,0,1],[5,1,2],[5,2,3]])
            assert surface["size"] == [2,3] and surface["weights"] == [1,2,3,2,3,4]
            mesh=objects[uid(15)]["geometry"]["Mesh"]
            assert mesh["faces"] == [[0,1,2,3]] and mesh["colors"] == [[50,100,150,210]]*4
            assert all(math.isclose(a,e,rel_tol=1e-6,abs_tol=1e-7) for p,q in zip(mesh["vertices"],[[0,0,0],[2,0,0],[2,2,0],[0,2,0]]) for a,e in zip(p,q))
            close(objects[uid(16)]["geometry"]["Cloud"]["points"],[[2,3,4],[5,6,7]])
            assert len(objects[uid(17)]["geometry"]["Curve"]["Polycurve"]["segments"]) == 2
            close(objects[uid(40)]["geometry"]["Point"],[1,2,3])
            close(objects[uid(18)]["placement"],[[-2,0,0,10],[0,3,0,0],[0,0,4,0],[0,0,0,1]])
            close(objects[uid(41)]["placement"],[[1,0,0,0],[0,1,0,5],[0,0,1,0],[0,0,0,1]])
            for component in data["components"]:
                assert component["native_component_type"] == type_ids[component["component_type"]]
            assert data["layers"][0]["native_component_type"] == 5 and data["layers"][0]["preservation"]["nondefault_fields"] == {},data["layers"][0]
            assert all(d["native_component_type"] == 11 and d["dependency_readiness"]["host_ready"] is False for d in data["definitions"])
            assert all(d["preservation"]["nondefault_fields"] == {} and d["preservation"]["default_fields"]["native_definition_type"] == 1 for d in data["definitions"])
            assert data["unit_resolution"]["model"]["resolved_by"] == ("explicit" if unknown else "declared")
            assert data["unit_resolution"]["model"]["original_native_enum"] == original["units"]["model"]["native_enum"]
            assert run(binary,["--project-source",path,packet]) == data,"cold output must be deterministic"
            if not unknown:
                packet.write_text(json.dumps(resolution(factor,data["unit_resolution"]["page"]["millimeters_per_unit"],{uid(n):factor for n in [30,31,32]})),encoding="utf-8")
                assert run(binary,["--project-source",path,packet]) == data,"matching declared overrides must not rescale"
            native_circle={"center":[2/factor,3/factor,4/factor],"x":[0,1,0],"y":[0,0,1],"radius":6/factor,"domain":[11,17],"angle_domain":[1,1+math.tau]}
            verify_circle_oracle(run(fixture,["--oracle-circle",path,"29"]),native_circle)
            projections[f"clean-{mode}"]=data
            if unknown:
                packet.write_text(json.dumps(resolution(factor,7)),encoding="utf-8")
                pending=run(binary,["--project-source",path,packet])
                pending_objects={o["id"]:o for o in pending["objects"]}
                assert pending_objects[uid(10)]["capability"] == "geometry_projection_exact"
                assert all(v is None for v in pending["unit_resolution"]["definitions"].values())
                assert all(pending_objects[uid(n)]["geometry"] is None for n in [18,19,27,40,41,43])
        for mode in ["clean-mixed","clean-unresolved","attribute-userdata"]:
            path=directory/f"{mode}.3dm"
            run(fixture,[mode,path])
            packet.write_text(json.dumps(resolution()),encoding="utf-8")
            data=run(binary,["--project-source",path,packet])
            objects={o["id"]:o for o in data["objects"]}
            if mode == "clean-mixed":
                assert data["unit_resolution"]["definitions"][uid(30)]["millimeters_per_unit"] == 25.4
                assert data["unit_resolution"]["definitions"][uid(32)]["millimeters_per_unit"] == 2.5
                close(objects[uid(40)]["geometry"]["Point"],[1,2,3])
                close(objects[uid(43)]["geometry"]["Point"],[1,2,3])
                close(objects[uid(19)]["placement"],[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]])
                close(objects[uid(41)]["placement"],[[1,0,0,0],[0,1,0,5],[0,0,1,0],[0,0,0,1]])
                assert objects[uid(18)]["dependency_readiness"]["unit_policy_definitions"] == []
            elif mode == "clean-unresolved":
                assert data["unit_resolution"]["definitions"][uid(33)] is None
                assert objects[uid(28)]["geometry"] is None and objects[uid(44)]["geometry"] is None
                assert objects[uid(18)]["capability"] == objects[uid(27)]["capability"] == "geometry_projection_exact"
                packet.write_text(json.dumps(resolution(definitions={uid(33):10})),encoding="utf-8")
                resolved=run(binary,["--project-source",path,packet])
                resolved_objects={o["id"]:o for o in resolved["objects"]}
                close(resolved_objects[uid(44)]["geometry"]["Point"],[10,20,30])
                close(resolved_objects[uid(28)]["placement"],[[.1,0,0,11],[0,.1,0,0],[0,0,.1,0],[0,0,0,1]])
            else:
                assert objects[uid(11)]["geometry"] is not None
                assert objects[uid(11)]["preservation"]["nondefault_fields"]["userdata"][0]["unknown"] is True
            projections[mode]=data
        # These selectors have exact native geometry but require explicit host
        # admission or a named attribute loss. They must not look like the
        # represented layer/object color selectors after cold decoding.
        for mode,selector in [("clean-color-parent",3),("clean-color-material",2)]:
            path=directory/f"{mode}.3dm"
            run(fixture,[mode,path])
            packet.write_text(json.dumps(resolution()),encoding="utf-8")
            data=run(binary,["--project-source",path,packet])
            assert data["archive"]["source_sha256"] == digest(path)
            objects={o["id"]:o for o in data["objects"]}
            point=objects[uid(10)]
            close(point["geometry"]["Point"],[25.4,50.8,0])
            assert point["capability"] == "geometry_projection_exact" and point["admission_required"]
            assert point["attributes"]["color_source"] == selector and point["attributes"]["color"] is None
            fields=point["preservation"]["nondefault_fields"]
            assert fields == {"color_source":selector,"opaque_native_extensions":{"sdk_default_comparison_equal":False}},fields
            assert point["preservation"]["default_fields"]["native_color"] == [0,0,0,255]
            assert all(o["preservation"]["nondefault_fields"] == {} for id_,o in objects.items() if id_ != uid(10))
            assert all(d["dependency_readiness"]["host_ready"] is False for d in data["definitions"])
            projections[mode]=data
        # Strict packet parsing and policy rejection precede admission.
        for name,values in [
            ("unknown-field",dict(resolution(),other=1)),("bad-schema",dict(resolution(),schema_version=2)),
            ("zero",resolution(0)),("negative",resolution(-1)),("conflict",resolution(25.4)),
            ("page-conflict",resolution(page=1)),("wrong-type",resolution(True)),
            ("missing-definition",resolution(definitions={uid(99):1})),
            ("bad-definition-uuid",resolution(definitions={"not-a-uuid":1})),
            ("zero-definition",resolution(definitions={uid(30):0})),
            ("null-definition",resolution(definitions={uid(30):None})),
            ("noncanonical-definition-uuid",resolution(definitions={uid(30).upper():1})),
            ("conflicting-definition",resolution(definitions={uid(30):25.4})),
        ]:
            packet.write_text(json.dumps(values),encoding="utf-8")
            run(binary,["--project-source",archive,packet],False)
        for raw in ['{"schema_version":1,"schema_version":1,"model_mm_per_unit":null,"page_mm_per_unit":null,"definition_mm_per_unit":{}}',json.dumps(resolution())+' trailing',json.dumps(resolution()).replace('"model_mm_per_unit": null','"model_mm_per_unit": 1e400')]:
            packet.write_text(raw,encoding="utf-8")
            run(binary,["--project-source",archive,packet],False)
        unknown=directory/"clean-unitless.3dm"
        for values in [resolution(),resolution(25.4),resolution(page=7),resolution(1e308,7,{uid(n):25.4 for n in [30,31,32]}),resolution(1e-308,7,{uid(n):25.4 for n in [30,31,32]})]:
            packet.write_text(json.dumps(values),encoding="utf-8")
            run(binary,["--project-source",unknown,packet],False)
        packet.write_text(json.dumps(resolution()),encoding="utf-8")
        for mode in ["unknown-class","unknown-table","duplicate-id","missing-dependency","definition-cycle","invalid-tolerance","polycurve-point-budget"]:
            path=directory/f"bad-{mode}.3dm"
            run(fixture,[mode,path])
            rejected=run(binary,["--project-source",path,packet],False)
            assert "conflicting declared unit override" not in rejected["stderr"],rejected
        packet.write_text(json.dumps(resolution()),encoding="utf-8")
        for name,bytes_ in [("truncated",archive.read_bytes()[:-30]),("trailing",archive.read_bytes()+b"unowned bytes"),("not-native",b"not a 3dm archive")]:
            path=directory/f"{name}.3dm"; path.write_bytes(bytes_)
            run(binary,["--project-source",path,packet],False)
        oversized=directory/"oversized.3dm"
        with oversized.open("wb") as file:
            file.seek(64*1024*1024); file.write(b"x")
        rejected=run(binary,["--project-source",oversized,packet],False)
        assert "file-byte budget" in rejected["stderr"]
    receipt={"scope":"actual_native_source_projection_only","schema_version":1,"goal_complete":False,"opennurbs_commit":PIN,
        "binary_sha256":digest(binary),"fixture_binary_sha256":digest(fixture),"runner_sha256":digest(runner),
        "source_sha256":digest(ROOT/"native/native_exchange.cpp"),"fixture_source_sha256":digest(ROOT/"native/source_inventory_fixture.cpp"),
        "header_sha256":digest(ROOT/"native/retained_merge.hpp"),
        "validator_sha256":digest(Path(__file__)),"runs":len(runs),"rejections":sum(r["status"]=="native_owner_rejected" for r in runs),
        "peak_committed_bytes":max(r["stats"]["peak_committed_bytes"] for r in runs),
        "operation_receipts":[{k:v for k,v in r.items() if k!="stdout"} for r in runs],"projections":projections}
    args.receipt.write_text(json.dumps(receipt,indent=2),encoding="utf-8")
    print(json.dumps({k:v for k,v in receipt.items() if k not in {"operation_receipts","projections"}}))


if __name__ == "__main__":
    main()
