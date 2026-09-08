// The AI-activity scope contract: `group`, `project`, `user_id` and `preset`
// are the four parameters every page in that group reads, and someone who set
// them on one page means them on the next. The server renders each page for the
// parameters it was given; this module is what carries them across, and what
// shows the reader which ones are in force.

const SCOPE_KEYS = ['group', 'project', 'user_id'];
const RANGE_KEY = 'preset';

const CHIP_LABELS = {
  group: 'Group',
  project: 'Project',
  user_id: 'User'
};

const currentParams = () => new URLSearchParams(window.location.search);

const scopeEntries = (params) =>
  SCOPE_KEYS.map((key) => [key, params.get(key)]).filter(([, value]) => value);

// Why: only the AI-activity links carry the scope. Marking them in the sidebar
// rather than matching on path keeps the group's membership in one place — the
// template that draws it.
const carryScopeIntoNav = (params) => {
  const entries = [...scopeEntries(params)];
  const preset = params.get(RANGE_KEY);
  if (preset) entries.push([RANGE_KEY, preset]);
  if (entries.length === 0) return;
  for (const link of document.querySelectorAll('a[data-scope-nav]')) {
    const url = new URL(link.getAttribute('href'), window.location.origin);
    for (const [key, value] of entries) url.searchParams.set(key, value);
    link.href = `${url.pathname}${url.search}`;
  }
};

// The range links are rendered as `?preset=…` alone so they work with no JS.
// Here they regain the rest of the current query, and the active one is marked.
const bindRanges = (params) => {
  const active = params.get(RANGE_KEY);
  for (const link of document.querySelectorAll('[data-scope-range]')) {
    const range = link.dataset.scopeRange;
    const url = new URL(window.location.href);
    url.searchParams.set(RANGE_KEY, range);
    url.searchParams.delete('page');
    link.href = `${url.pathname}${url.search}`;
    link.classList.toggle('sp-scope-filter__range--active', range === active);
    if (range === active) {
      link.setAttribute('aria-current', 'true');
    } else {
      link.removeAttribute('aria-current');
    }
  }
};

const chip = (key, value, params) => {
  const url = new URL(window.location.href);
  url.searchParams.delete(key);
  url.searchParams.delete('page');
  const link = document.createElement('a');
  link.className = 'sp-scope-filter__chip';
  link.href = `${url.pathname}${url.search}`;
  link.textContent = `${CHIP_LABELS[key]}: ${value}`;
  link.setAttribute('aria-label', `Remove the ${CHIP_LABELS[key].toLowerCase()} filter ${value}`);
  return link;
};

// Why: the chips are the only thing on the page that says a listing is showing
// part of the data. A filter carried in from another page with nothing on
// screen to say so is how people misread a narrowed total as the whole.
const drawChips = (bar, params) => {
  const list = bar.querySelector('[data-scope-chip-list]');
  const wrapper = bar.querySelector('[data-scope-chips]');
  const entries = scopeEntries(params);
  if (!list || !wrapper) return;
  list.replaceChildren(...entries.map(([key, value]) => chip(key, value, params)));
  wrapper.hidden = entries.length === 0;
  const clear = bar.querySelector('[data-scope-clear]');
  if (clear) {
    const preset = params.get(RANGE_KEY);
    clear.href = preset ? `?${RANGE_KEY}=${encodeURIComponent(preset)}` : window.location.pathname;
  }
};

export const initScope = () => {
  const params = currentParams();
  carryScopeIntoNav(params);
  const bar = document.querySelector('[data-scope-selector]');
  if (!bar) return;
  bindRanges(params);
  drawChips(bar, params);
};
