import { rawResponse, errorMessage } from '/js/services/api.js';
import { showToast } from '/js/services/toast.js';
import { safeStorageGet, safeStorageSet, safeStorageRemove } from '/js/utils/storage-safe.js';

const CLIENT_ID = 'marketplace-admin';
const OAUTH_BASE = '/api/v1/core/oauth';
const LOGIN_PATH = '/admin/login';

export const DEFAULT_REDIRECT = '/admin/profile';

const ALLOWED_REDIRECT_PREFIXES = ['/admin/', '/bridge-auth/'];

export const resolveRedirect = async (target) => {
  if (!target || !ALLOWED_REDIRECT_PREFIXES.some((p) => target.startsWith(p))) {
    return DEFAULT_REDIRECT;
  }
  try {
    const probe = await rawResponse(target, { method: 'HEAD' });
    if (probe.status === 404) return DEFAULT_REDIRECT;
  } catch {
    return target;
  }
  return target;
};

export const exchangeToken = async (code, codeVerifier) => {
  const tokenResponse = await rawResponse(OAUTH_BASE + '/token', {
    method: 'POST',
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
    credentials: 'same-origin',
    body: new URLSearchParams({
      grant_type: 'authorization_code', code,
      redirect_uri: window.location.origin + LOGIN_PATH,
      code_verifier: codeVerifier, client_id: CLIENT_ID,
    }),
  });
  if (!tokenResponse.ok) {
    throw new Error(await errorMessage(tokenResponse) || 'Token exchange failed');
  }
  return tokenResponse.json();
};

export const storeSession = async (tokenData) => {
  const response = await rawResponse('/api/public/auth/session', {
    method: 'POST',
    credentials: 'same-origin',
    body: JSON.stringify({
      access_token: tokenData.access_token,
      expires_in: tokenData.expires_in || 3600
    }),
  });
  if (!response.ok) {
    throw new Error(await errorMessage(response) || 'Failed to store session');
  }
  if (tokenData.refresh_token) safeStorageSet('refresh_token', tokenData.refresh_token);
};

export const completePendingRegistration = async () => {
  const pendingReg = safeStorageGet('pending_registration');
  if (!pendingReg) return;
  try {
    const response = await rawResponse('/api/public/auth/register', {
      method: 'POST',
      credentials: 'same-origin',
      body: pendingReg,
    });
    if (!response.ok) {
      showToast(await errorMessage(response) || 'Registration could not be completed.', 'error');
    }
  } catch {
    showToast('Registration could not be completed. Please try again.', 'error');
  }
  safeStorageRemove('pending_registration');
};
