import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from update_data import ROOT, REVISION, FILES, build
from refresh_compatibility import resolve_candidate

class UpdateDataTests(unittest.TestCase):
    def test_duplicate_names_resolve_only_with_an_exact_weapon_context(self):
        items=json.loads((ROOT/'research/wardogs-data/items.json').read_text())['items']
        attachments={a['id'].lower():a for a in json.loads((ROOT/'research/wardogs-data/attachments.json').read_text())['attachments']}
        matches=[i for i in items if i.get('name')=='QD-5 Suppressor']
        selected=resolve_candidate(matches,'Id.Item.WEPN_028',attachments)
        self.assertEqual(selected['id'],'Id.Item.MUZL_058')
        self.assertIsNone(resolve_candidate(matches,'Id.Item.AK74M',attachments))
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='update-test-', dir=ROOT / '.local')
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name).resolve()
        self.assertEqual(self.path.parent, (ROOT / '.local').resolve())
        source = ROOT / 'research/wardogs-data'
        for name in ['MANIFEST.json'] + FILES:
            original = source / name.replace('/', '-')
            if not original.exists(): original = source / name.split('/')[-1]
            shutil.copy2(original, self.path / name.replace('/', '-'))
    def rehash(self, name):
        manifest = json.loads((self.path / 'MANIFEST.json').read_text())
        entry = next(e for e in manifest['files'] if e['path'] == 'data/' + name)
        entry['sha256'] = hashlib.sha256((self.path / name).read_bytes()).hexdigest()
        (self.path / 'MANIFEST.json').write_text(json.dumps(manifest))
    def test_complete_counts_without_optional_supplement(self):
        data = build(self.path, REVISION, {})
        self.assertEqual(len(data['weapons']), 40)
        self.assertEqual(len(data['all_attachments']), 298)
    def test_corrupt_download_does_not_touch_active_catalogue(self):
        target = ROOT / 'data/catalog.json'
        before = target.read_bytes()
        with (self.path / 'items.json').open('ab') as stream: stream.write(b' ')
        with self.assertRaisesRegex(ValueError, 'Checksum mismatch'): build(self.path, REVISION, {})
        self.assertEqual(target.read_bytes(), before)
    def test_mixed_build_and_duplicate_ids_fail_validation(self):
        ammo = self.path / 'ammo.json'
        data = json.loads(ammo.read_text())
        data['gameBuild'] = 'another-build'
        ammo.write_text(json.dumps(data))
        self.rehash('ammo.json')
        with self.assertRaisesRegex(ValueError, 'Schema/build mismatch'): build(self.path, REVISION, {})
        data['gameBuild'] = '25937608'
        ammo.write_text(json.dumps(data)); self.rehash('ammo.json')
        path = self.path / 'weapons.json'
        weapons = json.loads(path.read_text()); weapons['weapons'].append(weapons['weapons'][0])
        path.write_text(json.dumps(weapons)); self.rehash('weapons.json')
        with self.assertRaisesRegex(ValueError, 'Duplicate source ID'): build(self.path, REVISION, {})

if __name__ == '__main__': unittest.main()
