// A software passkey for the sign-in flow the console actually runs.
//
// The console signs people in with WebAuthn only. Chromium's virtual
// authenticator (over CDP) answers navigator.credentials the way a platform
// authenticator would, so a spec can link a passkey through the setup link the
// CLI issues and then sign in through /admin/login with no stubbed endpoint —
// the same flow scripts/lib/console-login.py drives without a browser.
import { execFileSync } from 'node:child_process';
import { join } from 'node:path';
import type { BrowserContext, Page } from '@playwright/test';

const REPO = join(__dirname, '..', '..', '..');
const FLAVOUR = process.env.FLAVOUR ?? 'full';

// E2E_CLI and E2E_PROFILE point at another checkout's binary and profile, for
// the same reason GATEWAY_URL points at another checkout's server.
function cli(): string {
  return process.env.E2E_CLI ?? join(REPO, 'target', `flavour-${FLAVOUR}`, 'debug', 'systemprompt');
}

function profile(): string {
  return process.env.E2E_PROFILE ?? join(REPO, '.systemprompt', 'profiles', FLAVOUR, 'profile.yaml');
}

/** The path and query of a one-time passkey setup link for `email`. */
export function passkeySetupPath(email: string): string {
  const out = execFileSync(
    cli(),
    ['admin', 'users', 'webauthn', 'generate-setup-token', '--email', email],
    {
      env: { ...process.env, SYSTEMPROMPT_PROFILE: profile() },
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    },
  );
  const link = out.match(/https?:\/\/\S*\/auth\/link-passkey\?token=[A-Za-z0-9_-]+/)?.[0];
  if (!link) throw new Error(`no setup link from generate-setup-token for ${email}:\n${out}`);
  const url = new URL(link);
  return `${url.pathname}${url.search}`;
}

/** Give `page` a user-verifying platform authenticator that answers every prompt. */
export async function attachAuthenticator(context: BrowserContext, page: Page): Promise<void> {
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', {
    options: {
      protocol: 'ctap2',
      transport: 'internal',
      hasResidentKey: true,
      hasUserVerification: true,
      isUserVerified: true,
      automaticPresenceSimulation: true,
    },
  });
}
