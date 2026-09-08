"""Migration must retain custom settings and survive container replacement."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
import yaml

spec = importlib.util.spec_from_file_location('migrate_profile', Path(__file__).with_name('migrate-profile.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ProfileMigrationTest(unittest.TestCase):
    def test_preserves_custom_settings_and_reapplies_after_replacement(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            profile = root / 'profile.yaml'
            data = {'name': 'docker', 'paths': {'services': str(root / 'services')},
                    'providers': [{'name': 'custom', 'endpoint': 'http://local-model:8000'}],
                    'gateway': {'default_provider': 'custom', 'routes': []}}
            profile.write_text(yaml.safe_dump(data))
            original = profile.read_bytes()
            module.migrate(profile)
            migrated = yaml.safe_load(profile.read_text())
            self.assertNotIn('providers', migrated)
            self.assertNotIn('gateway', migrated)
            self.assertEqual((root / 'profile.pre-0.47.yaml').read_bytes(), original)
            providers = root / 'services/ai/providers.yaml'
            self.assertEqual(yaml.safe_load(providers.read_text())['providers'], data['providers'])
            providers.write_text('providers: []\n')
            module.migrate(profile)
            self.assertEqual(yaml.safe_load(providers.read_text())['providers'], data['providers'])
            self.assertEqual(yaml.safe_load(profile.read_text()), migrated)

    def test_modern_profile_is_unchanged(self):
        with tempfile.TemporaryDirectory() as directory:
            profile = Path(directory) / 'profile.yaml'
            profile.write_text('name: docker\n')
            module.migrate(profile)
            self.assertEqual(profile.read_text(), 'name: docker\n')
            self.assertFalse((profile.parent / 'legacy-services').exists())


if __name__ == '__main__':
    unittest.main()
