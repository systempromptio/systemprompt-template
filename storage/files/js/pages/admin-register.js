import {
  makeRequest,
  preparePublicKeyCredentialCreationOptions,
  preparePublicKeyCredentialRequestOptions,
} from '../services/webauthn-utils.js';
import {
  buildAuthCredentialPayload,
  buildCreationCredentialPayload,
  initPkceAndRedirect,
  WEBAUTHN_BASE,
} from '../services/webauthn-passkey-helpers.js';
import { rawFetch } from '../services/api.js';
import { hasValidAdminToken } from '../services/admin-token.js';
import { showError, showLoading, showUnsupported, describeError } from './admin-register-ui.js';

const LOGIN_PATH = '/admin/login';
const ADMIN_PATH = '/admin';
const FALLBACK_REDIRECT_MS = 2000;

const autoLogin = async (email) => {
  try {
    showLoading('Signing you in...');
    const start = await makeRequest(
      `${WEBAUTHN_BASE}/auth/start?email=${encodeURIComponent(email)}`,
      'POST',
    );
    const publicKey = preparePublicKeyCredentialRequestOptions(start.data.publicKey);
    const credential = await navigator.credentials.get({ publicKey });
    if (credential) {
      showLoading('Verifying...');
      const finish = await makeRequest(`${WEBAUTHN_BASE}/auth/finish`, 'POST', {
        challenge_id: start.data.challenge_id,
        credential: buildAuthCredentialPayload(credential),
      });
      await initPkceAndRedirect(finish.data.user_id, finish.data.auth_token, showLoading);
    } else {
      throw new Error('Authentication was cancelled');
    }
  } catch (_err) {
    showLoading('Passkey created! Redirecting to sign in...');
    setTimeout(() => {
      window.location.href = LOGIN_PATH;
    }, FALLBACK_REDIRECT_MS);
  }
};

const linkPasskey = async (setupToken) => {
  const start = await makeRequest(
    `${WEBAUTHN_BASE}/link/start?token=${encodeURIComponent(setupToken)}`,
    'GET',
  );
  const source = start.data.challenge ?? start.data;
  const publicKey = preparePublicKeyCredentialCreationOptions(source.publicKey);
  const credential = await navigator.credentials.create({ publicKey });
  if (credential) {
    showLoading('Finishing registration...');
    await makeRequest(`${WEBAUTHN_BASE}/link/finish`, 'POST', {
      challenge_id: start.data.challenge_id ?? start.challengeId,
      token: setupToken,
      credential: buildCreationCredentialPayload(credential),
    });
  } else {
    throw new Error('Passkey creation was cancelled');
  }
};

const createAccount = async (email, displayName, role) => {
  showLoading('Creating your account...');
  const registration = await rawFetch('/admin/api/register', {
    method: 'POST',
    body: JSON.stringify({ name: displayName, email, role }),
  });
  showLoading('Creating your passkey...');
  await linkPasskey(registration.token);
  await autoLogin(email);
};

const handleSubmit = async (event) => {
  event.preventDefault();
  const email = document.getElementById('reg-email').value.trim();
  const displayName = document.getElementById('reg-name').value.trim();
  const role = document.getElementById('reg-role').value;
  if (email && displayName) {
    try {
      await createAccount(email, displayName, role);
    } catch (error) {
      showError(describeError(error));
    }
  } else {
    showError('Please fill in all fields.');
  }
};

const init = () => {
  if (hasValidAdminToken()) {
    window.location.href = ADMIN_PATH;
  } else {
    if (!window.PublicKeyCredential) {
      showUnsupported(
        'Your browser does not support passkeys. Please use a modern browser (Chrome, Firefox, Safari, Edge).',
      );
    }
    document.getElementById('register-form').addEventListener('submit', handleSubmit);
  }
};

init();
