"""Measure owned layer primitives and cold metadata; never accept full Layers."""
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source,"sha256").hexdigest()


def source_paths():
    return {p for p in (ROOT/"standalone").rglob("*") if p.is_file() and "target" not in p.parts and p.suffix in {".rs",".toml",".lock",".json"}} | {Path(__file__).resolve()}


def main():
    before={p:digest(p) for p in source_paths()}
    commands=[
        ["-p","om9-core","--test","layer_transactions"],
        ["-p","om9-scene","--test","layer_colors"],
        ["-p","om9-core","--test","retained_imports","layers_assignment"],
        ["-p","om9-app","--test","cold_layers"],
        ["--workspace"],
    ]
    runs=[]
    cold=None
    scratch=ROOT/"build/standalone-validation"
    scratch.mkdir(parents=True,exist_ok=True)
    for n,args in enumerate(commands):
        command=["rtk","proxy","cargo","test","--locked","--manifest-path","Cargo.toml",*args,"--","--nocapture"]
        result=subprocess.run(command,cwd=ROOT,capture_output=True,timeout=180)
        output=result.stdout.decode("utf-8",errors="strict")+result.stderr.decode("utf-8",errors="replace")
        log=scratch/f"owned-layer-primitives-{n}.log"
        log.write_text(output,encoding="utf-8")
        if result.returncode: raise RuntimeError(output)
        counts=re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",output)
        assert counts and all(int(f)==0 for _,f,_ in counts),output
        runs.append({"command":command,"passed":sum(int(p) for p,_,_ in counts),"ignored":sum(int(i) for _,_,i in counts),"suites":len(counts),"log":{"path":log.relative_to(ROOT).as_posix(),"sha256":digest(log)}})
        for line in output.splitlines():
            if "OM9_LAYER_RECEIPT " in line: cold=json.loads(line.split("OM9_LAYER_RECEIPT ",1)[1])
    assert [r["passed"] for r in runs[:4]] == [12,1,1,1],runs
    assert cold and cold["exited"] and cold["active_processes"]==0 and not cold["full_layers_acceptance"]
    binary=Path(cold["binary"]).resolve(strict=True)
    assert cold["binary_sha256"]==digest(binary)
    assert source_paths()==set(before) and all(digest(p)==sha for p,sha in before.items()),"source changed during checks"
    receipt={"schema_version":1,"status":"validated_owned_layer_transaction_primitives","measured_at":dt.datetime.now(dt.timezone.utc).isoformat(timespec="seconds"),"goal_complete":False,"full_layers_acceptance":False,"runs":runs,"workspace_passed":runs[-1]["passed"],"workspace_ignored":runs[-1]["ignored"],"focused_checks_are_workspace_subset":True,"focused_tests":15,"cold_fixture":cold,"binary":{"path":binary.relative_to(ROOT).as_posix(),"sha256":digest(binary)},"source_sha256":{p.relative_to(ROOT).as_posix():sha for p,sha in sorted(before.items())},"limitations":["Preset identity, independent Hidden schema and migration remain open.","Actual Layers panel/CMD/current-output rebinding, selection/modal/Advanced/Lights/full feature acceptance remain open.","Ignored SDK integration tests and older native/UI receipts are not refreshed by this primitive batch.","No large-scene latency/performance or arbitrary native ownership acceptance claimed."]}
    path=ROOT/"docs/validation/2026-10-14-owned-layer-primitives.json"
    path.write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
    print(json.dumps({"status":receipt["status"],"workspace_passed":receipt["workspace_passed"],"workspace_ignored":receipt["workspace_ignored"],"focused_subset":15,"cold_processes":1,"goal_complete":False}))


if __name__=="__main__":
    main()
