import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class CurveIconExportTests(unittest.TestCase):
    def test_curve_svg_survives_reexport_with_legacy_images_enabled(self):
        """A later full export must not restore the replaced Matrix artwork."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / 'ref'
            root.mkdir()
            (root / 'MainMenu.ini').write_text(
                '[Menu7]\nName=Curve\nIcon1=CurveLineSingleLine\n', encoding='utf-8')
            (root / 'Matrix.rui').write_text(rui(), encoding='utf-8')
            shifted = {('ButtonIcons.bin', 'CurveLineSingleLine_1'): {
                'bytes': b'legacy image', 'size': [25, 25],
                'png_sha256': 'a' * 64, 'original_png_sha256': 'b' * 64}}
            output = Path(tmp) / 'Resources'
            with patch('export_menu_assets.read_shifted_icons', return_value=shifted):
                report = export_assets(root, output, allow_original=True)
            ini = configparser.ConfigParser(interpolation=None)
            ini.read(output / 'menu/icons.ini', encoding='utf-8')
            relative = ini['CurveLineSingleLine']['image']
            self.assertTrue(relative.endswith('.svg'), relative)
            self.assertEqual(ini['CurveLineSingleLine']['source'], 'OpenMatrix9-authored-svg')
            self.assertIn('CurveLineSingleLine', report['authored_symbols'])
            self.assertNotIn('CurveLineSingleLine', report['shifted_icons'])
            self.assertFalse((output / 'icons/rgb-plus5').exists())
            self.assertIn('<svg', (output / relative).read_text(encoding='utf-8'))


if __name__ == '__main__':
    unittest.main()
