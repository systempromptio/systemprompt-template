const SSO_MESSAGES = {
  unavailable: 'Single sign-on is not configured on this instance yet. Contact the platform team.',
  denied: 'Sign-in was cancelled or refused by Astound SSO.',
  no_subject: 'Astound SSO did not identify your account (no NameID in the assertion). IT needs to add a Name ID claim to the relying party.',
  invalid_assertion: 'Astound SSO returned a response this platform could not verify. Try again; if it persists, contact the platform team.',
  no_email: 'Astound SSO did not return an email address for your account. Contact the platform team.',
  forbidden: 'Your account is not on an email domain this platform accepts.',
  no_group: 'Your account is not in a Systemprompt-* AD group, so it has no access here. Ask IT to add you to the right group.',
  not_provisioned: 'No account exists for you yet. Ask IT to add you to a Systemprompt-* AD group.',
  no_project: 'Your account is not in a project AD group (Systemprompt-Commerce or Systemprompt-Core). Ask IT to add you to one.',
  ambiguous_project: 'Your account is in more than one project AD group. Ask IT to leave you in exactly one of Systemprompt-Commerce or Systemprompt-Core.',
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
