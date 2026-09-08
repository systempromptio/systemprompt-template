// Accessible tab strips. Any `[data-tabs]` root with `[role=tab]` buttons that
// name their panel via `aria-controls` gets click and arrow-key switching, and
// the chosen tab is remembered per strip in localStorage.

const STORAGE_PREFIX = 'sp-tabs:';

const remember = (key, value) => {
  try { localStorage.setItem(STORAGE_PREFIX + key, value); } catch { /* storage unavailable */ }
};

const recall = (key) => {
  try { return localStorage.getItem(STORAGE_PREFIX + key); } catch { return null; }
};

export const initTabs = (root) => {
  const tabs = [...root.querySelectorAll('[role="tab"]')];
  if (tabs.length === 0) return null;
  const panelOf = (tab) => root.ownerDocument.getElementById(tab.getAttribute('aria-controls'));

  const select = (name, focus = false) => {
    const target = tabs.find((t) => t.dataset.tab === name) ?? tabs[0];
    for (const tab of tabs) {
      const active = tab === target;
      tab.classList.toggle('sp-tab--active', active);
      tab.setAttribute('aria-selected', active ? 'true' : 'false');
      tab.tabIndex = active ? 0 : -1;
      const panel = panelOf(tab);
      if (panel) panel.hidden = !active;
    }
    if (focus) target.focus();
    if (root.dataset.tabs) remember(root.dataset.tabs, target.dataset.tab);
    return target.dataset.tab;
  };

  for (const tab of tabs) {
    tab.addEventListener('click', () => select(tab.dataset.tab));
    tab.addEventListener('keydown', (event) => {
      const index = tabs.indexOf(tab);
      const next = {
        ArrowRight: (index + 1) % tabs.length,
        ArrowLeft: (index - 1 + tabs.length) % tabs.length,
        Home: 0,
        End: tabs.length - 1,
      }[event.key];
      if (next === undefined) return;
      event.preventDefault();
      select(tabs[next].dataset.tab, true);
    });
  }

  const initial = (root.dataset.tabs && recall(root.dataset.tabs))
    ?? tabs.find((t) => t.getAttribute('aria-selected') === 'true')?.dataset.tab;
  select(initial);
  return { select, tabs };
};

export const initAllTabs = (scope = document) =>
  [...scope.querySelectorAll('[data-tabs]')].map(initTabs).filter(Boolean);

if (typeof document !== 'undefined') initAllTabs();
