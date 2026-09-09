# Railway partner review checklist

This checklist covers Railway template verification. Railway makes the final verification decision.

| Review requirement | Implementation | Verification |
| --- | --- | --- |
| Square icons with transparent backgrounds | `storage/files/images/template-icon.svg` has a square viewBox and no background; each service declares an icon. | Confirm both icons render in the published listing. |
| Official naming | The gateway is `systemprompt.io`; the database is `PostgreSQL`. | Compare the live template with `template.json`. |
| Described variables and generated secrets | Every template variable has a description. PostgreSQL credentials and the OAuth pepper use Railway's `secret()` function. Signing keys and seeds are generated at first boot and persisted. | Deploy without entering internal credentials. |
| Private service communication | The gateway references `${{PostgreSQL.DATABASE_URL}}`; the database connection uses its private hostname. | Inspect resolved hostnames without disclosing credentials. |
| Health checks | Gateway readiness uses `/api/v1/health`, with a 900-second first-boot timeout. | Require a successful deployment and HTTP 200 from health and homepage. |
| Persistent storage | Gateway state mounts at `/app/data`; the Railway PostgreSQL image mounts `/var/lib/postgresql/data`, with PGDATA beneath it. | Recreate the gateway and compare identity, user, audit, and upload data. |
| Authentication | An operator-controlled `ADMIN_EMAIL` is required. Public registration is disabled. Bootstrap passkey enrollment uses a one-time setup token. | Anonymous registration must return 403; protected routes must reject anonymous access. |
| Structured overview | `overview.md` describes hosting, use cases, dependencies, deployment, and licensing. | Publish the overview with the verified configuration. |

## Release procedure

1. Complete the repository release gates and publish the 0.49.0 image.
2. Deploy `template.json` in an isolated Railway project and verify boot, private networking, authentication, and persistence.
3. Apply the tested configuration and overview to the existing published template. Updating listing metadata alone does not update its service configuration.
4. Compare the published template's serialized configuration with the checked-in configuration.
5. Request another partner review only after these checks pass. Include the template URL, release version, and a concise description of the corrections.

References: [Railway template best practices](https://docs.railway.com/templates/best-practices) and [template creation and management](https://docs.railway.com/templates/create).

## Load and check the template

1. Open [Workspace Templates](https://railway.com/workspace/templates), select the workspace that owns the listing, and edit `systempromptio-the-self-owned-ai-control`. The [public deploy page](https://railway.com/deploy/systempromptio-the-self-owned-ai-control) deploys the saved listing; it does not import repository changes.
2. Open the [configuration JSON](https://raw.githubusercontent.com/systempromptio/systemprompt-template/next/deploy/railway/template.json). In the template composer, configure each service from its `source`, `deploy`, `networking`, `variables`, and `volumeMounts` fields. Copy each variable's `defaultValue`, description, and optional setting. This JSON is a configuration reference, not a `railway.json` file; do not paste it into a service's config-as-code setting.
3. Set the template name and gateway service name to `systemprompt.io`, and the database service name to `PostgreSQL`. Use the [square transparent icon](https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/storage/files/images/template-icon.svg) for the template and gateway; use `https://devicons.railway.app/i/postgresql.svg` for PostgreSQL. Paste the [overview Markdown](https://raw.githubusercontent.com/systempromptio/systemprompt-template/next/deploy/railway/overview.md) into the listing overview.
4. In gateway Settings, set the image to `ghcr.io/systempromptio/systemprompt-template:0.49.0`, readiness path to `/api/v1/health`, timeout to `900`, and public HTTP target port to `8080`. Leave the start command unset. Right-click the gateway, select **Attach Volume**, and mount it at `/app/data`. PostgreSQL uses `ghcr.io/railwayapp-templates/postgres-ssl:18` with a volume at `/var/lib/postgresql/data` and `PGDATA=/var/lib/postgresql/data/pgdata`. Keep PostgreSQL private, with no public domain or TCP proxy.
5. Check the template variable expressions before saving: gateway `DATABASE_URL=${{PostgreSQL.DATABASE_URL}}`, `OAUTH_AT_REST_PEPPER=${{secret(64)}}`, `ALLOW_REGISTRATION=false`, `SYSTEMPROMPT_DATA_DIR=/app/data`, `RAILWAY_RUN_UID=0`, `HOST=::`, and `EXTERNAL_URL=https://${{RAILWAY_PUBLIC_DOMAIN}}`. PostgreSQL's password must use the `secret()` expression in the JSON, and its database URL must use `RAILWAY_PRIVATE_DOMAIN`. Leave the administrator email and provider keys for the deployer to supply.
6. Save the template changes. Open the public deploy page above and deploy into a **new project**. Enter an administrator email you control and at least one provider API key. Confirm the preview contains both volumes before deploying. Wait for both services to start; gateway readiness can take up to 15 minutes.
7. Open gateway **Settings → Networking**, copy the generated HTTPS domain, and substitute it below. Both requests must return HTTP 200, and health must report `healthy` and version `0.49.0`:

   ```sh
   GATEWAY_URL='https://YOUR-GENERATED-DOMAIN'
   curl --fail --show-error --include "$GATEWAY_URL/api/v1/health"
   curl --fail --show-error --output /dev/null --write-out '%{http_code}\n' "$GATEWAY_URL/"
   ```

8. Connect to the deployed gateway with Railway SSH (select the new project and gateway service), then run:

   ```sh
   systemprompt admin users webauthn generate-setup-token --email "$ADMIN_EMAIL"
   ```

   Open the returned link privately and enroll a passkey. In an incognito window, confirm protected admin pages require sign-in and do not expose data. Confirm public registration is disabled. Do not include setup tokens, provider keys, or database URLs in review screenshots.
9. Create an identifiable record and upload a small file. Redeploy the gateway, then verify the same passkey, record, and upload still work. Redeploy PostgreSQL and confirm the record remains and the gateway becomes healthy again. Perform these checks in the new verification project.
10. Reopen the saved template and confirm service names, images, variables, health settings, and both volumes match the configuration JSON. Only then request another review with the public template URL and the successful verification results.

The older project is [zooming-cat](https://railway.com/project/84e900c6-3315-4378-a526-7f0739dac5ba). It is not evidence of a successful fresh-template deployment; verify the new project separately.
