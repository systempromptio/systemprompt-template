const form = document.querySelector('[data-demo-trace="form"]');
const select = document.querySelector('[data-demo-trace="session"]');

// Why: choosing a session is the only control on the form, so the change is
// the submit; the button stays for keyboard users and for scripts disabled.
if (form && select) {
  select.addEventListener('change', () => form.requestSubmit());
}
