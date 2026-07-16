#!/bin/bash
# Marketplace image hygiene, per digitalocean/marketplace-partners img_check.
set -eux
apt-get -y autoremove
apt-get -y autoclean
rm -rf /tmp/* /var/tmp/*
rm -f /root/.bash_history /home/*/.bash_history
rm -f /etc/ssh/ssh_host_*
truncate -s 0 /var/log/*.log 2>/dev/null || true
cloud-init clean --logs || true
# Recommended: run the official img_check.sh from
# github.com/digitalocean/marketplace-partners before submitting.
