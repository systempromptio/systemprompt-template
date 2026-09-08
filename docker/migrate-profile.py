#!/usr/bin/env python3
"""Preserve pre-0.44 provider configuration across container upgrades."""
import os
from pathlib import Path
import shutil
import sys
import yaml


def write_yaml(path, value):
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(yaml.safe_dump(value, sort_keys=False))
    os.chmod(temporary, 0o600)
    temporary.replace(path)


def migrate(profile_path):
    profile_path = Path(profile_path)
    profile = yaml.safe_load(profile_path.read_text())
    legacy = profile_path.parent / 'legacy-services'
    moved = [key for key in ('providers', 'gateway') if key in profile]
    if moved:
        backup = profile_path.with_name('profile.pre-0.47.yaml')
        if not backup.exists():
            shutil.copy2(profile_path, backup)
        legacy.mkdir(exist_ok=True)
        for key in moved:
            write_yaml(legacy / (key + '.yaml'), {key: profile.pop(key)})
        # Save provider files before removing their only other copy.
        write_yaml(profile_path, profile)
        print('Migrated legacy provider/gateway configuration; original profile retained as profile.pre-0.47.yaml')
    # The container filesystem is replaced at upgrade; the profile volume is not.
    # Reapply the preserved operator configuration on every boot.
    services = Path(profile.get('paths', {}).get('services', '/app/services'))
    for key in ('providers', 'gateway'):
        source = legacy / (key + '.yaml')
        if source.exists():
            destination = services / 'ai' / (key + '.yaml')
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)


if __name__ == '__main__':
    migrate(sys.argv[1])
