import configparser
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from export_menu_assets import export_assets,read_shifted_icons


def rui(*args,**kwargs):
    # Synthetic unused input verifies that old metadata cannot affect export.
    return '<legacy-input-unused/>'


class PublicExportTests(unittest.TestCase):
    def test_legacy_option_cannot_restore_bitmap(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'input';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text('malformed-unused-legacy')
            output=Path(tmp)/'Resources'
            report=export_assets(root,output,allow_original=True)
            ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
            self.assertEqual(ini['FileNew']['source'],'OpenMatrix9-authored-svg')
            self.assertFalse(report['shifted_icons'])
            self.assertEqual(list(output.rglob('*.png')),[])

    def test_external_sources_rejected(self):
        for kw in ('named_root','named_bindings','button_icons','slider_icons','shifted_root'):
            with self.subTest(kw=kw),tempfile.TemporaryDirectory() as tmp:
                with self.assertRaises(ValueError):export_assets(tmp,Path(tmp)/'out',**{kw:tmp})
        with self.assertRaises(ValueError):read_shifted_icons(None)

    def test_unknown_key_does_not_write_partial_export(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);(root/'MainMenu.ini').write_text('[Menu1]\nIcon1=UnknownCommand\n')
            with self.assertRaises(ValueError):export_assets(root,root/'out')
            self.assertFalse((root/'out').exists())

    def test_complete_catalog_is_idempotent_and_has_no_legacy_provenance(self):
        root=Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as tmp:
            output=Path(tmp)/'Resources'
            source=root/'Resources/menu'
            export_assets(source,output)
            before={p.relative_to(output):p.read_bytes() for p in output.rglob('*') if p.is_file()}
            report=export_assets(source,output)
            self.assertEqual(len(report['authored_symbols']),510)
            self.assertEqual(before,{p.relative_to(output):p.read_bytes() for p in output.rglob('*') if p.is_file()})
            self.assertNotIn('reference_image',json.dumps(report))


if __name__=='__main__':unittest.main()
