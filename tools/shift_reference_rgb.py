"""Apply a verified, reversible RGB offset to the extracted reference images."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import uuid
from zipfile import ZipFile

import numpy as np
from PySide6.QtGui import QImage


def shifted(image, delta):
    result = image.convertToFormat(QImage.Format_RGBA8888)
    rows = np.frombuffer(result.bits(), dtype=np.uint8).reshape(result.height(), result.bytesPerLine())
    pixels = rows[:, :result.width()*4].reshape(result.height(), result.width(), 4)
    pixels[:, :, :3] = np.clip(pixels[:, :, :3].astype(np.int16) + delta, 0, 255).astype(np.uint8)
    return result


def pixels(image):
    image = image.convertToFormat(QImage.Format_RGBA8888)
    rows = np.frombuffer(image.constBits(), dtype=np.uint8).reshape(image.height(), image.bytesPerLine())
    return rows[:, :image.width()*4].reshape(image.height(), image.width(), 4).copy()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(root, backup, delta):
    root = root.resolve(); backup = backup.resolve()
    manifest_path = root / 'manifest.json'
    manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
    if manifest.get('rgb_transform'):
        raise ValueError('This folder already has an RGB transform; refusing an accidental second pass')
    if not -255 <= delta <= 255:
        raise ValueError('Delta must be within -255..255')
    # The archive must contain the manifest for this exact source set.
    with ZipFile(backup) as archive:
        if archive.read('manifest.json') != manifest_path.read_bytes():
            raise ValueError('Backup manifest does not match the current reference folder')
    files = sorted(p for p in root.rglob('*') if p.is_file() and p.suffix.lower() in ('.png', '.bmp'))
    stage = root.parent / ('rgb-stage-' + uuid.uuid4().hex)
    stage.mkdir()
    changed = 0; image_pixels = 0
    for index, source in enumerate(files, 1):
        if source.is_symlink() or not source.resolve().is_relative_to(root):
            raise ValueError('Image path escapes reference folder')
        image = QImage(str(source))
        if image.isNull():
            raise ValueError(f'Cannot decode {source}')
        before = pixels(image)
        result = shifted(image, delta)
        target = stage / source.relative_to(root)
        target.parent.mkdir(parents=True, exist_ok=True)
        if not result.save(str(target), source.suffix[1:].upper()):
            raise ValueError(f'Cannot save {target}')
        after = pixels(QImage(str(target)))
        expected = np.clip(before[:, :, :3].astype(np.int16) + delta, 0, 255).astype(np.uint8)
        if after.shape != before.shape or not np.array_equal(after[:, :, :3], expected) or not np.array_equal(after[:, :, 3], before[:, :, 3]):
            raise ValueError(f'Pixel/alpha validation failed: {source}')
        changed += int(not np.array_equal(before, after))
        image_pixels += image.width()*image.height()
        if index % 1000 == 0:
            print(f'Prepared and pixel-verified {index}/{len(files)}', flush=True)
    # All converted files are valid before any existing image is replaced.
    for source in files:
        (stage / source.relative_to(root)).replace(source)
    resolved_stage = stage.resolve()
    if resolved_stage.parent != root.parent or not resolved_stage.name.startswith('rgb-stage-'):
        raise ValueError('Staging cleanup path is outside the intended reference directory')
    shutil.rmtree(resolved_stage)
    for item in manifest['records']:
        for kind in ('bmp', 'png'):
            item['original_' + kind + '_sha256'] = item[kind + '_sha256']
            item[kind + '_sha256'] = digest(root / item[kind])
    report = {'operation': 'RGB channel offset', 'delta': delta, 'clamp': [0, 255],
              'images_processed': len(files), 'png': sum(p.suffix.lower()=='.png' for p in files),
              'bmp': sum(p.suffix.lower()=='.bmp' for p in files), 'images_with_changed_pixels': changed,
              'pixels_verified': image_pixels, 'alpha_unchanged': True, 'dimensions_unchanged': True,
              'backup': str(backup), 'reference_only': True}
    manifest['rgb_transform'] = report
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    # Refresh the record index while retaining its existing columns.
    import csv
    index_path = root / 'index.csv'
    with index_path.open(encoding='utf-8-sig', newline='') as stream:
        columns = next(csv.reader(stream))
    with index_path.open('w', encoding='utf-8-sig', newline='') as stream:
        writer = csv.DictWriter(stream, fieldnames=columns, extrasaction='ignore')
        writer.writeheader(); writer.writerows(manifest['records'])
    (root / 'README.md').write_text(
        f'# Ảnh menu Matrix90 — RGB {delta:+d}\n\nToàn bộ PNG/BMP được cộng {delta} vào mỗi kênh R/G/B, '
        'giới hạn255 và giữ alpha/kích thước. Đây là ảnh đã đổi màu, không còn là BMP nguyên byte. '
        'Mở `index.html` để xem menu; `index.csv` và `manifest.json` chứa hash hiện tại cùng hash bản gốc. '
        f'Bản gốc để hoàn tác: `{backup}`. File bản cài đặt Matrix90 và bộ icon hiện đại OpenMatrix9 không thay đổi.\n', encoding='utf-8')
    gallery = root / 'index.html'
    text = gallery.read_text(encoding='utf-8')
    text = text.replace('<h1>Matrix90 — ảnh menu đối chiếu</h1>', f'<h1>Matrix90 — ảnh menu RGB {delta:+d}</h1><p>R/G/B {delta:+d}, giới hạn255; giữ nguyên alpha và kích thước.</p>')
    gallery.write_text(text, encoding='utf-8')
    (root / 'rgb-plus5-report.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, required=True)
    parser.add_argument('--backup', type=Path, required=True)
    parser.add_argument('--delta', type=int, default=5)
    args = parser.parse_args()
    print(json.dumps(run(args.root, args.backup, args.delta)))
