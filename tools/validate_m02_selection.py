#!/usr/bin/env python
"""Replay M02 gestures in the owned native app and retain measured, bounded evidence."""
from __future__ import annotations
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]

def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def yellow(color: tuple[int, int, int]) -> bool:
    # GPU textures are linearly filtered at fractional DPI: classify hue/chroma, not full brightness.
    r,g,b=color
    return r>100 and g>90 and b<min(r,g)*0.5 and 0.9<r/max(g,1)<1.7

def sources() -> dict[str, str]:
    paths = [ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']
    paths += [p for p in (ROOT / 'crates').rglob('*') if p.is_file() and p.suffix in {'.rs', '.toml', '.cpp', '.h'}]
    paths.append(ROOT / 'native/occt/pin.json')
    return {p.relative_to(ROOT).as_posix(): sha(p) for p in sorted(paths)}

def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, default=Path('L:/om9-target/debug/om9.exe'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--zoom', type=float, default=1.0)
    parser.add_argument('--gpu', action='store_true', help=argparse.SUPPRESS)  # Historical CLI spelling; rendering is always GPU.
    parser.add_argument('--occurrences', action='store_true', help='Replay full nested member identities and their four viewport outlines.')
    parser.add_argument('--brep', action='store_true', help='Use native BRep fixtures and the optional OCCT runtime (gpu-sphere build).')
    args = parser.parse_args()
    if args.brep:
        args.occurrences=True
    binary = args.binary.resolve(strict=True)
    output = args.output.resolve()
    output.relative_to((ROOT / 'build/standalone-validation/ui-artifacts').resolve())
    output.mkdir(parents=True, exist_ok=False)
    env = {k: v for k, v in os.environ.items() if not k.startswith('OM9_')}
    env.update(OM9_UI_AUDIT_SELECTION='1', OM9_UI_AUDIT_ZOOM=str(args.zoom),
               OM9_STARTUP_DIALOG='0',
               OM9_UI_AUDIT_WINDOW=f'{max(1600,round(1400*args.zoom))}x{max(1000,round(800*args.zoom))}', CARGO_TARGET_DIR='L:/om9-target',
               LOCALAPPDATA=str(output / 'profile'), APPDATA=str(output / 'profile'))
    if args.occurrences:
        env['OM9_UI_AUDIT_OCCURRENCES']='1'
    if args.brep:
        env['OM9_UI_AUDIT_BREP']='1'
    if args.brep:
        sdk=ROOT / 'build/standalone-deps/occt-8.0.1-sdk/bin'
        third=ROOT / 'build/standalone-deps/occt-8.0.1-3rdparty'
        dll_dirs={str(sdk)} | {str(p.parent) for p in third.rglob('*.dll')}
        env['PATH']=os.pathsep.join(sorted(dll_dirs))+os.pathsep+env.get('PATH','')
    before = sources()
    binary_sha = sha(binary)
    started = time.monotonic()
    command = [str(binary), '--ui-audit', str(output)]
    with (output / 'stdout.log').open('wb') as stdout, (output / 'stderr.log').open('wb') as stderr:
        process = subprocess.Popen(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr,
                                   creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0))
        try:
            process.wait(timeout=60)
        except subprocess.TimeoutExpired:
            process.terminate()  # Only this harness's child process, never an existing OM9 session.
            process.wait(timeout=5)
    receipt_path = output / 'ui-receipt.json'
    receipt = json.loads(receipt_path.read_text(encoding='utf-8')) if receipt_path.exists() else {}
    checks = receipt.get('selection_replay', [])
    screenshot = output / 'workspace.png'
    probes=[]
    move_image=output / 'm02-move.png'
    if move_image.exists() and len(checks)>1:
        with Image.open(move_image).convert('RGB') as frame:
            scale=receipt.get('regions',{}).get('pixels_per_point',1.0)
            for sample in checks[1].get('actual',{}).get('curve_samples',[]):
                x,y=[round(float(v)*scale) for v in sample['screen']]
                colors=[frame.getpixel((px,py)) for px in range(max(0,x-3),min(frame.width,x+4))
                        for py in range(max(0,y-3),min(frame.height,y+4))]
                probes.append(dict(view=sample['view'],pixel=[x,y],selected_color_visible=any(yellow(c) for c in colors)))
    mesh_image=output / 'm02-mesh.png'
    if mesh_image.exists() and len(checks)==12:
        with Image.open(mesh_image).convert('RGB') as frame:
            scale=receipt.get('regions',{}).get('pixels_per_point',1.0)
            x,y=[round(float(v)*scale) for v in checks[10]['actual']['mesh_probe']]
            colors=[frame.getpixel((px,py)) for px in range(max(0,x-3),min(frame.width,x+4)) for py in range(max(0,y-3),min(frame.height,y+4))]
            colored=any(g>r*1.1 and g>b*1.1 for r,g,b in colors)
            probes.append(dict(view='Looking Down mesh',pixel=[x,y],selected_color_visible=colored))
    if args.occurrences:
        capture=output/'m02-occurrence.png'
        if capture.exists() and checks:
            scale=receipt.get('regions',{}).get('pixels_per_point',1.0)
            with Image.open(capture).convert('RGB') as frame:
                for probe in checks[0]['actual'].get('probes',[]):
                    x,y=[round(float(v)*scale) for v in probe['screen']]
                    colors=[frame.getpixel((px,py)) for px in range(max(0,x-2),min(frame.width,x+3)) for py in range(max(0,y-2),min(frame.height,y+3))]
                    present=any(60<=r<=125 and g>=190 and 95<=b<=170 for r,g,b in colors)
                    probes.append(dict(view=probe['view'],pixel=[x,y],expected_green=probe['expected_green'],observed_green=present))
        visual_pass=len(probes)==5 and all(p['observed_green']==p['expected_green'] for p in probes)
    else:
        visual_pass=len(probes)==3 and all(p['selected_color_visible'] for p in probes)
    passed = (process.returncode == 0 and len(checks) == (4 if args.occurrences else 12) and all(c.get('pass') is True for c in checks)
              and receipt.get('screenshot_saved') is True and screenshot.is_file()
              and len(receipt.get('gpu_scene_views', [])) == 4
              and visual_pass
              and sources() == before and sha(binary) == binary_sha)
    report = dict(schema=1, module='OM9-L2-01-07', target='standalone Rust native winit/wgpu',
                  utc=dt.datetime.now(dt.timezone.utc).isoformat(), command=command,
                  zoom=args.zoom, gpu_requested=True, scenario='brep-occurrences' if args.brep else ('occurrences' if args.occurrences else 'selection'), elapsed_seconds=round(time.monotonic()-started, 3),
                  exit_code=process.returncode, binary_sha256=binary_sha, source_sha256=before,
                  checks=checks, passed=passed, screenshot_sha256=sha(screenshot) if screenshot.exists() else None,
                  gpu_scene_views=receipt.get('gpu_scene_views',[]),
                  preview_screenshots={p.name:sha(p) for p in output.glob('m02-*.png')},
                  visible_moved_curve_probes=probes,
                  human_approval=None, limitations=[('Native BRep replay covers a box; curved faces/edges and codec are covered by targeted Rust/native tests.' if args.brep else 'Native BRep topology is not represented by this mesh/curve fixture.'),
                                                  'Pointer, keyboard and focus are synthetic events through actual widgets; OS foreground focus is not proved.'])
    (output / 'm02-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(dict(passed=passed,exit_code=process.returncode,elapsed_seconds=report['elapsed_seconds'],check_count=len(checks),failed_checks=[c['name'] for c in checks if not c['pass']],pixel_probes=probes),ensure_ascii=False))
    if not passed:
        print((output / 'stderr.log').read_text(errors='replace')[-3000:])
        raise SystemExit(1)

if __name__ == '__main__':
    main()
