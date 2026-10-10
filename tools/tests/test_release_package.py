"""Exercise the real packager with a tiny, internally verified payload."""
import hashlib,json,subprocess,sys,tempfile,unittest,zipfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]

class ReleasePackageTests(unittest.TestCase):
    def test_release003_manifest_and_download_name(self):
        (ROOT/'build').mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=ROOT/'build') as temporary:
            root=Path(temporary);plugin=root/'plugin';(plugin/'bin').mkdir(parents=True)
            files={'InitGui.py':b'# fixture','bin/OpenMatrix9Gui.pyd':b'native fixture',
                   'bin/OM9ThreeDmImportWorker.exe':b'worker fixture'}
            for name,data in files.items():(plugin/name).write_bytes(data)
            validation={'ok':True,'host':'Official FreeCAD 1.1.4 Windows x64',
                        'plugin_sha256':hashlib.sha256(files['bin/OpenMatrix9Gui.pyd']).hexdigest(),
                        'worker_sha256':hashlib.sha256(files['bin/OM9ThreeDmImportWorker.exe']).hexdigest()}
            (plugin/'release-validation.json').write_text(json.dumps(validation))
            result=subprocess.run([sys.executable,str(ROOT/'tools/build-launcher-bundle.py'),
                                   '--plugin',str(plugin),'--output',str(root/'bundle')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            manifest=json.loads((root/'bundle/launcher-manifest.json').read_text())
            self.assertEqual(manifest['version'],'0.0.3','must package the approved new release')
            archive=root/'OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip'
            self.assertTrue(archive.is_file(),'versioned name must be discoverable by the updater')
            with zipfile.ZipFile(archive) as package:
                self.assertIsNone(package.testzip())
                self.assertEqual(package.read('OpenMatrix9/bin/OpenMatrix9Gui.pyd'),files['bin/OpenMatrix9Gui.pyd'])
                self.assertIn('Start-OM9.cmd',package.namelist())

if __name__=='__main__':unittest.main()
