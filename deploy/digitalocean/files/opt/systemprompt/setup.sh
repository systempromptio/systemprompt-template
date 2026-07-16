#!/bin/bash
# One-time interactive setup for the systemprompt 1-Click droplet.
# Invoked on first root login (via /etc/profile.d/systemprompt-setup.sh),
# re-runnable manually.
set -eu

ENV_FILE=/opt/systemprompt/.env
MARKER=/opt/systemprompt/.configured

if [ -f "$MARKER" ]; then
    echo "systemprompt is already configured. Edit $ENV_FILE and run"
    echo "'systemctl restart systemprompt' to change settings."
    exit 0
fi

echo "systemprompt setup — at least one AI provider API key is required."
read -rp "Anthropic API key (blank to skip): " ANTHROPIC
read -rp "OpenAI API key (blank to skip): " OPENAI
read -rp "Gemini API key (blank to skip): " GEMINI

if [ -z "$ANTHROPIC$OPENAI$GEMINI" ]; then
    echo "No key entered — nothing configured. Re-run: sudo /opt/systemprompt/setup.sh"
    exit 1
fi

sed -i "s|^ANTHROPIC_API_KEY=.*|ANTHROPIC_API_KEY=${ANTHROPIC}|" "$ENV_FILE"
sed -i "s|^OPENAI_API_KEY=.*|OPENAI_API_KEY=${OPENAI}|" "$ENV_FILE"
sed -i "s|^GEMINI_API_KEY=.*|GEMINI_API_KEY=${GEMINI}|" "$ENV_FILE"

touch "$MARKER"
systemctl enable systemprompt >/dev/null 2>&1
systemctl start systemprompt

echo "Starting the gateway (first boot runs migrations — this can take several minutes)..."
EXTERNAL_URL=$(grep '^EXTERNAL_URL=' "$ENV_FILE" | cut -d= -f2-)
for i in $(seq 1 120); do
    if curl -sf http://localhost:8080/api/v1/health >/dev/null; then
        echo
        echo "systemprompt is up: ${EXTERNAL_URL}"
        echo "Point a domain at this droplet and update EXTERNAL_URL in $ENV_FILE"
        echo "(then: systemctl restart systemprompt) to serve it on your own URL."
        exit 0
    fi
    sleep 5
done
echo "The gateway did not become healthy within 10 minutes."
echo "Inspect with: cd /opt/systemprompt && docker compose logs app"
exit 1
