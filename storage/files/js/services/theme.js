const STORAGE_KEY = 'sp-admin-theme';
const DARK = 'dark';
const LIGHT = 'light';

export const getPreferred = () => {
  let stored = null;
  try { stored = localStorage.getItem(STORAGE_KEY); } catch (_e) { }
  if (stored === DARK || stored === LIGHT) return stored;
  if (window.matchMedia?.('(prefers-color-scheme: dark)').matches) return DARK;
  return LIGHT;
};

const apply = (theme) => {
  document.documentElement.style.colorScheme = theme === DARK ? 'dark' : 'light';
  if (theme === DARK) {
    document.documentElement.setAttribute('data-theme', DARK);
  } else {
    document.documentElement.removeAttribute('data-theme');
  }
  updateToggle(theme);
};

const updateToggle = (theme) => {
  const btn = document.getElementById('theme-toggle');
  if (btn) {
    const label = btn.querySelector('.theme-label');
    if (label) label.textContent = theme === DARK ? 'Light mode' : 'Dark mode';
    btn.setAttribute('aria-label', theme === DARK ? 'Switch to light mode' : 'Switch to dark mode');
  }
};

const toggle = () => {
  const current = document.documentElement.getAttribute('data-theme');
  const next = current === DARK ? LIGHT : DARK;
  apply(next);
  try { localStorage.setItem(STORAGE_KEY, next); } catch (_e) { }
};

export const initTheme = () => {
  window.matchMedia?.('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
    let stored = null;
    try { stored = localStorage.getItem(STORAGE_KEY); } catch (_e) { }
    if (!stored) apply(e.matches ? DARK : LIGHT);
  });

  const btn = document.getElementById('theme-toggle');
  btn?.addEventListener('click', toggle);
  apply(getPreferred());
};
