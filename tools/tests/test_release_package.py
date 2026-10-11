"""Exercise the real packager with a tiny, internally verified payload."""
import hashlib,json,shutil,subprocess,sys,tempfile,unittest,zipfile
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
            metadata=json.loads((ROOT/'release.json').read_text())
            validation={'ok':True,'version':metadata['version'],
                        'release_metadata_sha256':hashlib.sha256((ROOT/'release.json').read_bytes()).hexdigest(),
                        'host':'Official FreeCAD 1.1.4 Windows x64',
                        'plugin_sha256':hashlib.sha256(files['bin/OpenMatrix9Gui.pyd']).hexdigest(),
                        'worker_sha256':hashlib.sha256(files['bin/OM9ThreeDmImportWorker.exe']).hexdigest()}
            (plugin/'release-validation.json').write_text(json.dumps(validation))
            result=subprocess.run([sys.executable,str(ROOT/'tools/build-launcher-bundle.py'),
                                   '--plugin',str(plugin),'--output',str(root/'bundle')],capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stderr)
            manifest=json.loads((root/'bundle/launcher-manifest.json').read_text())
            metadata=json.loads((ROOT/'release.json').read_text())
            self.assertEqual(manifest['version'],metadata['version'],'must package the selected source version')
            suffix=f"-r{metadata['launcher_revision']}" if metadata['launcher_revision']>1 else ''
            archive=root/f"OpenMatrix9-{metadata['version']}-SetupLauncher{suffix}-Windows-x64.zip"
            self.assertTrue(archive.is_file(),'versioned name must be discoverable by the updater')
            with zipfile.ZipFile(archive) as package:
                self.assertIsNone(package.testzip())
                self.assertEqual(package.read('OpenMatrix9/bin/OpenMatrix9Gui.pyd'),files['bin/OpenMatrix9Gui.pyd'])
                self.assertIn('Start-OM9.cmd',package.namelist())
                evidence=json.loads(package.read('OpenMatrix9/release-validation.json'))
                self.assertEqual(evidence['version'],metadata['version'])
                self.assertEqual(evidence['release_metadata_sha256'],hashlib.sha256((ROOT/'release.json').read_bytes()).hexdigest())

    def package_ci_fixture(self,validation_change=None,metadata_change=None):
        (ROOT/'build').mkdir(exist_ok=True)
        temporary=tempfile.TemporaryDirectory(dir=ROOT/'build')
        root=Path(temporary.name).resolve()
        self.assertEqual(root.parent,(ROOT/'build').resolve())
        self.addCleanup(temporary.cleanup)
        source=root/'source';(source/'tools/ci').mkdir(parents=True)
        for name in ('tools/ci/package_release.py','tools/build-launcher-bundle.py','tools/release_version.py',
                     'release.json','README.md','INSTALL.vi.md','INSTALL.en.md','LICENSE','THIRD_PARTY_NOTICES.md'):
            shutil.copy2(ROOT/name,source/name)
        shutil.copytree(ROOT/'launcher',source/'launcher')
        plugin=root/'plugin';(plugin/'bin').mkdir(parents=True)
        payload={'InitGui.py':b'# synthetic package fixture','bin/OpenMatrix9Gui.pyd':b'native fixture',
                 'bin/OM9ThreeDmImportWorker.exe':b'worker fixture'}
        for name,data in payload.items():(plugin/name).write_bytes(data)
        release=json.loads((source/'release.json').read_text())
        plugin_hash=hashlib.sha256(payload['bin/OpenMatrix9Gui.pyd']).hexdigest()
        validation={'ok':True,'version':release['version'],
            'release_metadata_sha256':hashlib.sha256((source/'release.json').read_bytes()).hexdigest(),
            'host':'Official FreeCAD 1.1.4 Windows x64','plugin_sha256':plugin_hash,
            'worker_sha256':hashlib.sha256(payload['bin/OM9ThreeDmImportWorker.exe']).hexdigest(),
            'gumball':{'ok':True,'plugin_sha256':plugin_hash,'runs':[{'name':'synthetic-'+str(i)} for i in range(30)]}}
        if validation_change:validation_change(validation)
        if metadata_change:
            metadata_change(release)
            (source/'release.json').write_text(json.dumps(release))
        (plugin/'release-validation.json').write_text(json.dumps(validation))
        (root/'build-paths.json').write_text(json.dumps({'plugin':str(plugin)}))
        result=subprocess.run([sys.executable,str(source/'tools/ci/package_release.py'),'--root',str(root)],capture_output=True,text=True)
        return result,root,release

    def test_ci_packager_retains_matching_version_evidence(self):
        result,root,release=self.package_ci_fixture()
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertEqual(len(list((root/'packages').glob('*.zip'))),2)
        manual=root/'packages'/f"OpenMatrix9-{release['version']}-FreeCAD-1.1.4-Windows-x64.zip"
        with zipfile.ZipFile(manual) as package:
            evidence=json.loads(package.read('OpenMatrix9/release-validation.json'))
            self.assertEqual(evidence['version'],release['version'])
            self.assertEqual(evidence['release_metadata_sha256'],hashlib.sha256((root/'source/release.json').read_bytes()).hexdigest())

    def test_ci_packager_rejects_stale_version_evidence(self):
        result,root,_=self.package_ci_fixture(validation_change=lambda validation:validation.update(version='9.9.9'))
        self.assertNotEqual(result.returncode,0)
        self.assertIn('release metadata changed after validation',result.stderr)
        self.assertFalse((root/'packages').exists())

    def test_ci_packager_rejects_changed_metadata_with_same_version(self):
        result,root,_=self.package_ci_fixture(metadata_change=lambda release:release.update(launcher_revision=release['launcher_revision']+1))
        self.assertNotEqual(result.returncode,0)
        self.assertIn('release metadata changed after validation',result.stderr)
        self.assertFalse((root/'packages').exists())

if __name__=='__main__':unittest.main()
