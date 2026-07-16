# Launch systemprompt setup on interactive root login until configured.
if [ -n "${PS1:-}" ] && [ "$(id -u)" = "0" ] && [ ! -f /opt/systemprompt/.configured ]; then
    /opt/systemprompt/setup.sh || true
fi
