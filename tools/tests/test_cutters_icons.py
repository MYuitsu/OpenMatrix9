import configparser
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets
from test_export_menu_assets import rui


class CuttersExportTests(unittest.TestCase):
    def test_authored_cutters_precede_legacy_png(self):
        for key,color in {'BuilderGemCutter':'#F2793A','BuilderChannelBuilder':'#FFFFCF',
                          'BuilderMicroProngCutter':'#FFFFCF','BuilderQuadFlip':'#FF6605'}.items():
            with self.subTest(key=key),tempfile.TemporaryDirectory() as tmp:
                ref=Path(tmp)/'ref';ref.mkdir()
                (ref/'MainMenu.ini').write_text('[Menu17]\nName=Cutters\nIcon1='+key+'\n')
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

    def test_shared_scallop_and_boolean_assets_are_unchanged(self):
        from cutters_icons import write_asset
        from settings_icons import svg as setting_svg
        from tools_icons import svg as tool_svg
        cases=[('BuilderBezelCutter','settings-minimal',setting_svg),
               ('BuilderBooleanBuilder','tools-minimal',tool_svg)]
        for key,folder,source in cases:
            with self.subTest(key=key),tempfile.TemporaryDirectory() as tmp:
                metadata,record=write_asset(tmp,key)
                self.assertEqual(metadata['image'],'icons/'+folder+'/'+key+'.svg')
                self.assertEqual((Path(tmp)/metadata['image']).read_text(encoding='utf-8'),source(key))
