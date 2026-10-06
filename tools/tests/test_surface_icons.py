import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui

class SurfaceExportTests(unittest.TestCase):
    def test_redraw_wins_over_legacy_images(self):
        with tempfile.TemporaryDirectory() as tmp:
            ref=Path(tmp)/'ref'
            ref.mkdir()
            key='SurfaceSweepSweep1Rail'
            (ref/'MainMenu.ini').write_text('[Menu8]\nName=Surface\nIcon1='+key+'\n')
            (ref/'Matrix.rui').write_text(rui())
            shifted={('ButtonIcons.bin',key+'_1'):{'bytes':b'legacy','size':[25,25],
                'png_sha256':'a'*64,'original_png_sha256':'b'*64}}
            output=Path(tmp)/'Resources'
            with patch('export_menu_assets.read_shifted_icons',return_value=shifted):
                report=export_assets(ref,output,allow_original=True)
            ini=configparser.ConfigParser()
            ini.read(output/'menu/icons.ini')
            self.assertEqual(ini[key]['source'],'OpenMatrix9-authored-svg')
            self.assertIn(key,report['authored_symbols'])
            self.assertNotIn(key,report['shifted_icons'])
            self.assertTrue(ini[key]['image'].endswith('.svg'))
            self.assertIn('#5FCB62',(output/ini[key]['image']).read_text())
