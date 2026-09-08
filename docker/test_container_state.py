#!/usr/bin/env python3
"""Container replacement must retain operator data and authentication identity."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import yaml

spec = importlib.util.spec_from_file_location('state', Path(__file__).with_name('container-state.py'))
state = importlib.util.module_from_spec(spec)
spec.loader.exec_module(state)


class ContainerStateTest(unittest.TestCase):
    def test_replacement_preserves_uploads_and_profiles(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'app'
            data = Path(directory) / 'persistent'
            upload = root / 'storage/files/uploads/user/file.txt'
            upload.parent.mkdir(parents=True)
            upload.write_text('operator upload')
            asset = root / 'storage/files/css/admin.css'
            asset.parent.mkdir(parents=True)
            asset.write_text('new image assets')
            state.attach_state(root, data)
            state.attach_state(root, data)
            self.assertEqual(upload.read_text(), 'operator upload')
            self.assertEqual(asset.read_text(), 'new image assets')
            self.assertTrue((root / '.systemprompt').is_symlink())

    def test_profile_redeploy_retains_identity_and_updates_url(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            secret = root / 'secrets.json'
            secret.write_text(json.dumps({'oauth_at_rest_pepper': 'a' * 32}))
            key = root / 'old-key.pem'
            key.write_text('existing signing key')
            profile = root / 'profile.yaml'
            profile.write_text(yaml.safe_dump({'secrets': {'secrets_path': str(secret)},
                                             'security': {'signing_key_path': str(key)}}))
            env = {'EXTERNAL_URL': 'https://example.com', 'HOST': '::', 'PORT': '8080'}
            state.configure(profile, env)
            state.configure(profile, dict(env, OAUTH_AT_REST_PEPPER='b' * 32))
            result = yaml.safe_load(profile.read_text())
            self.assertFalse(result['security']['allow_registration'])
            self.assertEqual(result['security']['issuer'], 'https://example.com')
            self.assertEqual((root / 'signing_key.pem').read_text(), 'existing signing key')
            self.assertEqual(json.loads(secret.read_text())['oauth_at_rest_pepper'], 'a' * 32)
            self.assertEqual(secret.stat().st_mode & 0o777, 0o600)


if __name__ == '__main__':
    unittest.main()
