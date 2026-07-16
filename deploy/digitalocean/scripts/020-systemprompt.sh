#!/bin/bash
# Bake the systemprompt stack into the image.
set -eux
# Pin the app image tag to the version this image ships.
sed -i "s|ghcr.io/systempromptio/systemprompt-template:.*|ghcr.io/systempromptio/systemprompt-template:${IMAGE_VERSION}|" /opt/systemprompt/docker-compose.yml
# Pre-pull images so droplet first boot is fast.
cd /opt/systemprompt && docker compose pull
chmod +x /opt/systemprompt/setup.sh /etc/update-motd.d/99-one-click \
  /var/lib/cloud/scripts/per-instance/001-systemprompt-firstboot.sh
systemctl daemon-reload
# Not enabled here: setup.sh enables the unit after a provider key exists.
