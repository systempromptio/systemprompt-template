const SSO_MESSAGES = {
  seat_limit: 'Your organization has used all of its seats. Ask your administrator to free a seat or raise your plan limit.',
  error: 'Sign-in failed. Please try again.'
};

const params = new URLSearchParams(window.location.search);

const ssoStatus = params.get('sso');
if (ssoStatus) {
  const errEl = document.getElementById('error');
  if (errEl) {
    errEl.textContent = SSO_MESSAGES[ssoStatus] || SSO_MESSAGES.error;
    errEl.removeAttribute('hidden');
  }
}
