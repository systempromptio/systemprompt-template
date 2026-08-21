import {
  makeRequest, preparePublicKeyCredentialCreationOptions, assertRpIdMatchesOrigin
} from '/js/services/webauthn-utils.js?v=3';
import { buildCreationCredentialPayload, WEBAUTHN_BASE } from '/js/services/webauthn-passkey-helpers.js';
import { startPasskeyAuth, finishPasskeyAuth, redirectWithPkce } from '/js/services/webauthn-helpers.js?v=3';

const pane = document.getElementById('pane-invite');
const acceptBtn = document.getElementById('accept-btn');
const errorDiv = document.getElementById('error');
const loadingSection = document.getElementById('loading');
const loadingText = document.getElementById('loading-text');
let isAccepting = false;

const showLoading = (msg) => {
  if (loadingText) loadingText.textContent = msg || 'Processing...';
  if (loadingSection) loadingSection.hidden = false;
  if (errorDiv) errorDiv.hidden = true;
};

const showError = (msg) => {
  if (errorDiv) { errorDiv.textContent = msg; errorDiv.hidden = false; }
  if (loadingSection) loadingSection.hidden = true;
};

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

const acceptInvite = async () => {
  if (isAccepting || !pane) return;
  isAccepting = true;
  acceptBtn.disabled = true;
  try {
    showLoading('Setting up your account...');
    const { data } = await makeRequest('/admin/auth/invite/accept', 'POST', {
      token: pane.dataset.inviteToken,
    });
    await enrollPasskey(data.setup_token);
    showLoading('Signing you in...');
    const { startResponse, credential } = await startPasskeyAuth(data.email);
    const finishResponse = await finishPasskeyAuth(startResponse, credential);
    await redirectWithPkce(finishResponse);
  } catch (error) {
    if (error.name === 'NotAllowedError') {
      showError('Passkey creation was cancelled or not allowed.');
    } else if (error.name === 'NotSupportedError') {
      showError('Passkeys are not supported on this device.');
    } else {
      showError(error.message || 'Accepting the invite failed. Please try again.');
    }
  } finally {
    isAccepting = false;
    acceptBtn.disabled = false;
  }
};

acceptBtn?.addEventListener('click', acceptInvite);
