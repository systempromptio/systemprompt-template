# Install the gateway via Helm

Deploys the `systemprompt-gateway` server to Kubernetes. For the cowork client CLI, see [../cowork/](../cowork/).

The chart is published to [`charts.systemprompt.io`](https://charts.systemprompt.io) and indexed on [Artifact Hub](https://artifacthub.io/packages/helm/systemprompt/gateway).

## Add the repo

```bash
helm repo add systemprompt https://charts.systemprompt.io
helm repo update
```

## Install (single-replica with embedded Postgres)

```bash
helm install gateway systemprompt/gateway \
  --set secrets.anthropicApiKey=sk-ant-... \
  --set postgresql.auth.password=<strong-pw>
```

## HA install (3 replicas, external Postgres, ingress)

```bash
curl -LO https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/helm/gateway/values-ha.yaml
# edit values-ha.yaml — set externalDatabase.url and ingress.hosts
helm install gateway systemprompt/gateway \
  -f values-ha.yaml \
  --set secrets.anthropicApiKey=sk-ant-...
```

## Key values

| Value | Default | Purpose |
|---|---|---|
| `replicaCount` | `1` | Number of gateway pods |
| `image.repository` | `ghcr.io/systempromptio/systemprompt-template` | Image source |
| `image.tag` | `Chart.appVersion` | Override to pin a version |
| `postgresql.enabled` | `true` | Deploy bundled bitnami Postgres |
| `externalDatabase.url` | — | Required when `postgresql.enabled=false` |
| `secrets.existingSecret` | — | Reference an existing Secret with keys `anthropic`/`openai`/`gemini` |
| `ingress.enabled` | `false` | Enable Ingress |
| `resources.*` | 250m / 512Mi req | Tune per tenant |

Full reference: [`values.yaml` on Artifact Hub](https://artifacthub.io/packages/helm/systemprompt/gateway?modal=values).

## Upgrade

```bash
helm repo update
helm upgrade gateway systemprompt/gateway --reuse-values
```

## Verify the chart signature

Charts are signed via cosign:

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/systempromptio/charts/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature gateway-0.1.0.tgz.sig \
  gateway-0.1.0.tgz
```

## Uninstall

```bash
helm uninstall gateway
```

Docs: https://systemprompt.io/documentation/?utm_source=helm&utm_medium=install_doc
