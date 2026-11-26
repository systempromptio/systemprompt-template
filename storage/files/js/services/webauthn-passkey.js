import {
  preparePublicKeyCredentialCreationOptions,
  preparePublicKeyCredentialRequestOptions, makeRequest,
  assertRpIdMatchesOrigin
} from '/js/services/webauthn-utils.js?v=4';

import { rawResponse } from '/js/services/api.js';

import {
  buildAuthCredentialPayload, buildCreationCredentialPayload,
  initPkceAndRedirect, WEBAUTHN_BASE
} from '/js/services/webauthn-passkey-helpers.js';

const LOGIN_PATH = '/admin/login';

const errorDiv = document.getElementById('error');
const loadingSection = document.getElementById('loading');
const loadingText = document.getElementById('loading-text');
const retrySection = document.getElementById('retry');
const passkeyForm = document.getElementById('passkey-form');
const userEmailEl = document.getElementById('user-email');
const createBtn = document.getElementById('create-passkey-btn');

let validatedEmail = '';

const showError = (msg) => {
  errorDiv.textContent = msg;
  errorDiv.hidden = false;
  loadingSection.hidden = true;
  passkeyForm.hidden = true;
  retrySection.hidden = false;
};

const showLoading = (msg) => {
  loadingText.textContent = msg || 'Processing...';
  passkeyForm.hidden = true;
  errorDiv.hidden = true;
  loadingSection.hidden = false;
  retrySection.hidden = true;
};

const showPasskeyForm = (email) => {
  userEmailEl.textContent = email;
  passkeyForm.hidden = false;
  loadingSection.hidden = true;
  errorDiv.hidden = true;
  retrySection.hidden = true;
};

const validateToken = async () => {
  const params = new URLSearchParams(window.location.search);
  const token = params.get('token');
  if (!token) {
    showError('No magic link token found. Please request a new magic link from the sign in page.');
  } else {
    try {
      showLoading('Validating link...');
      const response = await rawResponse('/api/public/auth/magic-link/validate', {
        method: 'POST',
        body: JSON.stringify({ token }),
      });
      const data = await response.json();
      if (!response.ok || !data.ok) {
        showError(data.error || 'This link is invalid or has expired. Please request a new one.');
      } else {
        validatedEmail = data.email;
        showPasskeyForm(data.email);
      }
    } catch (err) {
      showError(err.message || 'Failed to validate link.');
    }
  }
};

const autoLogin = async () => {
  try {
    showLoading('Signing you in...');
    const startResponse = await makeRequest(
      WEBAUTHN_BASE + '/auth/start?email=' + encodeURIComponent(validatedEmail), 'POST'
    );
    const publicKeyOptions = preparePublicKeyCredentialRequestOptions(startResponse.data.publicKey);
    const credential = await navigator.credentials.get({ publicKey: publicKeyOptions });
    if (!credential) throw new Error('Authentication was cancelled');
    showLoading('Verifying...');
    const finishResponse = await makeRequest(WEBAUTHN_BASE + '/auth/finish', 'POST', {
      challenge_id: startResponse.data.challenge_id,
      credential: buildAuthCredentialPayload(credential),
    });
    await initPkceAndRedirect(finishResponse.data.user_id, finishResponse.data.auth_token, showLoading);
  } catch {
    showError('Passkey created successfully! Redirecting to sign in...');
    setTimeout(() => { window.location.href = LOGIN_PATH; }, 2000);
  }
};

const showCreationError = (error) => {
  passkeyForm.hidden = false;
  loadingSection.hidden = true;
  if (error.name === 'NotAllowedError') errorDiv.textContent = 'Passkey creation was cancelled or not allowed.';
  else if (error.name === 'NotSupportedError') errorDiv.textContent = 'Passkeys are not supported on this device.';
  else errorDiv.textContent = error.message || 'Failed to create passkey. Please try again.';
  if (error.correctUrl) {
    errorDiv.append(' ');
    const link = document.createElement('a');
    link.href = error.correctUrl;
    link.textContent = 'Continue on ' + new URL(error.correctUrl).hostname;
    errorDiv.append(link);
  }
  errorDiv.hidden = false;
};

const runCreationCeremony = async () => {
  showLoading('Creating your passkey...');
  const username = validatedEmail.split('@')[0];
  const startResponse = await makeRequest(
    WEBAUTHN_BASE + '/link/start?username=' + encodeURIComponent(username) +
    '&email=' + encodeURIComponent(validatedEmail), 'POST'
  );
  assertRpIdMatchesOrigin(startResponse.data.publicKey.rp && startResponse.data.publicKey.rp.id);
  const publicKeyOptions = preparePublicKeyCredentialCreationOptions(startResponse.data.publicKey);
  const credential = await navigator.credentials.create({ publicKey: publicKeyOptions });
  if (!credential) throw new Error('Passkey creation was cancelled');
  showLoading('Finishing setup...');
  await makeRequest(WEBAUTHN_BASE + '/link/finish', 'POST', {
    challenge_id: startResponse.data.challenge_id || startResponse.challengeId,
    email: validatedEmail,
    credential: buildCreationCredentialPayload(credential),
  });
};

const createPasskey = async () => {
  if (!validatedEmail) return;
  try {
    await runCreationCeremony();
    await autoLogin();
  } catch (error) {
    showCreationError(error);
  }
};

createBtn.addEventListener('click', createPasskey);

if (!window.PublicKeyCredential) {
  showError('Your browser does not support passkeys. Please use a modern browser (Chrome, Firefox, Safari, Edge).');
} else {
  validateToken();
}
