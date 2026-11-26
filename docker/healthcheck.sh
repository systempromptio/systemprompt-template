#!/bin/sh
# HEALTHCHECK: a 2xx from /api/v1/health on the local listener. The image has
# no HTTP client, so the request is written over busybox nc.
set -eu
printf 'GET /api/v1/health HTTP/1.0\r\nHost: localhost\r\n\r\n' \
    | timeout 5 nc 127.0.0.1 "${PORT:-8080}" 2>/dev/null \
    | head -n 1 | grep -q '^HTTP/1\.[01] 2'
