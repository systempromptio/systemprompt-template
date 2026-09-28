import { on } from './events.js';

const STORAGE_KEY = 'sp-nav:open-groups';

const readOpen = () => {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return new Set(raw ? JSON.parse(raw) : []);
  } catch {
    return new Set();
  }
};

const writeOpen = (open) => {
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify([...open])); } catch { /* storage unavailable */ }
};

const setGroupOpen = (group, isOpen) => {
  group.classList.toggle('is-open', isOpen);
  group.querySelector('[data-action="nav-group-toggle"]')?.setAttribute('aria-expanded', String(isOpen));
};

// Why: the group holding the current page is always open and cannot be
// collapsed — a rail that hides where you are is a rail you have to search.
// The reader's choices for the other groups are honoured from storage.
const holdsCurrent = (group) => group.querySelector('a.is-active, a.is-ancestor') !== null;

const applyState = (nav, open) => {
  for (const group of nav.querySelectorAll('[data-nav-group]')) {
    setGroupOpen(group, holdsCurrent(group) || open.has(group.dataset.navGroup));
    const count = group.querySelector('[data-nav-group-count]');
    if (count) count.textContent = String(group.querySelectorAll('.sp-nav-group__inner > a').length);
  }
  nav.dataset.navReady = 'true';
};

const toggleGroup = (btn) => {
  const group = btn.closest('[data-nav-group]');
  if (!group || holdsCurrent(group)) return;
  const isOpen = !group.classList.contains('is-open');
  setGroupOpen(group, isOpen);
  const open = readOpen();
  if (isOpen) {
    open.add(group.dataset.navGroup);
  } else {
    open.delete(group.dataset.navGroup);
  }
  writeOpen(open);
};

export const initNavGroups = () => {
  const nav = document.querySelector('#sp-admin-sidebar nav[data-nav]');
  if (!nav) return;
  applyState(nav, readOpen());
  // Why: groups restored from storage must land open, not animate open. A
  // single frame callback runs before that frame's style pass, so the open
  // state and the lifted gate would land together and still animate; the
  // second frame is the first one painted with the state already applied.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => { nav.dataset.navMotion = 'true'; });
  });

  on('click', '[data-action="nav-group-toggle"]', (e, btn) => {
    e.preventDefault();
    toggleGroup(btn);
  });
};
