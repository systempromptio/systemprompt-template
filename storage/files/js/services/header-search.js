import { rawResponse } from './api.js';
import { onKey } from './events.js';

const ID_SHAPE = /^[A-Za-z0-9_\-:.]{6,128}$/;

const showError = (form, status, message) => {
  form.classList.add('sp-topbar__search--error');
  if (status) status.textContent = message;
};

const resolveId = async (raw) => {
  const url = `/admin/api/search/resolve?q=${encodeURIComponent(raw)}`;
  const res = await rawResponse(url, { headers: { Accept: 'application/json' } });
  if (!res.ok) return null;
  return res.json();
};

const runResolve = async (input, status, form) => {
  const raw = (input.value || '').trim();
  if (!raw) return;

  if (!ID_SHAPE.test(raw)) {
    showError(form, status, 'Not a valid ID');
    return;
  }

  form.classList.remove('sp-topbar__search--error');
  form.classList.add('sp-topbar__search--loading');
  if (status) status.textContent = 'Resolving…';

  try {
    const data = await resolveId(raw);
    if (data && data.url) {
      window.location.assign(data.url);
      return;
    }
    showError(form, status, data ? 'Not found' : 'Lookup failed');
  } catch {
    showError(form, status, 'Lookup failed');
  } finally {
    form.classList.remove('sp-topbar__search--loading');
  }
};

const focusOnSlash = (input) => (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  const el = document.activeElement;
  if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
  event.preventDefault();
  input.focus();
  input.select();
};

let searchReady = false;

export const initHeaderSearch = () => {
  if (searchReady) return;
  const form = document.getElementById('admin-header-search-form');
  const input = document.getElementById('admin-header-search-input');
  const status = document.getElementById('admin-header-search-status');
  if (!form || !input) return;
  searchReady = true;

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    await runResolve(input, status, form);
  });

  onKey('/', focusOnSlash(input));

  input.addEventListener('input', () => {
    form.classList.remove('sp-topbar__search--error');
    if (status) status.textContent = '';
  });
};
