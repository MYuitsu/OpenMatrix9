import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class SubDExportTests(unittest.TestCase):
    def test_redraw_and_special_colors_win_over_legacy_images(self):
        cases={'ClayooEditDivide':'#FF0505',
               'ClayooEditCrease':'#05B3FF',
               'ClayooSelectionSelectNaked':'#FF60AF',
               'ClayooCreationFromTSplines':'#05FF05'}
        for key,color in cases.items():
            with self.subTest(key=key),tempfile.TemporaryDirectory() as tmp:
                ref=Path(tmp)/'ref';ref.mkdir()
                (ref/'MainMenu.ini').write_text('[Menu11]\nName=Clayoo\nIcon1='+key+'\n')
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

    def test_shared_ring_artwork_remains_the_builder_asset(self):
        with tempfile.TemporaryDirectory() as tmp:
            ref=Path(tmp)/'ref';ref.mkdir()
            key='ClayooCreationRing'
            (ref/'MainMenu.ini').write_text('[Menu11]\nName=Clayoo\nIcon1='+key+'\n[Menu13]\nName=Builder\nIcon1='+key+'\n')
            (ref/'Matrix.rui').write_text(rui())
            output=Path(tmp)/'Resources'
            report=export_assets(ref,output,allow_original=True)
            ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
            self.assertEqual(ini[key]['image'],'icons/builder-minimal/'+key+'.svg')
            self.assertEqual(report['authored_symbols'][key]['spec_id'],'OM9-SUBD-017')
