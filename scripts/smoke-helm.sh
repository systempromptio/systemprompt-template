#!/usr/bin/env bash
# Disposable kind cluster is provisioned by the caller.
set -euo pipefail
kubectl create deployment postgres --image=postgres:18-alpine
kubectl set env deployment/postgres POSTGRES_USER=test POSTGRES_PASSWORD=test POSTGRES_DB=test
kubectl expose deployment postgres --port=5432
kubectl rollout status deployment/postgres --timeout=180s
helm install gateway helm/gateway \
    --set externalDatabase.url=postgres://test:test@postgres:5432/test \
    --set secrets.anthropicApiKey=sk-ant-smoke-placeholder \
    --set adminEmail=smoke@example.invalid \
    --wait --timeout 600s "$@"
helm test gateway --timeout 120s
kubectl rollout restart deployment/gateway
kubectl rollout status deployment/gateway --timeout=600s
helm test gateway --timeout 120s
