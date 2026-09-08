#!/usr/bin/env python3
"""Prepare durable container state without masking packaged web assets."""
import base64
import json
import os
from pathlib import Path
import shutil
import sys
import subprocess
import yaml


def attach_state(root, state):
    for relative in ('.systemprompt', 'storage/data', 'storage/files/uploads',
                     'storage/files/images/generated', 'storage/files/audio',
                     'storage/files/video', 'storage/files/documents'):
        source = root / relative
        destination = state / relative
        destination.mkdir(parents=True, exist_ok=True)
        if source.is_symlink():
            if source.resolve() != destination.resolve():
                raise ValueError(f'{source} points outside the configured state directory')
            continue
        if source.exists():
            for item in source.rglob('*'):
                target = destination / item.relative_to(source)
                if item.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                elif not target.exists():
                    shutil.copy2(item, target)
            shutil.rmtree(source)
        source.parent.mkdir(parents=True, exist_ok=True)
        source.symlink_to(destination, target_is_directory=True)


def configure(profile_path, env):
    profile = yaml.safe_load(profile_path.read_text())
    security = profile.setdefault('security', {})
    security['allow_registration'] = env.get('ALLOW_REGISTRATION', 'false').lower() == 'true'
    server = profile.setdefault('server', {})
    server['host'] = env.get('HOST', '0.0.0.0')
    server['port'] = int(env.get('PORT', '8080'))
    external = env.get('EXTERNAL_URL') or env.get('RENDER_EXTERNAL_URL')
    if external:
        server['api_external_url'] = external.rstrip('/')
        server['cors_allowed_origins'] = [external.rstrip('/')]
        security['jwt_issuer'] = external.rstrip('/')
    profile.setdefault('rate_limits', {})['disabled'] = False
    # Existing file-backed signing keys must move with the persistent profile.
    key_path = Path(security.get('signing_key_path', '/app/signing_key.pem'))
    persistent_key = profile_path.parent / 'signing_key.pem'
    if key_path.exists() and not persistent_key.exists():
        shutil.copy2(key_path, persistent_key)
    security['signing_key_path'] = str(persistent_key)
    secret_path = Path(profile['secrets']['secrets_path'])
    if not secret_path.is_absolute():
        secret_path = profile_path.parent / secret_path
    secrets = json.loads(secret_path.read_text())
    if env.get('DATABASE_URL'):
        secrets['database_url'] = env['DATABASE_URL']
    # Platform-generated identity inputs apply once; redeploys retain identity.
    marker = profile_path.parent / '.platform-identity-initialized'
    if not marker.exists() and env.get('PROFILE_CREATED') == 'true':
        for variable, field in [('OAUTH_AT_REST_PEPPER', 'oauth_at_rest_pepper'),
                                ('MANIFEST_SIGNING_SECRET_SEED', 'manifest_signing_secret_seed')]:
            if env.get(variable):
                value = env[variable]
                if field == 'manifest_signing_secret_seed':
                    if len(base64.b64decode(value, validate=True)) != 32:
                        raise ValueError('MANIFEST_SIGNING_SECRET_SEED must encode 32 bytes')
                elif len(value) < 32:
                    raise ValueError('OAUTH_AT_REST_PEPPER must contain at least 32 characters')
                secrets[field] = value
    if not secrets.get('manifest_signing_secret_seed'):
        secrets['manifest_signing_secret_seed'] = base64.b64encode(os.urandom(32)).decode()
    if not secrets.get('signing_key_pem'):
        if not persistent_key.exists():
            pem = subprocess.check_output(['openssl', 'genpkey', '-algorithm', 'RSA',
                                           '-pkeyopt', 'rsa_keygen_bits:2048'], stderr=subprocess.DEVNULL)
            persistent_key.write_bytes(pem)
            persistent_key.chmod(0o600)
        secrets['signing_key_pem'] = base64.b64encode(persistent_key.read_bytes()).decode()
    secret_path.write_text(json.dumps(secrets, indent=2) + '\n')
    secret_path.chmod(0o600)
    marker.touch(mode=0o600)
    profile_path.write_text(yaml.safe_dump(profile, sort_keys=False))
    profile_path.chmod(0o600)


if __name__ == '__main__':
    if sys.argv[1] == 'attach':
        attach_state(Path('/app'), Path(os.environ['SYSTEMPROMPT_DATA_DIR']))
    else:
        configure(Path(sys.argv[1]), os.environ)
