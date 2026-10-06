import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class SolidIconExportTests(unittest.TestCase):
    def test_solid_redraw_precedes_legacy_art_on_reexport(self):
        """A future export must retain authored icons even with originals enabled."""
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'ref'
            root.mkdir()
            (root/'MainMenu.ini').write_text(
                '[Menu9]\nName=Solid\nIcon1=SolidUnion\nIcon2=SolidTube\n',encoding='utf-8')
            (root/'Matrix.rui').write_text(rui(),encoding='utf-8')
            shifted={('ButtonIcons.bin',key+'_1'):{'bytes':b'legacy image','size':[25,25],
                'png_sha256':'a'*64,'original_png_sha256':'b'*64} for key in ('SolidUnion','SolidTube')}
            output=Path(tmp)/'Resources'
            with patch('export_menu_assets.read_shifted_icons',return_value=shifted):
                report=export_assets(root,output,allow_original=True)
            ini=configparser.ConfigParser(interpolation=None)
            ini.read(output/'menu/icons.ini',encoding='utf-8')
            for key in ('SolidUnion','SolidTube'):
                self.assertEqual(ini[key]['source'],'OpenMatrix9-authored-svg')
                self.assertTrue(ini[key]['image'].endswith('.svg'))
                self.assertIn(key,report['authored_symbols'])
                self.assertNotIn(key,report['shifted_icons'])
                self.assertIn('<svg',(output/ini[key]['image']).read_text(encoding='utf-8'))
            self.assertFalse((output/'icons/rgb-plus5').exists())
            # The exporter also emits the SolidPtOn reviewed alias as an
            # auxiliary asset even when it is absent from this tiny menu.
            self.assertIn('SolidPtOn',report['authored_symbols'])
            self.assertEqual(report['solid_icon_style']['count'],3)


if __name__=='__main__':
    unittest.main()
