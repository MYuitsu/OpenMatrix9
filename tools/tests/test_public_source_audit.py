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

    def test_private_directory_and_unbound_bitmap_fail(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);self.make(root)
            self.assertEqual(audit(root)['errors'],[])
            (root/'ref').mkdir();(root/'Resources/icons/private.png').write_bytes(b'not-approved')
            errors=audit(root)['errors']
            self.assertTrue(any('Private directory' in e for e in errors))
            self.assertTrue(any('Non-vector icon' in e for e in errors))

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
