"""Extract installed Matrix90 pictures for reference, never into product Resources."""
import argparse
import configparser
import csv
import hashlib
import html
import json
from pathlib import Path
import re

from PySide6.QtGui import QImage
from export_menu_assets import BUTTON_ALIASES, read_button_icons


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def write_preserving(path, raw):
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and path.read_bytes() != raw:
        raise ValueError(f'Existing output differs; choose a new output folder: {path}')
    path.write_bytes(raw)


def extract(source, output, menu):
    source = source.resolve(); output = output.resolve()
    if output == source or output.is_relative_to(source):
        raise ValueError('Reference output must be outside installed UserInterface')
    records = []; sources = {}; buttons = {}; gallery = []
    for archive in ('ButtonIcons.bin', 'SliderIcons.bin', 'InBoxIcons.bin'):
        archive_path = source / archive
        fingerprint = digest(archive_path.read_bytes())
        folder = archive.removesuffix('.bin')
        decoded = read_button_icons(archive_path)
        sources[archive] = {'path': str(archive_path), 'sha256': fingerprint, 'records': len(decoded)}
        for key, item in decoded.items():
            image = QImage.fromData(item['bytes'], 'BMP')
            if image.isNull():
                raise ValueError(f'Qt cannot decode {archive}/{key}')
            bmp = output / folder / 'bmp' / (key + '.bmp')
            png = output / folder / 'png' / (key + '.png')
            write_preserving(bmp, item['bytes'])
            png.parent.mkdir(parents=True, exist_ok=True)
            # PNG is a lossless view; original BMP bytes are preserved alongside it.
            if not png.exists():
                if not image.save(str(png), 'PNG'):
                    raise ValueError(f'Cannot save PNG: {png}')
            if QImage(str(png)).convertToFormat(QImage.Format_ARGB32) != image.convertToFormat(QImage.Format_ARGB32):
                raise ValueError(f'PNG pixels differ: {png}')
            records.append({'archive': archive, 'key': key, 'size': item['size'], 'record_offset': item['record_offset'],
                            'bmp': bmp.relative_to(output).as_posix(), 'bmp_sha256': digest(item['bytes']),
                            'png': png.relative_to(output).as_posix(), 'png_sha256': digest(png.read_bytes())})
            if archive == 'ButtonIcons.bin':
                buttons[key] = png
        if digest(archive_path.read_bytes()) != fingerprint:
            raise ValueError(f'Source changed during extraction: {archive_path}')

    parser = configparser.ConfigParser(interpolation=None); parser.optionxform = str
    parser.read(menu, encoding='utf-8-sig')
    mappings = []; missing = []
    for section in parser.sections():
        icons = [(k, v.strip()) for k, v in parser[section].items() if re.fullmatch(r'Icon\d+', k)]
        if not icons:
            continue
        name = parser[section].get('Name', section)
        group = re.sub(r'[^A-Za-z0-9_-]', '-', section + '-' + name)
        tiles = []
        for position, key in icons:
            record = BUTTON_ALIASES.get(key, key) + '_1'
            if record not in buttons:
                missing.append({'section': section, 'key': key}); continue
            relative = Path('main-menu') / group / (f'{int(position[4:]):02d}-{key}.png')
            write_preserving(output / relative, buttons[record].read_bytes())
            mappings.append({'section': section, 'group': name, 'position': int(position[4:]), 'key': key,
                             'record': record, 'mapping': 'reviewed-command-alias' if key in BUTTON_ALIASES else 'exact-button-key',
                             'png': relative.as_posix()})
            tiles.append(f'<figure><img src="{html.escape(relative.as_posix(), quote=True)}"><figcaption>{html.escape(key)}</figcaption></figure>')
        gallery.append(f'<h2>{html.escape(name)}</h2><div class="grid">'+''.join(tiles)+'</div>')

    report = {'purpose': 'Original artwork reference only; modern OpenMatrix9 distribution remains separate',
              'sources': sources, 'records': records, 'main_menu_mappings': mappings, 'missing_main_menu': missing,
              'menu_source': str(menu.resolve()), 'menu_source_sha256': digest(menu.read_bytes())}
    (output / 'manifest.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    with (output / 'index.csv').open('w', encoding='utf-8-sig', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=records[0].keys()); writer.writeheader(); writer.writerows(records)
    page = '<!doctype html><meta charset="utf-8"><title>Matrix90 menu images</title><style>body{background:#555;color:white;font:14px Arial;padding:20px}.grid{display:flex;flex-wrap:wrap;gap:10px}figure{margin:0;width:170px;padding:10px;background:#333;border:1px solid #888}img{width:32px;height:32px;image-rendering:pixelated}figcaption{overflow-wrap:anywhere;margin-top:8px}a{color:#9dc8ff}</style><h1>Matrix90 — ảnh menu đối chiếu</h1><p>Ảnh trích từ bản cài đặt. OpenMatrix9 vẫn ưu tiên bộ icon người dùng vẽ lại. Các hậu tố _1/_2/_3 được giữ nguyên; chưa suy diễn trạng thái nút.</p>'+''.join(gallery)
    (output / 'index.html').write_text(page, encoding='utf-8')
    (output / 'README.md').write_text(
        '# Ảnh menu Matrix90 để đối chiếu\n\nMở `index.html` để xem các nhóm MAIN MENU và11 nút nhanh. '
        '`main-menu` sắp xếp PNG theo thứ tự INI. `ButtonIcons`, `SliderIcons`, `InBoxIcons` chứa toàn bộ ảnh tìm thấy, '
        'mỗi kho có `bmp` nguyên byte và `png` cùng pixel. `index.csv` liệt kê tên/đường dẫn; `manifest.json` ghi nguồn/hash/offset. '
        'Đây là tài liệu tham chiếu, không phải bộ artwork phân phối cho OpenMatrix9. Không sửa file bản cài đặt.\n', encoding='utf-8')
    return {'output': str(output), 'archive_records': {k: v['records'] for k, v in sources.items()},
            'total_png': len(records), 'menu_positions': len(mappings), 'menu_unique_keys': len({x['key'] for x in mappings}),
            'missing_main_menu': missing, 'source_hashes_unchanged': True, 'png_pixels_match_bmp': True}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--menu', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(extract(args.source, args.output, args.menu), ensure_ascii=False))
