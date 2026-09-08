import { onOutsideClick, onEscape } from './events.js';

const TYPEAHEAD_THRESHOLD = 10;

const initTypeahead = (group) => {
  const search = group.querySelector('[data-filter-typeahead]');
  const list = group.querySelector('[data-filter-list]');
  if (!search || !list) return;

  const items = [...list.querySelectorAll('.sp-filter-ribbon__group-item')];
  if (items.length <= TYPEAHEAD_THRESHOLD) {
    search.hidden = true;
    return;
  }

  search.addEventListener('input', () => {
    const query = search.value.trim().toLowerCase();
    for (const item of items) {
      const label = (item.dataset.label || '').toLowerCase();
      item.hidden = query !== '' && !label.includes(query);
    }
  });
};

const initExclusiveOpen = (group, groups) => {
  group.addEventListener('toggle', () => {
    if (!group.open) return;
    for (const other of groups) {
      if (other !== group) other.open = false;
    }
    const search = group.querySelector('[data-filter-typeahead]');
    if (search && !search.hidden) search.focus();
  });
};

const closeAll = (groups) => {
  for (const group of groups) group.open = false;
};

const initRibbon = (ribbon) => {
  const groups = [...ribbon.querySelectorAll('details.sp-filter-ribbon__group')];
  for (const group of groups) {
    initTypeahead(group);
    initExclusiveOpen(group, groups);
  }
  onOutsideClick((e) => {
    if (!ribbon.contains(e.target)) closeAll(groups);
  });
  onEscape(() => {
    const open = groups.find((g) => g.open);
    if (!open) return;
    open.open = false;
    open.querySelector('summary')?.focus();
  });
};

let ribbonsReady = false;

export const initFilterRibbon = () => {
  if (ribbonsReady) return;
  const ribbons = document.querySelectorAll('[data-filter-ribbon]');
  if (!ribbons.length) return;
  ribbonsReady = true;
  for (const ribbon of ribbons) initRibbon(ribbon);
};
