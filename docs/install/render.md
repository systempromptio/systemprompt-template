# Deploy the gateway to Render

Deploy [the Blueprint](https://render.com/deploy?repo=https://github.com/systempromptio/systemprompt-template) to provision the published gateway image, a paid web service with a 1 GB persistent disk, and paid PostgreSQL. Hosting and provider usage are billed separately.

Before deploying, supply `ADMIN_EMAIL` (an address you control) and at least one of `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `GEMINI_API_KEY`. Blank provider inputs are ignored. Render generates the OAuth pepper and provides the private database connection. The gateway derives its HTTPS origin from `RENDER_EXTERNAL_URL`.

State is mounted at `/app/data`, selected by `SYSTEMPROMPT_DATA_DIR`. Profiles, signing identity, and uploaded files survive replacement; packaged assets remain supplied by the image. Public registration is disabled. In the service shell, run:

```sh
systemprompt admin users webauthn generate-setup-token --email "$ADMIN_EMAIL"
```

Open the returned private link to enroll your administrator passkey. API clients require access tokens. Health is checked at `/api/v1/health`; first boot runs migrations and prepares web assets.

The image uses release alias `:0`; explicitly redeploy to pull a new release. Back up PostgreSQL and application state before upgrading. Keep one gateway instance when using a local persistent disk. Existing free deployments need a paid plan and disk before they can retain filesystem state; back up their current profile before replacing the instance.
