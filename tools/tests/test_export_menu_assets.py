import base64
import bz2
import hashlib
import json
import configparser
from pathlib import Path
import struct
import sys
import tempfile
import unittest
import zlib
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from export_menu_assets import read_rui, read_form, export_assets, read_button_icons

def png():
    def chunk(t,d): return struct.pack('>I',len(d))+t+d+struct.pack('>I',zlib.crc32(t+d))
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',48,24,8,6,0,0,0))+chunk(b'IDAT',zlib.compress((b'\0'+b'\xff'*192)*24))+chunk(b'IEND',b'')

def rui(image=None,index=1):
    image=base64.b64encode(png()).decode() if image is None else image
    return f'<RhinoUI><macros><macro_item guid="macro" bitmap_id="image"><text><locale_1033>FileNew</locale_1033></text><script>! _New</script></macro_item></macros><bitmaps><normal_bitmap item_width="24" item_height="24"><bitmap_item guid="image" index="{index}"/><bitmap>{image}</bitmap></normal_bitmap></bitmaps></RhinoUI>'

def button_archive(entries, indexed=False):
    # VB6 stores UBound (last index), ASCII names, compressed UBound lengths,
    # a persisted StdPicture containing a BMP, and a four-byte zero terminator.
    raw_bmp=b'BM'+struct.pack('<IHHI',58,0,0,54)+struct.pack('<IiiHHIIiiII',40,1,1,1,24,0,4,0,0,0,0)+b'\xff\0\0\0'
    if indexed:
        raw_bmp=b'BM'+struct.pack('<IHHI',1082,0,0,1078)+struct.pack('<IiiHHIIiiII',40,1,1,1,8,0,4,0,0,0,0)+b'\0'*1028
    picture=b'\0'*42+struct.pack('<I',len(raw_bmp))+raw_bmp
    output=struct.pack('<I',len(entries)-1)
    for name in entries:
        key=name.encode('ascii'); compressed=bz2.compress(picture)
        output+=struct.pack('<I',len(key))+key+struct.pack('<I',len(compressed)-1)+compressed
    return output+b'\0'*4

class AssetsTests(unittest.TestCase):
    def test_shifted_png_precedes_named_and_original(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            (root/'ButtonIcons.bin').write_bytes(button_archive(['FileNew_1']))
            named=Path(t)/'named';named.mkdir();(named/'new.png').write_bytes(png())
            (named/'manifest.json').write_text(json.dumps([{'output_file':'new.png','source_used':'generated_modern_icon_board.png'}]))
            bindings=Path(t)/'bindings.json';bindings.write_text(json.dumps({'FileNew':{'file':'new.png','feature_id':'OM9-FILE-002','feature_name':'New'}}))
            shifted=Path(t)/'shifted';(shifted/'ButtonIcons/png').mkdir(parents=True)
            raw=png();(shifted/'ButtonIcons/png/FileNew_1.png').write_bytes(raw)
            (shifted/'manifest.json').write_text(json.dumps({'rgb_transform':{'delta':5,'clamp':[0,255],'alpha_unchanged':True},'records':[
                {'archive':'ButtonIcons.bin','key':'FileNew_1','png':'ButtonIcons/png/FileNew_1.png','png_sha256':hashlib.sha256(raw).hexdigest(),'original_png_sha256':'0'*64,'size':[48,24]}]}))
            out=Path(t)/'Resources'
            report=export_assets(root,out,named,bindings,allow_original=True,shifted_root=shifted)
            self.assertIn('FileNew',report['shifted_icons']);self.assertNotIn('FileNew',report['named_crops'])
            self.assertNotIn('FileNew',report['original_buttons'])
            ini=configparser.ConfigParser();ini.read(out/'menu/icons.ini')
            self.assertEqual(ini['FileNew']['source'],'Matrix90-RGB+5')
            self.assertEqual((out/ini['FileNew']['image']).read_bytes(),raw)

    def test_shifted_import_rejects_wrong_delta_hash_and_path(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n');(root/'Matrix.rui').write_text(rui())
            shifted=Path(t)/'shifted';shifted.mkdir();raw=png();(shifted/'new.png').write_bytes(raw)
            base={'rgb_transform':{'delta':5,'clamp':[0,255],'alpha_unchanged':True},'records':[
                {'archive':'ButtonIcons.bin','key':'FileNew_1','png':'new.png','png_sha256':hashlib.sha256(raw).hexdigest(),'original_png_sha256':'0'*64,'size':[48,24]}]}
            for mutate in ('delta','hash','path'):
                invalid=json.loads(json.dumps(base))
                if mutate=='delta':invalid['rgb_transform']['delta']=10
                elif mutate=='hash':invalid['records'][0]['png_sha256']='1'*64
                else:invalid['records'][0]['png']='../outside.png'
                (shifted/'manifest.json').write_text(json.dumps(invalid))
                with self.assertRaises(ValueError):export_assets(root,Path(t)/'Resources',shifted_root=shifted)

    def test_slider_archive_exports_display_record_with_spaces(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            (root/'SliderIcons.bin').write_bytes(button_archive(['Dial_Working Render']))
            report=export_assets(root,Path(t)/'Resources',allow_original=True)
            self.assertIn('Dial_Working Render',report['auxiliary_resolved'])
            self.assertEqual(report['original_buttons']['Dial_Working Render']['source'],'SliderIcons.bin')

    def test_button_archive_rejects_truncation_duplicate_keys_and_bad_terminator(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'ButtonIcons.bin'
            valid=button_archive(['FileNew_1'])
            compressed_start=4+4+len('FileNew_1')+4
            corrupt=valid[:compressed_start]+b'xx'+valid[compressed_start+2:]
            for raw in [valid[:-5],valid[:-4]+b'junk',corrupt,button_archive(['FileNew_1','FileNew_1']),
                        button_archive(['../FileNew_1'])]:
                p.write_bytes(raw)
                with self.assertRaises(ValueError):read_button_icons(p)

    def test_button_archive_supports_original_indexed_bmp_records(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'ButtonIcons.bin';p.write_bytes(button_archive(['Indexed_1'],indexed=True))
            item=read_button_icons(p)['Indexed_1']
            self.assertEqual(item['size'],[1,1])
            self.assertEqual(struct.unpack_from('<H',item['bytes'],28)[0],8)

    def test_named_redraw_precedes_original_asset(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            (root/'ButtonIcons.bin').write_bytes(button_archive(['FileNew_1']))
            named=Path(t)/'named';named.mkdir()
            (named/'new.png').write_bytes(png())
            (named/'manifest.json').write_text(json.dumps([{'output_file':'new.png','source_used':'generated_modern_icon_board.png'}]))
            bindings=Path(t)/'bindings.json';bindings.write_text(json.dumps({'FileNew':{'file':'new.png','feature_id':'OM9-FILE-002','feature_name':'New'}}))
            report=export_assets(root,Path(t)/'Resources',named,bindings,allow_original=True)
            self.assertIn('FileNew',report['named_crops'])
            self.assertNotIn('FileNew',report['original_buttons'])

    def test_modern_distribution_does_not_copy_legacy_images(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\nIcon2=Unknown\n')
            (root/'Matrix.rui').write_text(rui())
            (root/'ButtonIcons.bin').write_bytes(button_archive(['FileNew_1']))
            output=Path(t)/'Resources';report=export_assets(root,output)
            self.assertFalse((output/'icons/matrix9-24.png').exists())
            self.assertFalse((output/'icons/original').exists())
            self.assertEqual(report['original_buttons'],{})
            ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
            self.assertEqual(ini['FileNew']['source'],'OpenMatrix9-authored-svg')
            self.assertTrue((output/ini['Unknown']['image']).exists())
            self.assertEqual(ini['ToolsObjectSnapTangentTo']['status'],'fallback')

    def test_original_button_archive_resolves_exact_menu_name_without_rui_macro(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=OriginalOnly\nIcon2=Unknown\n')
            (root/'Matrix.rui').write_text(rui())
            archive=button_archive(['OriginalOnly_1'])
            (root/'ButtonIcons.bin').write_bytes(archive)
            report=export_assets(root,Path(t)/'Resources',allow_original=True)
            self.assertIn('OriginalOnly',report['resolved'])
            self.assertIn('Unknown',report['missing'])
            self.assertEqual(set(report['missing']),{'Unknown'})
            ini=configparser.ConfigParser();ini.read(Path(t)/'Resources/menu/icons.ini')
            self.assertEqual(ini['OriginalOnly']['mapping'],'exact-button-key')
            bmp=(Path(t)/'Resources'/ini['OriginalOnly']['image']).read_bytes()
            self.assertEqual(bmp[:2],b'BM')
            self.assertEqual((root/'ButtonIcons.bin').read_bytes(),archive)

    def test_modern_reexport_removes_only_managed_legacy_art(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            (root/'ButtonIcons.bin').write_bytes(button_archive(['FileNew_1']))
            output=Path(t)/'Resources'
            export_assets(root,output,allow_original=True)
            managed=output/'icons/original/FileNew_1.bmp'
            self.assertTrue(managed.exists())
            unmanaged=output/'icons/original/user-note.txt';unmanaged.write_text('preserve')
            export_assets(root,output)
            self.assertFalse(managed.exists())
            self.assertFalse((output/'icons/matrix9-24.png').exists())
            self.assertEqual(unmanaged.read_text(),'preserve')

    def test_modern_export_preserves_unmanaged_atlas_without_prior_ini(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            output=Path(t)/'Resources';(output/'icons').mkdir(parents=True)
            atlas=output/'icons/matrix9-24.png';atlas.write_bytes(b'user data')
            export_assets(root,output)
            self.assertEqual(atlas.read_bytes(),b'user data')

    def test_named_crop_overrides_atlas_and_copies_without_changing_source(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n')
            (root/'Matrix.rui').write_text(rui())
            named=Path(t)/'named';(named/'01-core').mkdir(parents=True)
            raw=png();(named/'01-core/new.png').write_bytes(raw)
            (named/'01-core/manifest.json').write_text(json.dumps([{'output_file':'new.png','source_used':'generated_modern_icon_board.png'}]))
            bindings=Path(t)/'bindings.json';bindings.write_text(json.dumps({'FileNew':{'file':'01-core/new.png','feature_id':'OM9-FILE-002','feature_name':'New','evidence':'explicit'}}))
            output=Path(t)/'Resources'
            report=export_assets(root,output,named,bindings)
            self.assertEqual(report['resolved'],['FileNew'])
            self.assertEqual(report['named_crops']['FileNew']['source_used'],'generated_modern_icon_board.png')
            self.assertEqual((output/'icons/named/01-core/new.png').read_bytes(),raw)
            self.assertEqual((named/'01-core/new.png').read_bytes(),raw)
            ini=configparser.ConfigParser();ini.read(output/'menu/icons.ini')
            self.assertEqual(ini['FileNew']['image'],'icons/named/01-core/new.png')

    def test_named_crop_rejects_escape_and_corrupt_png(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref';root.mkdir()
            (root/'MainMenu.ini').write_text('[Menu1]\nIcon1=FileNew\n');(root/'Matrix.rui').write_text(rui())
            named=Path(t)/'named';named.mkdir()
            (named/'broken.png').write_bytes(b'bad')
            bindings=Path(t)/'bindings.json'
            for path in ['../outside.png','broken.png']:
                bindings.write_text(json.dumps({'FileNew':{'file':path,'feature_id':'OM9-FILE-002','feature_name':'New','evidence':'explicit'}}))
                with self.assertRaises(ValueError):export_assets(root,Path(t)/'Resources',named,bindings)
    def test_guid_resolves_valid_atlas_rectangle(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'Matrix.rui'; p.write_text(rui())
            data=read_rui(p)
            self.assertEqual(data['macros']['FileNew']['rect'],[24,0,24,24])
            self.assertEqual(data['macros']['FileNew']['guid'],'macro')

    def test_bad_base64_or_rectangle_has_clear_error(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'Matrix.rui'
            for content in [rui('???'),rui(index=2)]:
                p.write_text(content)
                with self.assertRaises(ValueError): read_rui(p)

    def test_usercontrol_preserves_parent_and_resource_pointer(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'title.123'
            p.write_text('Begin VB.UserControl title\n Begin VB.Label close\n OleObjectBlob = "title.ctx":00AF\n End\nEnd\nAttribute VB_Name = "title"\n')
            data=read_form(p)
            self.assertEqual(data['controls'][1]['parent'],'title')
            self.assertEqual(data['references'][0]['offset'],175)

    def test_export_preserves_sources_and_marks_unmatched_icons_missing(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)/'ref'; root.mkdir()
            (root/'MainMenu.ini').write_text('[Settings]\nMenuCount=1\n[Menu1]\nIcon1=FileNew\nIcon2=Unknown\n[Top11]\nIcon1=Unknown\n')
            (root/'Matrix.rui').write_text(rui())
            before={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()}
            report=export_assets(root,Path(t)/'Resources',allow_original=True)
            self.assertEqual(report['resolved'],['FileNew'])
            self.assertIn('Unknown',report['missing'])
            self.assertEqual(before,{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir()})

if __name__=='__main__': unittest.main()
