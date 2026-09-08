#!/usr/bin/env python3
"""Validate catalog documents and required first-boot identity inputs."""
import json
from pathlib import Path
import yaml

root = Path(__file__).resolve().parents[1]
for pattern in ('deploy/**/*.json', 'deploy/**/*.yaml', 'deploy/**/*.yml'):
    for path in root.glob(pattern):
        if path.suffix == '.json':
            json.loads(path.read_text())
        else:
            list(yaml.safe_load_all(path.read_text()))
for name in (
    'render.yaml', 'deploy/casaos/docker-compose.yml',
    'deploy/dokploy/docker-compose.yml', 'deploy/compose/one-click.docker-compose.yml',
    'deploy/digitalocean/files/opt/systemprompt/docker-compose.yml',
    'deploy/caprover/systemprompt.yml', 'deploy/zeabur/template.yaml',
    'deploy/northflank/template.json', 'deploy/portainer/templates.json',
):
    text = (root / name).read_text()
    assert 'ADMIN_EMAIL' in text, f'{name}: missing administrator email input'
    assert 'systemprompt-template:edge' not in text, f'{name}: catalog must use a release image'
compose = json.loads((root / 'deploy/coolify/service-template.json').read_text())['compose']
data = yaml.safe_load(compose)
assert 'ADMIN_EMAIL' in data['services']['gateway']['environment']
print('Deployment documents parse and catalog admin inputs are present')

render = yaml.safe_load((root / 'render.yaml').read_text())
web = render['services'][0]
assert web['plan'] != 'free' and web['disk']['mountPath'] == '/app/data'
assert render['databases'][0]['plan'] != 'free'
assert render['databases'][0]['ipAllowList'] == []
railway = json.loads((root / 'deploy/railway/template.json').read_text())
for service in railway['services'].values():
    assert service.get('icon'), f"{service['name']}: missing icon"
    assert service.get('volumeMounts'), f"{service['name']}: missing persistent volume"
    for key, variable in service['variables'].items():
        assert variable.get('description'), f"{service['name']}.{key}: missing description"
        if key == 'POSTGRES_PASSWORD':
            assert 'secret(' in variable['defaultValue']
    assert 'DATABASE_PUBLIC_URL' not in service['variables']
print('Railway template metadata and Render persistence checks passed')
