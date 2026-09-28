import { rawResponse } from './api.js';
import { on, onKey, onOutsideClick } from './events.js';
import { createSearchList } from './header-search-list.js';

// Why: every list page copies out a `short_id` — twelve characters and an
// ellipsis — so the box must accept that shape and resolve it as a prefix.
const ID_SHAPE = /^[A-Za-z0-9_\-:.]{4,128}$/;
const MIN_SUGGEST_LEN = 4;
const SUGGEST_DELAY_MS = 150;

const normalise = (raw) => (raw || '').trim().replace(/…+$/u, '').trim();

const showError = (form, status, message) => {
  form.classList.add('sp-topbar__search--error');
  if (status) status.textContent = message;
};

const clearStatus = (form, status) => {
  form.classList.remove('sp-topbar__search--error');
  if (status) status.textContent = '';
};

const resolveId = async (raw) => {
  const url = `/admin/api/search/resolve?q=${encodeURIComponent(raw)}`;
  const res = await rawResponse(url, { headers: { Accept: 'application/json' } });
  if (!res.ok) return null;
  return res.json();
};

const runResolve = async ({ input, status, form, list }) => {
  const raw = normalise(input.value);
  if (!raw) return;
  const picked = list.activeUrl();
  if (picked) {
    window.location.assign(picked);
    return;
  }
  if (!ID_SHAPE.test(raw)) {
    showError(form, status, 'Not a valid ID');
    return;
  }

  clearStatus(form, status);
  form.classList.add('sp-topbar__search--loading');
  if (status) status.textContent = 'Resolving…';

  try {
    const data = await resolveId(raw);
    if (data?.url) {
      window.location.assign(data.url);
      return;
    }
    if (data?.matches?.length) {
      list.show(data.matches);
      if (status) status.textContent = `${data.matches.length} matches — pick one`;
      return;
    }
    showError(form, status, data ? 'Not found' : 'Lookup failed');
  } catch {
    showError(form, status, 'Lookup failed');
  } finally {
    form.classList.remove('sp-topbar__search--loading');
  }
};

const createSuggester = ({ input, list, status }) => {
  let timer = 0;
  let latest = 0;
  const suggest = async () => {
    const raw = normalise(input.value);
    if (raw.length < MIN_SUGGEST_LEN || !ID_SHAPE.test(raw)) {
      list.close();
      return;
    }
    const seq = ++latest;
    try {
      const data = await resolveId(raw);
      if (seq !== latest) return;
      list.show(data?.matches ?? []);
      if (data?.matches?.length === 0 && status) status.textContent = 'No match';
    } catch {
      list.close();
    }
  };
  return () => {
    clearTimeout(timer);
    timer = setTimeout(suggest, SUGGEST_DELAY_MS);
  };
};

const focusOnSlash = (input) => (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey) return;
  const el = document.activeElement;
  if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
  event.preventDefault();
  input.focus();
  input.select();
};

const bindKeys = (input, list) => {
  input.addEventListener('keydown', (event) => {
    if (!list.isOpen()) return;
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault();
      list.move(event.key === 'ArrowDown' ? 1 : -1);
    } else if (event.key === 'Escape') {
      event.stopPropagation();
      list.close();
    }
  });
};

let searchReady = false;

export const initHeaderSearch = () => {
  if (searchReady) return;
  const form = document.getElementById('admin-header-search-form');
  const input = document.getElementById('admin-header-search-input');
  const status = document.getElementById('admin-header-search-status');
  const listEl = document.getElementById('admin-header-search-list');
  const tpl = document.getElementById('admin-header-search-item');
  if (!form || !input || !listEl || !tpl) return;
  searchReady = true;

  const list = createSearchList({ list: listEl, input, tpl });
  const ctx = { input, status, form, list };
  const scheduleSuggest = createSuggester(ctx);

  form.addEventListener('submit', async (event) => {
    event.preventDefault();
    await runResolve(ctx);
  });
  bindKeys(input, list);
  onKey('/', focusOnSlash(input));
  on('click', '[data-action="header-search-pick"]', (e, item) => {
    e.preventDefault();
    window.location.assign(item.dataset.url);
  });
  onOutsideClick((e) => {
    if (!form.contains(e.target)) list.close();
  });

  input.addEventListener('input', () => {
    clearStatus(form, status);
    scheduleSuggest();
  });
};
