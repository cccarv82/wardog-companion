"""Build a validated offline catalogue. Never execute downloaded repository code."""
import argparse
import csv
import hashlib
import io
import json
import math
from pathlib import Path
import re
import urllib.request
from datetime import date

ROOT = Path(__file__).resolve().parents[1]
REVISION = 'e4ca3e9f1c48d980ba531202642884c358208304'
FILES = ['items.json', 'weapons.json', 'attachments.json', 'ammo.json', 'damage.json',
         'equipment.json', 'tags.json', 'csv/weapon-compatibility.csv', 'csv/weapon-damage.csv']
SLOTS = {'Muzzle': 'muzzle', 'Sight': 'optic', 'Underbarrel': 'grip', 'Magazine': 'magazine',
         'Barrel': 'barrel', 'Stock': 'stock', 'Grip': 'pistolgrip', 'Handguard': 'handguard'}
CLASSES = {'AR': 'Assault rifles', 'SMG': 'SMGs', 'LMG': 'LMGs', 'DMR': 'DMRs',
           'Sniper': 'Snipers', 'Pistol': 'Pistols', 'Shotgun': 'Shotguns',
           'Launcher': 'Launchers', 'Bow': 'Bows'}

def read_json(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'WardogCompanion-data-validator/0.8'})
    with urllib.request.urlopen(request, timeout=30) as response:
        blob = response.read(15_000_001)
    if len(blob) > 15_000_000:
        raise ValueError('Source exceeds size limit')
    return blob

def index(rows):
    result = {}
    for row in rows:
        key = row['id'].lower()
        if key in result:
            raise ValueError(f'Duplicate source ID: {key}')
        result[key] = row
    return result

def norm(name):
    return re.sub('[^a-z0-9]', '', name.lower())

def build(raw, revision, supplement):
    manifest = read_json(raw / 'MANIFEST.json')
    hashes = {entry['path']: entry['sha256'] for entry in manifest['files']}
    sources = []
    for name in FILES:
        path = raw / name.replace('/', '-')
        if not path.exists() and name == 'csv/weapon-compatibility.csv':
            path = raw / 'weapon-compatibility.csv'
        blob = path.read_bytes()
        digest = hashlib.sha256(blob).hexdigest()
        if digest != hashes['data/' + name]:
            raise ValueError(f'Checksum mismatch: {name}')
        sources.append({'path': name, 'sha256': digest})
    build_id = manifest['provenance']['gameBuild']
    docs = {}
    for name in FILES:
        if name.endswith('.json'):
            doc = read_json(raw / name)
            if doc.get('schemaVersion') != 1 or doc.get('gameBuild') != build_id:
                raise ValueError(f'Schema/build mismatch: {name}')
            docs[name] = doc
    items = index(docs['items.json']['items'])
    attachments = index(docs['attachments.json']['attachments'])
    weapons = index(docs['weapons.json']['weapons'])
    damage_rows = list(csv.DictReader((raw / 'csv-weapon-damage.csv').read_text(encoding='utf-8-sig').splitlines()))
    compat_path = raw / 'csv-weapon-compatibility.csv'
    if not compat_path.exists(): compat_path = raw / 'weapon-compatibility.csv'
    compatibility = list(csv.DictReader(compat_path.read_text(encoding='utf-8-sig').splitlines()))
    for row in compatibility:
        if row['weapon_id'].lower() not in weapons or row['item_id'].lower() not in items:
            raise ValueError('Broken compatibility link')
    def part_record(key):
        att, item = attachments[key], items[key]
        return dict(id=item['id'], name=item.get('name', item['id']), slot=att['type'], is_default=False,
                    unlock_free=item.get('unlock') is None and not item.get('developer', False), developer=item.get('developer', False),
                    unlock=item.get('unlock'), weight=item.get('weight'),
                    vendor=[{'tag': p['tag'], 'price': p['price']} for p in item.get('vendor', [])],
                    modifiers=att.get('modifiers', []), character_modifiers=att.get('characterModifiers', []),
                    magazine_capacity=att.get('magazine', {}).get('capacity'), compatible_slots=[],
                    conditional=att['type'] == 'Bipod')
    all_parts = [part_record(key) for key in sorted(attachments)]
    supplement = supplement or {}
    refs, issues = [], []
    for key, weapon in weapons.items():
        item = items[key]
        archetype = next((t.split('.')[-1] for t in item.get('tags', []) if t.startswith('Item.Archetype.')), 'Other')
        test = item.get('developer', False) or 'test' in weapon.get('name', '').lower()
        class_name = 'Test / vehicle' if test else CLASSES.get(archetype, archetype)
        parts = {}
        slots = []
        for row in compatibility:
            if row['weapon_id'].lower() != key or row['slot'] not in SLOTS: continue
            part_key = row['item_id'].lower()
            if part_key not in attachments: continue
            slot = SLOTS[row['slot']]
            record = parts.setdefault(part_key, part_record(part_key))
            record['slot'] = row['slot']
            if slot not in record['compatible_slots']: record['compatible_slots'].append(slot)
            record['is_default'] |= row['is_default'].lower() == 'true'
            if slot not in slots: slots.append(slot)
        for candidate in supplement.get(key, []):
            part_key = candidate['item_id'].lower()
            if part_key not in attachments: raise ValueError('Broken supplemental attachment link')
            record = parts.setdefault(part_key, part_record(part_key))
            record['slot'] = 'Muzzle'
            record['compatible_slots'] = ['muzzle']
            record['compatibility_source'] = candidate['url']
            if 'muzzle' not in slots: slots.append('muzzle')
        rows = [r for r in damage_rows if r['weapon_id'].lower() == key]
        for row in rows:
            hit = float(row['damage_per_hit']) if row['damage_per_hit'] else None
            rpm = float(row['rpm']) if row['rpm'] else None
            if hit is not None and rpm is not None and row['raw_dps']:
                if abs(float(row['raw_dps']) - hit * rpm / 60) > .011:
                    raise ValueError('Inconsistent published DPS')
        calibers = weapon.get('calibers', [])
        baseline_id = 'item.caliber.' + calibers[0].lower() if len(calibers) == 1 else None
        baseline = next((r for r in rows if r['projectile'].lower() == baseline_id), None)
        projectile = next((p for p in docs['ammo.json']['projectiles'] if p['id'].lower() == baseline_id), None)
        # Only an unambiguous, nonexplosive single-projectile baseline can enter ideal TTK rankings.
        comparable = bool(baseline and projectile and 'explosive' not in projectile and archetype not in ['Shotgun', 'Launcher', 'Bow'] and not test)
        pattern = weapon.get('recoil', {}).get('pattern', [])
        recoil_index = sum(math.hypot(p['x'], p['y']) for p in pattern[:10]) / 10 if len(pattern) >= 10 else None
        w = dict(id=weapon['id'], name='AK-74' if key == 'id.item.ak74m' else weapon.get('name', weapon['id']),
                 class_name=class_name, test_record=test, unlock=item.get('unlock'), rpm=weapon.get('rpm'),
                 accuracy=weapon.get('accuracy'), ads_time=weapon.get('adsTime'), fire_modes=weapon.get('fireModes', []),
                 recoil_index=recoil_index, damage_per_hit=float(baseline['damage_per_hit']) if baseline else None,
                 unlock_free=item.get('unlock') is None and not test,
                 falloff=weapon.get('damageFalloff', []), recoil_pattern=pattern[:10],
                 projectile=baseline['projectile'] if baseline else None, comparable=comparable,
                 damage_rows=[dict(projectile=r['projectile'], damage=float(r['damage_per_hit']) if r['damage_per_hit'] else None,
                                   raw_dps=float(r['raw_dps']) if r['raw_dps'] else None) for r in rows])
        if not comparable: issues.append({'weapon': w['id'], 'note': 'Ideal TTK unavailable: ambiguous/unsupported projectile baseline or test record'})
        refs.append(dict(schema_version=1, game_build=build_id, local_build_at_import='25937608',
                         imported_on=str(date.today()), source_commit=revision,
                         source_url='https://github.com/warpuppies/wardogs-data', weapon=w,
                         slots=slots, items=sorted(parts.values(), key=lambda p: p['id'].lower())))
    return dict(schema_version=2, game_build=build_id, imported_on=str(date.today()), source_commit=revision,
                source_url='https://github.com/warpuppies/wardogs-data', sources=sources,
                weapons=refs, all_attachments=all_parts, coverage_notes=issues)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--revision', default=REVISION)
    parser.add_argument('--download', action='store_true')
    parser.add_argument('--apply', action='store_true', help='Write catalogue for review/rebuild; does not replace the running app')
    parser.add_argument('--source-dir', type=Path, default=ROOT / 'research/wardogs-data')
    args = parser.parse_args()
    if not re.fullmatch('[0-9a-f]{40}', args.revision): raise ValueError('A pinned 40-character commit is required')
    raw = args.source_dir
    if args.download:
        # New revisions stage separately, preserving the active and previously verified data.
        raw = ROOT / 'research' / ('staged-' + args.revision)
        raw.mkdir(exist_ok=True)
        for name in ['MANIFEST.json'] + FILES:
            url = f'https://raw.githubusercontent.com/warpuppies/wardogs-data/{args.revision}/data/{name}'
            (raw / name.replace('/', '-')).write_bytes(fetch(url))
    elif args.revision != REVISION and raw == ROOT / 'research/wardogs-data':
        raise ValueError('A different revision must be downloaded into staging or supplied with --source-dir')
    supplement_path = ROOT / 'research/muzzle-compatibility-all.json'
    supplement = read_json(supplement_path).get('mapping', {}) if supplement_path.exists() else {}
    if args.revision != REVISION: supplement = {}  # Revalidate external compatibility for a new source revision.
    candidate = build(raw, args.revision, supplement)
    if supplement_path.exists() and args.revision == REVISION:
        audit = read_json(supplement_path)
        for excluded in audit.get('excluded', []):
            candidate['coverage_notes'].append({'weapon': excluded['weapon'], 'note':
                'Supplemental muzzle coverage: ' + excluded.get('part', 'weapon') + ' — ' + excluded['reason']})
        for limitation in audit.get('limitations', []):
            candidate['coverage_notes'].append({'weapon':limitation['weapon'], 'note': limitation['part'] + ' — ' + limitation['reason']})
    json.dumps(candidate, allow_nan=False)  # Validate every generated numeric value before reporting success.
    target = ROOT / 'data/catalog.json'
    previous = read_json(target) if target.exists() else {}
    changed = []
    before = {w['weapon']['id'].lower(): w for w in previous.get('weapons', [])}
    for ref in candidate['weapons']:
        old = before.get(ref['weapon']['id'].lower())
        if old is None or old['weapon'] != ref['weapon'] or old['items'] != ref['items']:
            changed.append(ref['weapon']['id'])
    report = dict(status='validated', applied=args.apply, old_build=previous.get('game_build'),
                  new_build=candidate['game_build'], revision=args.revision, weapon_count=len(candidate['weapons']),
                  attachment_count=len(candidate['all_attachments']), changed_weapons=changed,
                  removed_weapons=sorted(set(before) - {w['weapon']['id'].lower() for w in candidate['weapons']}),
                  notes=candidate['coverage_notes'])
    old_parts = {p['id'].lower(): p for p in previous.get('all_attachments', [])}
    report['changed_attachments'] = [p['id'] for p in candidate['all_attachments'] if old_parts.get(p['id'].lower()) != p]
    report['removed_attachments'] = sorted(set(old_parts) - {p['id'].lower() for p in candidate['all_attachments']})
    (ROOT / 'data/update-report.json').write_text(json.dumps(report, indent=2), encoding='utf-8')
    if args.apply:
        temporary = target.with_suffix('.json.tmp')
        temporary.write_text(json.dumps(candidate, indent=2, allow_nan=False), encoding='utf-8')
        temporary.replace(target)
    print(json.dumps({k: v for k, v in report.items() if k not in ['notes', 'changed_weapons']}, indent=2))

if __name__ == '__main__':
    main()
