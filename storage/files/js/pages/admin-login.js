// The login page's single sign-on outcome: the SAML callback never shows an
// error page of its own, it sends the browser back here with `?sso=<reason>`,
// and this names the reason in words next to a way to try again.
const SSO_MESSAGES = {
  unavailable: 'Single sign-on is not configured on this instance yet. Contact the platform team.',
  denied: 'Sign-in was cancelled or refused by your identity provider.',
  no_subject: 'Your identity provider did not identify your account (no NameID in the assertion). Ask IT to add a Name ID claim to the relying party.',
  invalid_assertion: 'Your identity provider returned a response this platform could not verify. Try again; if it persists, contact the platform team.',
  no_email: 'Your identity provider did not return an email address for your account. Contact the platform team.',
  forbidden: 'Your account is not on an email domain this platform accepts.',
  no_group: 'Your account is not in a directory group that grants access here. Ask IT to add you to the right group.',
  not_provisioned: 'No account exists for you yet. Ask IT to add you to a directory group that grants access here.',
  error: 'Sign-in failed. Please try again.'
};

const params = new URLSearchParams(window.location.search);

const ssoStatus = params.get('sso');
if (ssoStatus) {
  const errEl = document.getElementById('error');
  if (errEl) {
    errEl.textContent = SSO_MESSAGES[ssoStatus] || SSO_MESSAGES.error;
    errEl.hidden = false;
  }
  const retryEl = document.getElementById('retry');
  if (retryEl) {
    retryEl.hidden = false;
  }
}
