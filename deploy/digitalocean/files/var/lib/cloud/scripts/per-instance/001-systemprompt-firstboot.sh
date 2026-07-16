#!/bin/bash
# Runs once on the first boot of each droplet created from the image.
set -eu

ENV_FILE=/opt/systemprompt/.env
PUBLIC_IP=$(curl -sf http://169.254.169.254/metadata/v1/interfaces/public/0/ipv4/address || hostname -I | awk '{print $1}')

cat > "$ENV_FILE" <<ENV
POSTGRES_PASSWORD=$(openssl rand -hex 24)
EXTERNAL_URL=http://${PUBLIC_IP}:8080
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
GEMINI_API_KEY=
ENV
chmod 600 "$ENV_FILE"

ufw allow 22/tcp
ufw allow 8080/tcp
ufw --force enable

# Warm the images so first-login setup starts fast.
cd /opt/systemprompt && docker compose pull --quiet || true
