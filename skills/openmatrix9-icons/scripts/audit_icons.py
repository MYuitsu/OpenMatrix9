"""Read-only menu icon audit and actual-size SVG review sheets (stdlib only)."""
import argparse
import base64
from collections import defaultdict
import configparser
import hashlib
import html
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

NS = 'http://www.w3.org/2000/svg'
ET.register_namespace('', NS)


def config(file):
    value = configparser.ConfigParser(interpolation=None)
    value.optionxform = str
    if not value.read(file, encoding='utf-8'):
        raise ValueError(f'Missing config: {file}')
    return value


def audit(root, group, output):
    resources = (root / 'Resources').resolve()
    output = output.resolve()
    if output.is_relative_to(resources):
        raise ValueError('Review output must be outside product Resources')
    menu = config(resources / 'menu/MainMenu.ini')
    bindings = config(resources / 'menu/icons.ini')
    manifest = json.loads((resources / 'menu/source-manifest.json').read_text(encoding='utf-8'))
    groups = [s for s in menu.values() if s.get('Name')
              and (group == 'All' or s.get('Name', '').casefold() == group.casefold())]
    if not groups:
        raise ValueError(f'Unknown menu group: {group}')
    errors, items, duplicates, artwork = [], [], defaultdict(list), {}
    for section in groups:
        keys = [v for k, v in section.items() if re.fullmatch(r'Icon\d+', k)]
        if section.get('IconCount') and len(keys) != int(section['IconCount']):
            errors.append(f"{section['Name']}: declared icon count disagrees")
        for key in keys:
            item = {'key': key, 'group': section['Name'], 'issues': []}
            if key not in bindings or not bindings[key].get('image'):
                item['issues'].append('Missing direct image binding')
                items.append(item)
                continue
            binding = bindings[key]
            file = (resources / binding['image']).resolve()
            if not file.is_relative_to(resources) or not file.is_file():
                item['issues'].append('Missing or escaping resource path')
                items.append(item)
                continue
            raw = file.read_bytes()
            item.update({'path': binding['image'], 'source': binding.get('source'),
                         'status': binding.get('status'), 'sha256': hashlib.sha256(raw).hexdigest()})
            if item['status'] not in ('resolved', 'fallback'):
                item['issues'].append('Image binding has no resolved/fallback status')
            record = next((manifest.get(field, {}).get(key) for field in
                           ('authored_symbols', 'shifted_icons', 'original_buttons', 'named_crops')
                           if key in manifest.get(field, {})), None)
            if record and record.get('sha256') != item['sha256']:
                item['issues'].append('Manifest hash differs')
            if item['source'] == 'OpenMatrix9-authored-svg' and not record:
                item['issues'].append('Authored asset has no manifest record')
            item['label'] = (record or {}).get('label', key)
            if file.suffix.lower() == '.svg':
                try:
                    svg = ET.fromstring(raw)
                    if svg.tag != f'{{{NS}}}svg':
                        raise ValueError('SVG namespace/root is missing')
                    box = [float(n) for n in svg.get('viewBox', '').replace(',', ' ').split()]
                    if len(box) != 4 or box[2] <= 0 or box[3] <= 0:
                        raise ValueError('Invalid viewBox')
                    for node in svg.iter():
                        tag = node.tag.rsplit('}', 1)[-1]
                        if tag in ('image', 'script', 'foreignObject'):
                            raise ValueError(f'Non-vector or active element: {tag}')
                        for attr, value in node.attrib.items():
                            if attr.startswith('on') or ('href' in attr and not value.startswith('#')):
                                raise ValueError('Active or external resource reference')
                            if 'url(' in value and not re.fullmatch(r'url\(#[\w-]+\)', value):
                                raise ValueError('External paint/resource reference')
                    geometry = b''.join(ET.tostring(n) for n in svg
                                        if n.tag.rsplit('}', 1)[-1] not in ('title', 'desc'))
                    if not geometry:
                        raise ValueError('No SVG drawing elements')
                    duplicates[hashlib.sha256(geometry).hexdigest()].append(key)
                    item['palette'] = sorted(set(re.findall(r'#[0-9a-fA-F]{6}\b', raw.decode('utf-8'))))
                    artwork[key] = svg
                except (ET.ParseError, ValueError) as exc:
                    item['issues'].append(str(exc))
            else:
                item['raster'] = True
                if file.suffix.lower() == '.png':
                    artwork[key] = 'data:image/png;base64,' + base64.b64encode(raw).decode('ascii')
            items.append(item)
    errors.extend(f"{item['key']}: {issue}" for item in items for issue in item['issues'])
    report = {'group': group, 'entries': len(items),
              'unique_assets': len({item.get('path') for item in items if item.get('path')}),
              'svg_entries': sum(item.get('path', '').endswith('.svg') for item in items),
              'errors': errors, 'shared_geometry': [keys for keys in duplicates.values() if len(keys) > 1],
              'recognition': 'Requires visual/semantic review; static audit cannot establish it',
              'items': items}
    output.mkdir(parents=True, exist_ok=True)
    (output / 'audit.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    for name, color in [('gray', '#696969'), ('dark', '#333333')]:
        cell_w, cell_h, columns = 200, 110, 6
        parts = [f'<svg xmlns="{NS}" width="{cell_w*columns}" height="{40+cell_h*((len(items)+columns-1)//columns)}">',
                 f'<rect width="100%" height="100%" fill="{color}"/>',
                 '<text x="12" y="24" fill="white" font-family="Arial" font-size="16">24 / 32 / 64 px — inspect at 100% zoom</text>']
        for index, item in enumerate(items):
            key = item['key']
            x, y = (index % columns) * cell_w, 40 + (index // columns) * cell_h
            label = html.escape(item.get('label', key)[:32])
            parts.append(f'<text x="{x+6}" y="{y+18}" fill="white" font-family="Arial" font-size="10">{index+1:02} {label}</text>')
            for size, dx in [(24, 8), (32, 50), (64, 108)]:
                drawing = artwork.get(key)
                if isinstance(drawing, ET.Element):
                    drawing = ET.fromstring(ET.tostring(drawing))
                    drawing.set('x', str(x+dx))
                    drawing.set('y', str(y+28))
                    drawing.set('width', str(size))
                    drawing.set('height', str(size))
                    parts.append(ET.tostring(drawing, encoding='unicode'))
                elif isinstance(drawing, str):
                    parts.append(f'<image x="{x+dx}" y="{y+28}" width="{size}" height="{size}" href="{drawing}"/>')
            parts.append(f'<path d="M{x} {y+105} H{x+cell_w}" stroke="#888888" stroke-width="0.5"/>')
        parts.append('</svg>')
        (output / f'review-{name}.svg').write_text('\n'.join(parts), encoding='utf-8')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project-root', required=True, type=Path)
    parser.add_argument('--group', default='Curve', help='Exact menu name, or All for inventory')
    parser.add_argument('--output', type=Path, help='Default: <project>/build/icon-review/<group>')
    args = parser.parse_args()
    root = args.project_root.resolve()
    output = args.output or root / 'build/icon-review' / re.sub(r'[^A-Za-z0-9_-]', '_', args.group)
    report = audit(root, args.group, output)
    print(json.dumps({k: report[k] for k in ('group', 'entries', 'svg_entries', 'errors', 'shared_geometry')}))
    raise SystemExit(1 if report['errors'] else 0)
