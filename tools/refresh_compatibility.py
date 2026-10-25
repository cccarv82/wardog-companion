"""Cross-check public per-weapon muzzle lists; never infer mounts from caliber."""
import argparse
import time
import hashlib
import json
import re
from update_data import ROOT, REVISION, fetch, read_json, norm

def resolve_candidate(matches, weapon_id, attachments):
    matches = [m for m in matches if m['id'].lower() in attachments]
    if len(matches) > 1:
        exact_context = 'item.attachment.muzzle.' + weapon_id.split('.')[-1].lower()
        matches = [m for m in matches if exact_context in [t.lower() for t in m.get('tags', [])]]
    return matches[0] if len(matches) == 1 else None

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--retry-unavailable', action='store_true', help='Retry only previous 429 responses, preserving verified mappings')
    parser.add_argument('--retry-gaps', action='store_true', help='Recheck only unresolved named attachment joins')
    args = parser.parse_args()
    raw = ROOT / 'research/wardogs-data'
    items = read_json(raw / 'items.json')['items']
    weapons = read_json(raw / 'weapons.json')['weapons']
    attachments = {p['id'].lower(): p for p in read_json(raw / 'attachments.json')['attachments']}
    catalogue = read_json(ROOT / 'research/public-catalog.json')
    script = (ROOT / 'research/catalog.js').read_text(encoding='utf-8-sig')
    match = re.search(r'window.WARDOGS_PROFILE_PATHS=(\{[^\n]+\});', script)
    route_ids = list(json.loads(match[1])) if match else []
    lookup = {norm(p['name']): p['id'] for p in catalogue['weapons']}
    lookup.update({norm(key): key for key in route_ids})
    by_name = {}
    for item in items: by_name.setdefault(norm(item.get('name', '')), []).append(item)
    audit = {'revision': REVISION, 'checked_on': '2026-10-24', 'mapping': {}, 'responses': [], 'excluded': []}
    output = ROOT / 'research/muzzle-compatibility-all.json'
    if output.exists():
        audit = read_json(output)
        if audit['revision'] != REVISION: raise ValueError('Supplement revision mismatch')
    if args.retry_unavailable:
        retry_ids = {e['weapon'].lower() for e in audit['excluded'] if '429' in e['reason']}
        weapons = [w for w in weapons if w['id'].lower() in retry_ids]
    if args.retry_gaps:
        retry_ids = {e['weapon'].lower() for e in audit['excluded'] if 'part' in e}
        weapons = [w for w in weapons if w['id'].lower() in retry_ids]
    audit.setdefault('limitations', [])
    def query(weapon):
        route = lookup.get(norm(weapon.get('name', '')))
        if not route: return weapon, None, None, 'No matching published website weapon'
        url = f'https://wardogsweapons.com/api/weapon/{route}/attachments/MUZZLE'
        try:
            blob = fetch(url)
            return weapon, url, blob, None
        except Exception as error: return weapon, url, None, str(error)
    # Sequential, paced requests; stop on a rate-limit response instead of hammering the service.
    for position, source_weapon in enumerate(weapons):
        if position: time.sleep(3)
        weapon, url, blob, error = query(source_weapon)
        if error:
            if not args.retry_unavailable:
                audit['excluded'].append({'weapon': weapon['id'], 'reason': error})
            if '429' in error:
                print('Service rate limit reached; verified results preserved. Retry later.')
                break
            continue
        audit['excluded'] = [e for e in audit['excluded'] if e['weapon'].lower() != weapon['id'].lower()]
        audit['responses'] = [e for e in audit['responses'] if e['weapon'].lower() != weapon['id'].lower()]
        audit['mapping'].pop(weapon['id'].lower(), None)
        audit['limitations'] = [e for e in audit['limitations'] if e['weapon'].lower() != weapon['id'].lower()]
        response = json.loads(blob)
        if not isinstance(response.get('attachments'), list): raise ValueError('Unexpected endpoint schema')
        audit['responses'].append({'weapon': weapon['id'], 'url': url, 'sha256': hashlib.sha256(blob).hexdigest()})
        for candidate in response['attachments']:
            alias = {'SRD762TI-QD Suppressor': 'SRD762TI-QD'}.get(candidate['name'], candidate['name'])
            item = resolve_candidate(by_name.get(norm(alias), []), weapon['id'], attachments)
            if item is None:
                audit['excluded'].append({'weapon': weapon['id'], 'part': candidate['name'], 'reason': 'Unmatched/ambiguous ID'})
                continue
            part = attachments[item['id'].lower()]
            checks = {'Weapon.VerticalRecoilMultiplier': 'verticalRecoilModifier',
                      'Weapon.HorizontalRecoilMultiplier': 'horizontalRecoilModifier',
                      'Weapon.AimTime': 'aimTimeModifier', 'Weapon.SpreadMultiplier': 'spreadModifier'}
            errors = []
            for attribute, field in checks.items():
                mods = [m for m in part['modifiers'] if m['attribute'] == attribute]
                if len(mods) > 1 or any(m['op'] not in ['multiplyCompound','multiply'] for m in mods):
                    errors.append(attribute)
                    continue
                factor = mods[0]['value'] if mods else 1
                if candidate.get(field) is None or abs((factor-1)*100 - candidate[field]) > .000001:
                    errors.append(attribute)
            if errors:
                audit['excluded'].append({'weapon': weapon['id'], 'part': candidate['name'], 'reason': 'Modifier cross-check failed', 'fields': errors})
                continue
            audit['mapping'].setdefault(weapon['id'].lower(), []).append({'item_id': item['id'], 'url': url})
            if any(m['op'] != 'multiplyCompound' for m in part['modifiers'] if m['attribute'] in checks):
                audit['limitations'].append({'weapon': weapon['id'], 'part': item['name'],
                    'reason': 'Compatibility cross-checked; multiply stacking is not modeled, so automatic suggestions exclude this part'})
    audit['checked_on'] = '2026-10-25'
    output.write_text(json.dumps(audit, indent=2), encoding='utf-8')
    print(json.dumps({'verified_weapons': len(audit['mapping']), 'verified_pairs': sum(map(len, audit['mapping'].values())), 'excluded': len(audit['excluded'])}))

if __name__ == '__main__': main()
