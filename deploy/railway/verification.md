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

1. Complete the repository release gates and publish the 0.48.0 image.
2. Deploy `template.json` in an isolated Railway project and verify boot, private networking, authentication, and persistence.
3. Apply the tested configuration and overview to the existing published template. Updating listing metadata alone does not update its service configuration.
4. Compare the published template's serialized configuration with the checked-in configuration.
5. Request another partner review only after these checks pass. Include the template URL, release version, and a concise description of the corrections.

References: [Railway template best practices](https://docs.railway.com/templates/best-practices) and [template creation and management](https://docs.railway.com/templates/create).
