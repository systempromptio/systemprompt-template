import {
  makeRequest, preparePublicKeyCredentialCreationOptions, assertRpIdMatchesOrigin
} from '/js/services/webauthn-utils.js?v=3';
import { buildCreationCredentialPayload, WEBAUTHN_BASE } from '/js/services/webauthn-passkey-helpers.js';
import { showLoading, showRegisterError } from '/js/services/webauthn-login-ui.js?v=4';
import { startPasskeyAuth, finishPasskeyAuth, redirectWithPkce } from '/js/services/webauthn-helpers.js?v=3';

const registerBtn = document.getElementById('register-btn');
const emailInput = document.getElementById('register-email');
const nameInput = document.getElementById('register-name');
let isRegistering = false;

const enrollPasskey = async (setupToken) => {
  showLoading('Creating your passkey...');
  const start = await makeRequest(
    WEBAUTHN_BASE + '/link/start?token=' + encodeURIComponent(setupToken), 'GET'
  );
  const publicKey = start.data.challenge.publicKey;
  assertRpIdMatchesOrigin(publicKey.rpId);
  const credential = await navigator.credentials.create({
    publicKey: preparePublicKeyCredentialCreationOptions(publicKey),
  });
  if (!credential) throw new Error('Passkey creation was cancelled');
  await makeRequest(WEBAUTHN_BASE + '/link/finish', 'POST', {
    challenge_id: start.challengeId,
    token: setupToken,
    credential: buildCreationCredentialPayload(credential),
  });
};

const registerWithPasskey = async () => {
  if (isRegistering) return;
  const email = emailInput.value.trim();
  if (!email) {
    showRegisterError('Please enter your work email address.');
    return;
  }
  isRegistering = true;
  registerBtn.disabled = true;
  try {
    showLoading('Creating your account...');
    const { data } = await makeRequest('/admin/auth/passkey/register', 'POST', {
      email,
      display_name: nameInput.value.trim() || null,
    });
    await enrollPasskey(data.setup_token);
    const { startResponse, credential } = await startPasskeyAuth(email);
    const finishResponse = await finishPasskeyAuth(startResponse, credential);
    await redirectWithPkce(finishResponse);
  } catch (error) {
    if (error.name === 'NotAllowedError') {
      showRegisterError('Passkey creation was cancelled or not allowed.');
    } else if (error.name === 'NotSupportedError') {
      showRegisterError('Passkeys are not supported on this device.');
    } else {
      showRegisterError(error.message || 'Registration failed. Please try again.');
    }
  } finally {
    isRegistering = false;
    registerBtn.disabled = false;
  }
};

registerBtn?.addEventListener('click', registerWithPasskey);
[emailInput, nameInput].forEach((input) => input?.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') { e.preventDefault(); registerWithPasskey(); }
}));
