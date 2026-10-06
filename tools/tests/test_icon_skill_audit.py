import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'skills/openmatrix9-icons/scripts'))
from audit_icons import audit


class IconSkillAuditTests(unittest.TestCase):
    def make_project(self, root, svg, digest=None):
        resources = root / 'Resources'
        (resources / 'menu').mkdir(parents=True)
        (resources / 'icons').mkdir()
        (resources / 'icons/test.svg').write_text(svg, encoding='utf-8')
        (resources / 'menu/MainMenu.ini').write_text('[Menu1]\nName=Example\nIconCount=1\nIcon1=Test\n')
        (resources / 'menu/icons.ini').write_text('[Test]\nimage=icons/test.svg\nsource=OpenMatrix9-authored-svg\nstatus=resolved\n')
        (resources / 'menu/source-manifest.json').write_text(json.dumps({
            'authored_symbols': {'Test': {'sha256': digest or hashlib.sha256(svg.encode()).hexdigest()}}}))
        return resources

    def test_audit_and_sheets_do_not_mutate_product_resources(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            resources = self.make_project(root, '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><path d="M4 4 L28 28" stroke="#FFFF05"/></svg>')
            before = {file: file.read_bytes() for file in resources.rglob('*') if file.is_file()}
            report = audit(root, 'Example', root / 'review')
            self.assertEqual(report['errors'], [])
            self.assertEqual(report['entries'], 1)
            self.assertTrue((root / 'review/review-gray.svg').is_file())
            self.assertTrue((root / 'review/review-dark.svg').is_file())
            self.assertTrue(all(file.read_bytes() == content for file, content in before.items()))
            with self.assertRaises(ValueError):
                audit(root, 'Example', resources / 'review')

    def test_audit_detects_changed_asset_and_embedded_bitmap(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.make_project(root, '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><image href="data:image/png;base64,AAAA"/></svg>', digest='a'*64)
            report = audit(root, 'Example', root / 'review')
            self.assertTrue(any('Manifest hash' in message for message in report['errors']))
            self.assertTrue(any('Non-vector' in message for message in report['errors']))


if __name__ == '__main__':
    unittest.main()
