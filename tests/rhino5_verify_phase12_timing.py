# Rhino 5 / IronPython 2.7 bootstrap only. This is test bootstrap; product state/cache/curve logic remains Rust.
# Run this in your working Rhino. Test geometry belongs to a separate process.
import Rhino
import scriptcontext as sc
import System
import os
import ntpath
import json
import time
import traceback
import sys

ROOT = 'H:/FreeCAD-src/build/om9-perf-dev'
RUNNING = 'OM9_PHASE12_VERIFY_RUNNING'
LAST_REPORT = 'OM9_PHASE12_VERIFY_LAST_REPORT'
sys.path.insert(0, ntpath.join(ROOT, 'tests'))
import phase12_timing
try:
    reload
except NameError:
    from importlib import reload
reload(phase12_timing)
from phase12_timing import read_new_events, format_event

def main():
    if sc.sticky.get(RUNNING):
        Rhino.RhinoApp.WriteLine('Phase 1/2 verification is already running in this Rhino session.')
        return
    sc.sticky[RUNNING] = True
    sc.sticky.pop(LAST_REPORT, None)
    out = None
    try:
        if not str(Rhino.RhinoApp.Version).startswith('5.'):
            raise RuntimeError('Run this script inside Rhino 5.')
        helper = ntpath.join(ROOT, 'tests/run_rhino5_phase12_verify.ps1')
        pwsh = 'C:/Program Files/PowerShell/7/pwsh.exe'
        for path in [pwsh, helper]:
            if not os.path.isfile(path):
                raise RuntimeError('Missing required file: ' + path)
        out = ntpath.join('H:/FreeCAD-src/build/rhino5-phase12-user', System.Guid.NewGuid().ToString('N'))
        os.makedirs(out)
        result_path = ntpath.join(out, 'phase12-verification.json')
        progress_path = ntpath.join(out, 'progress.json')
        timing_directory = ntpath.join(out, 'timings')
        timing_offsets = {}
        timing_warnings = set()
        def drain_timings():
            try:
                for event in read_new_events(timing_directory, timing_offsets):
                    Rhino.RhinoApp.WriteLine(format_event(event))
            except (ValueError, IOError, TypeError) as error:
                warning = str(error)
                if warning not in timing_warnings:
                    timing_warnings.add(warning)
                    Rhino.RhinoApp.WriteLine('Timing log warning: ' + warning)
        sc.sticky[LAST_REPORT] = result_path
        Rhino.RhinoApp.WriteLine('Phase 1/2: starting isolated Rhino 5 and optimized OM9 runtime. This full check can take several minutes; it uses the system clipboard.')
        Rhino.RhinoApp.WriteLine('Logs: ' + out)
        info = System.Diagnostics.ProcessStartInfo()
        info.FileName = pwsh
        info.Arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + helper + '" -OutputDirectory "' + out + '"'
        info.UseShellExecute = False
        info.CreateNoWindow = True
        info.WindowStyle = System.Diagnostics.ProcessWindowStyle.Hidden
        info.RedirectStandardOutput = True
        info.RedirectStandardError = True
        # Rhino may have been started before the command-line tool PATH changed.
        info.EnvironmentVariables['PATH'] = 'C:/Users/nguye/.cargo/bin;H:/FreeCAD-src/.pixi/envs/default;' + os.environ.get('PATH', '')
        process = System.Diagnostics.Process.Start(info)
        stdout_task = process.StandardOutput.ReadToEndAsync()
        stderr_task = process.StandardError.ReadToEndAsync()
        started = time.time()
        last_message = started
        stage = None
        last_timing_poll = 0
        while not process.HasExited:
            if time.time() - started > 2400:
                raise RuntimeError('Verification exceeded 2400 seconds. Inspect logs in ' + out + '; its isolated processes may still be running.')
            if os.path.isfile(progress_path):
                try:
                    progress = json.loads(System.IO.File.ReadAllText(progress_path))
                    if progress.get('stage') != stage:
                        stage = progress.get('stage')
                        Rhino.RhinoApp.WriteLine('Phase 1/2: ' + stage)
                        last_message = time.time()
                except (ValueError, IOError):
                    pass  # The child may be replacing the progress file.
            if time.time() - last_timing_poll >= 1:
                drain_timings()
                last_timing_poll = time.time()
            if time.time() - last_message >= 30:
                Rhino.RhinoApp.WriteLine('Phase 1/2: still testing; elapsed ' + str(int(time.time() - started)) + ' seconds.')
                last_message = time.time()
            Rhino.RhinoApp.Wait()
            # Timed Join pumps COM/SendMessage while waiting on Rhino's STA.
            System.Threading.Thread.CurrentThread.Join(100)
        System.IO.File.WriteAllText(ntpath.join(out, 'launcher-stdout.log'), stdout_task.Result)
        System.IO.File.WriteAllText(ntpath.join(out, 'launcher-stderr.log'), stderr_task.Result)
        drain_timings()
        if not os.path.isfile(result_path):
            raise RuntimeError('Verification did not produce a report. Child exit: ' + str(process.ExitCode) + '; see launcher-stderr.log in ' + out)
        result = json.loads(System.IO.File.ReadAllText(result_path))
        if process.ExitCode != 0 or not result.get('ok'):
            raise RuntimeError(result.get('error', 'Verification process failed with exit ' + str(process.ExitCode)))
        Rhino.RhinoApp.WriteLine('Phase 1/2 Rhino test: PASS; fixtures=' + str(len(result['phase1']['fixtures']) + len(result['phase2']['fixtures'])) + '; Rhino checks=' + str(result['rhino_checks']) + '; OM9 reports=' + str(result['host_report_count']) + '; OM9 checks=' + str(result['host_checks']) + '; saved reread checks=' + str(result['saved_reread_checks']))
        Rhino.RhinoApp.WriteLine('Report: ' + result_path)
        Rhino.RhinoApp.WriteLine('Timing summary: ' + result['timing_summary'])
    except:
        error = traceback.format_exc()
        if out:
            System.IO.File.WriteAllText(ntpath.join(out, 'manual-entry-error.txt'), unicode(error))
        Rhino.RhinoApp.WriteLine('Phase 1/2 Rhino test: FAIL\n' + error)
        if out:
            Rhino.RhinoApp.WriteLine('Logs: ' + out)
            timing_summary = ntpath.join(out, 'timing-summary.json')
            if os.path.isfile(timing_summary):
                Rhino.RhinoApp.WriteLine('Timing summary: ' + timing_summary)
    finally:
        sc.sticky[RUNNING] = False

if __name__ == '__main__':
    main()
