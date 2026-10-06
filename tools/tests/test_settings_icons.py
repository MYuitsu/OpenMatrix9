import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class SettingsExportTests(unittest.TestCase):
    def test_setting_symbols_and_palette_precede_legacy_png(self):
        for key,color in {'BuilderHead':'#C655A3','BuilderBezelCutter':'#F3782B',
                          'BuilderBeadOnCrv':'#FF67D0','BuilderBezel':'#FFD705'}.items():
            with self.subTest(key=key),tempfile.TemporaryDirectory() as tmp:
                ref=Path(tmp)/'ref';ref.mkdir()
                (ref/'MainMenu.ini').write_text('[Menu16]\nName=Settings\nIcon1='+key+'\n')
                (ref/'Matrix.rui').write_text(rui())
                legacy={('ButtonIcons.bin',key+'_1'):{'bytes':b'legacy','size':[25,25],
                    'png_sha256':'a'*64,'original_png_sha256':'b'*64}}
                output=Path(tmp)/'Resources'
                with patch('export_menu_assets.read_shifted_icons',return_value=legacy):
                    report=export_assets(ref,output,allow_original=True)
                ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
                self.assertEqual(ini[key]['source'],'OpenMatrix9-authored-svg')
                self.assertIn(key,report['authored_symbols'])
                self.assertNotIn(key,report['shifted_icons'])
                self.assertIn(color,(output/ini[key]['image']).read_text())

    def test_shared_bezel_retains_subd_asset_and_provenance(self):
        from settings_icons import write_asset
        from subd_icons import svg
        key='ClayooCreationClayooBezel'
        with tempfile.TemporaryDirectory() as tmp:
            metadata,record=write_asset(tmp,key)
            self.assertEqual(metadata['image'],'icons/subd-minimal/'+key+'.svg')
            self.assertEqual(record['spec_id'],'OM9-SUBD-016')
            self.assertEqual((Path(tmp)/metadata['image']).read_text(encoding='utf-8'),svg(key))
