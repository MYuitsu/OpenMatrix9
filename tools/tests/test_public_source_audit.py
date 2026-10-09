import configparser
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from public_source_audit import audit


class PublicSourceAuditTests(unittest.TestCase):
    def make(self,root):
        menu=root/'input';menu.mkdir();(menu/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
        export_assets(menu,root/'Resources')

    def copy_public_guide(self,root):
        source=Path(__file__).resolve().parents[2]/'ref/rhino5/windows_pdf_user_s_guide.pdf'
        guide=root/'ref/rhino5/windows_pdf_user_s_guide.pdf'
        guide.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(source,guide)
        return guide

    def test_private_directory_and_unbound_bitmap_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root)
            self.assertEqual(audit(root)['errors'],[])
            (root/'analysis').mkdir();(root/'Resources/icons/private.png').write_bytes(b'not-approved')
            errors=audit(root)['errors']
            self.assertTrue(any('Private directory' in e for e in errors))
            self.assertTrue(any('Non-vector icon' in e for e in errors))

    def test_exact_public_guide_and_reference_metadata_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root);self.copy_public_guide(root)
            reference=root/'ref/rhino5'
            (reference/'README.md').write_text('# Rhino 5 public documentation\n',encoding='utf-8')
            (reference/'SOURCES.json').write_text('{"sources":[]}',encoding='utf-8')
            (reference/'CORE_REFERENCE_INDEX.json').write_text('{"topics":[]}',encoding='utf-8')
            self.assertEqual(audit(root)['errors'],[])

    def test_public_guide_same_size_tampering_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root);guide=self.copy_public_guide(root)
            with guide.open('r+b') as stream:
                first=stream.read(1);stream.seek(0);stream.write(bytes([first[0]^1]))
            self.assertTrue(any('Public reference hash mismatch' in e for e in audit(root)['errors']))

    def test_public_guide_truncation_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root);guide=self.copy_public_guide(root)
            with guide.open('r+b') as stream:stream.truncate(guide.stat().st_size-1)
            self.assertTrue(any('Public reference size mismatch' in e for e in audit(root)['errors']))

    def test_public_guide_other_paths_fail(self):
        for relative in ('ref/rhino5/other.pdf','ref/other/windows_pdf_user_s_guide.pdf',
                         'docs/windows_pdf_user_s_guide.pdf'):
            with self.subTest(path=relative),tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);self.make(root);guide=self.copy_public_guide(root)
                destination=root/relative;destination.parent.mkdir(parents=True,exist_ok=True)
                guide.rename(destination)
                errors=audit(root)['errors']
                self.assertTrue(any('Private/binary artifact: '+relative in e for e in errors))

    def test_unapproved_reference_files_fail(self):
        for relative in ('ref/private.md','Ref/rhino5/private.md','ref/rhino5/private.md','ref/rhino5/private.json',
                         'ref/rhino5/recovered.py','ref/rhino5/recovered.cpp',
                         'ref/rhino5/resources.svg','ref/rhino5/private.dll'):
            with self.subTest(path=relative),tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);self.make(root);file=root/relative
                file.parent.mkdir(parents=True,exist_ok=True);file.write_text('unapproved')
                self.assertTrue(any('Private reference artifact: '+relative in e for e in audit(root)['errors']))

    def test_unapproved_reference_directories_fail_even_if_excluded(self):
        for relative in ('ref/private','ref/rhino5/private','ref/rhino5/build','ref/rhino5/.git',
                         'ref/rhino5/__pycache__','ref/rhino5/README.md'):
            with self.subTest(path=relative),tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);self.make(root);(root/relative).mkdir(parents=True)
                self.assertTrue(any('Private directory: '+relative in e for e in audit(root)['errors']))

    def test_reference_metadata_rejects_binary_and_invalid_json(self):
        for name,data in (('README.md',b'raw\x00resource'),('README.md',b'\xff'),
                          ('SOURCES.json',b'not JSON'),('CORE_REFERENCE_INDEX.json',b'[]')):
            with self.subTest(name=name,data=data),tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);self.make(root);reference=root/'ref/rhino5'
                reference.mkdir(parents=True);(reference/name).write_bytes(data)
                self.assertTrue(any('Invalid public reference metadata' in e for e in audit(root)['errors']))

    def test_reference_exception_keeps_credential_and_binary_checks(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root);self.copy_public_guide(root)
            (root/'ref/rhino5/.env').write_text('TOKEN=private')
            (root/'ref/rhino5/private.key').write_text('private key')
            (root/'ref/rhino5/private.exe').write_bytes(b'private resource')
            errors=audit(root)['errors']
            self.assertTrue(any('Credential file: ref/rhino5/.env' in e for e in errors))
            self.assertTrue(any('Key material: ref/rhino5/private.key' in e for e in errors))
            self.assertTrue(any('Private/binary artifact: ref/rhino5/private.exe' in e for e in errors))

    def test_tampered_svg_and_path_escape_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root)
            ini=configparser.ConfigParser();ini.read(root/'Resources/menu/icons.ini')
            file=root/'Resources'/ini['FileNew']['image']
            file.write_text('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><script>bad()</script></svg>')
            self.assertTrue(any('Non-self-contained' in e for e in audit(root)['errors']))
            ini['FileNew']['image']='../../outside.svg'
            with (root/'Resources/menu/icons.ini').open('w') as stream:ini.write(stream)
            self.assertTrue(any('Escaping binding' in e for e in audit(root)['errors']))

    def test_reference_screenshot_in_skill_assets_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root)
            file=root/'skills/example/assets/reference.png';file.parent.mkdir(parents=True)
            file.write_bytes(b'private reference screenshot')
            self.assertIn('Unreviewed raster artifact: skills/example/assets/reference.png',audit(root)['errors'])


if __name__=='__main__':unittest.main()
