import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class ToolsExportTests(unittest.TestCase):
    def test_authored_tools_and_command_colors_win_over_legacy_png(self):
        cases={'BuilderRingRail':'#4260FF','gvSmartTarget':'#FF0CA1',
               'BuilderImageTrace':'#FF05FF','BuilderProfile':'#FFDE2A',
               'gvJoinHistory':'#FFD705'}
        for key,color in cases.items():
            with self.subTest(key=key),tempfile.TemporaryDirectory() as tmp:
                ref=Path(tmp)/'ref';ref.mkdir()
                (ref/'MainMenu.ini').write_text('[Menu14]\nName=Tools\nIcon1='+key+'\n')
                (ref/'Matrix.rui').write_text(rui())
                shifted={('ButtonIcons.bin',key+'_1'):{'bytes':b'legacy','size':[25,25],
                          'png_sha256':'a'*64,'original_png_sha256':'b'*64}}
                output=Path(tmp)/'Resources'
                with patch('export_menu_assets.read_shifted_icons',return_value=shifted):
                    report=export_assets(ref,output,allow_original=True)
                ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
                self.assertEqual(ini[key]['source'],'OpenMatrix9-authored-svg')
                self.assertIn(key,report['authored_symbols'])
                self.assertNotIn(key,report['shifted_icons'])
                self.assertIn(color,(output/ini[key]['image']).read_text())
