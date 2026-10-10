# Rhino 5 / IronPython 2.7 bootstrap only. Product curve logic remains Rust.
# Run this in your working Rhino. Test geometry belongs to a separate process.
import Rhino
import scriptcontext as sc
import System
import os
import ntpath
import json
import time
import traceback

ROOT = 'H:/FreeCAD-src/build/om9-dev'
RUNNING = 'OM9_PHASE2_VERIFY_RUNNING'
LAST_REPORT = 'OM9_PHASE2_VERIFY_LAST_REPORT'

def main():
    if sc.sticky.get(RUNNING):
        Rhino.RhinoApp.WriteLine('Phase 2 verification is already running in this Rhino session.')
        return
    sc.sticky[RUNNING] = True
    sc.sticky.pop(LAST_REPORT, None)
    out = None
    try:
        if not str(Rhino.RhinoApp.Version).startswith('5.'):
            raise RuntimeError('Run this script inside Rhino 5.')
        helper = ntpath.join(ROOT, 'tests/run_rhino5_phase2_verify.ps1')
        pwsh = 'C:/Program Files/PowerShell/7/pwsh.exe'
        for path in [pwsh, helper]:
            if not os.path.isfile(path):
                raise RuntimeError('Missing required file: ' + path)
        out = ntpath.join('H:/FreeCAD-src/build/rhino5-phase2-user', System.Guid.NewGuid().ToString('N'))
        os.makedirs(out)
        result_path = ntpath.join(out, 'phase2-verification.json')
        progress_path = ntpath.join(out, 'progress.json')
        sc.sticky[LAST_REPORT] = result_path
        Rhino.RhinoApp.WriteLine('Phase 2: starting isolated Rhino 5 and OM9 Rust runtime.')
        Rhino.RhinoApp.WriteLine('Logs: ' + out)
        info = System.Diagnostics.ProcessStartInfo()
        info.FileName = pwsh
        info.Arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + helper + '" -OutputDirectory "' + out + '"'
        info.UseShellExecute = False
        info.CreateNoWindow = True
        info.WindowStyle = System.Diagnostics.ProcessWindowStyle.Hidden
        # Rhino may have been started before the command-line tool PATH changed.
        info.EnvironmentVariables['PATH'] = 'C:/Users/nguye/.cargo/bin;H:/FreeCAD-src/.pixi/envs/default;' + os.environ.get('PATH', '')
        process = System.Diagnostics.Process.Start(info)
        started = time.time()
        last_message = started
        stage = None
        while not process.HasExited:
            if time.time() - started > 1000:
                raise RuntimeError('Verification exceeded 1000 seconds. Inspect logs in ' + out + '; its isolated processes may still be running.')
            if os.path.isfile(progress_path):
                try:
                    progress = json.loads(System.IO.File.ReadAllText(progress_path))
                    if progress.get('stage') != stage:
                        stage = progress.get('stage')
                        Rhino.RhinoApp.WriteLine('Phase 2: ' + stage)
                        last_message = time.time()
                except (ValueError, IOError):
                    pass  # The child may be replacing the progress file.
            if time.time() - last_message >= 30:
                Rhino.RhinoApp.WriteLine('Phase 2: still testing; elapsed ' + str(int(time.time() - started)) + ' seconds.')
                last_message = time.time()
            Rhino.RhinoApp.Wait()
            # Timed Join pumps COM/SendMessage while waiting on Rhino's STA.
            System.Threading.Thread.CurrentThread.Join(100)
        if not os.path.isfile(result_path):
            raise RuntimeError('Verification did not produce a report. Child exit: ' + str(process.ExitCode))
        result = json.loads(System.IO.File.ReadAllText(result_path))
        if process.ExitCode != 0 or not result.get('ok'):
            raise RuntimeError(result.get('error', 'Verification process failed with exit ' + str(process.ExitCode)))
        Rhino.RhinoApp.WriteLine('Phase 2 Rhino test: PASS; fixtures=' + str(len(result['fixtures'])) + '; Rhino checks=' + str(result['rhino_checks']) + '; OM9 checks=' + str(result['host_checks']) + '; saved reread checks=' + str(result['saved_reread_checks']))
        Rhino.RhinoApp.WriteLine('Report: ' + result_path)
    except:
        error = traceback.format_exc()
        if out:
            with open(ntpath.join(out, 'manual-entry-error.txt'), 'w') as stream:
                stream.write(error)
        Rhino.RhinoApp.WriteLine('Phase 2 Rhino test: FAIL\n' + error)
        if out:
            Rhino.RhinoApp.WriteLine('Logs: ' + out)
    finally:
        sc.sticky[RUNNING] = False

if __name__ == '__main__':
    main()
