"""Exercise timestamp versions through the real launcher bundle producer."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[2]


class TimestampReleaseTests(unittest.TestCase):
    def build(self, version):
        temporary = tempfile.TemporaryDirectory(dir=ROOT / 'build')
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source = root / 'source'
        (source / 'tools').mkdir(parents=True)
        shutil.copy2(ROOT / 'tools/build-launcher-bundle.py', source / 'tools')
        if (ROOT / 'tools/release_version.py').is_file():
            shutil.copy2(ROOT / 'tools/release_version.py', source / 'tools')
        shutil.copytree(ROOT / 'launcher', source / 'launcher')
        (source / 'release.json').write_text(json.dumps({
            'version': version, 'freecad': '1.1.4', 'launcher_revision': 1,
        }))
        plugin = root / 'plugin'
        (plugin / 'bin').mkdir(parents=True)
        payload = {
            'InitGui.py': b'# package fixture',
            'bin/OpenMatrix9Gui.pyd': b'native package fixture',
            'bin/OM9ThreeDmImportWorker.exe': b'worker package fixture',
        }
        for path, data in payload.items():
            (plugin / path).write_bytes(data)
        (plugin / 'release-validation.json').write_text(json.dumps({
            'ok': True, 'version': version,
            'release_metadata_sha256': hashlib.sha256((source / 'release.json').read_bytes()).hexdigest(),
            'host': 'Official FreeCAD 1.1.4 Windows x64',
            'plugin_sha256': hashlib.sha256(payload['bin/OpenMatrix9Gui.pyd']).hexdigest(),
            'worker_sha256': hashlib.sha256(payload['bin/OM9ThreeDmImportWorker.exe']).hexdigest(),
        }))
        result = subprocess.run([
            sys.executable, str(source / 'tools/build-launcher-bundle.py'),
            '--plugin', str(plugin), '--output', str(root / 'bundle'),
        ], capture_output=True, text=True)
        return result, root

    def setUp(self):
        (ROOT / 'build').mkdir(exist_ok=True)

    def test_timestamp_version_produces_installable_named_bundle(self):
        result, root = self.build('0.3.4-202610121234')
        self.assertEqual(result.returncode, 0, result.stderr)
        archive = root / 'OpenMatrix9-0.3.4-202610121234-SetupLauncher-Windows-x64.zip'
        self.assertTrue(archive.is_file())
        with zipfile.ZipFile(archive) as package:
            manifest = json.loads(package.read('launcher-manifest.json'))
            self.assertEqual(manifest['version'], '0.3.4-202610121234')
            self.assertEqual(package.read('OpenMatrix9/bin/OpenMatrix9Gui.pyd'), b'native package fixture')
            evidence = json.loads(package.read('OpenMatrix9/release-validation.json'))
            self.assertEqual(evidence['version'], manifest['version'])
            self.assertEqual(evidence['release_metadata_sha256'], hashlib.sha256((root / 'source/release.json').read_bytes()).hexdigest())

    def test_legacy_release_remains_installable(self):
        result, root = self.build('0.0.3')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue((root / 'OpenMatrix9-0.0.3-SetupLauncher-Windows-x64.zip').is_file())

    def test_invalid_date_and_noncanonical_versions_are_rejected(self):
        for version in ['0.3.4-202602291200', '0.3.4-202613011200',
                        '0.3.4-202610122460', '0.3.4-20261012',
                        '00.3.4-202610121234', '0.3.4-202610121234-extra']:
            with self.subTest(version=version):
                result, root = self.build(version)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((root / 'bundle').exists())


if __name__ == '__main__':
    unittest.main()
