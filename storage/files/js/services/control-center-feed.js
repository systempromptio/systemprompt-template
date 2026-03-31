import { buildSessionCard, updateSessionCard } from './control-center-cards.js';
import { animateReorder, getIsReordering, pushDeferredUpdate } from './cc-feed-reorder.js';

const SORT_INTERVAL_MS = 30000;
let lastSortTime = 0;
let pendingGroups = null;
let sortTimerId = null;
let currentFilter = '';

export const getCurrentFilter = () => currentFilter;
export const setCurrentFilter = (f) => { currentFilter = f; };

export const applyFilter = () => {
  let activeCount = 0;
  let analysedCount = 0;
  let recentCount = 0;
  for (const card of document.querySelectorAll('.sf-session')) {
    const isActive = card.getAttribute('data-is-active') === 'true';
    const status = card.getAttribute('data-status') || 'active';
    const isAnalysed = card.getAttribute('data-is-analysed') === 'true';
    const isVisible = status !== 'deleted';
    if (isVisible) {
      if (isActive) activeCount++;
      if (isAnalysed && !isActive) analysedCount++;
      else recentCount++;
    }
    let show = false;
    if (currentFilter === '') show = isVisible && !(isAnalysed && !isActive);
    else if (currentFilter === 'active') show = isActive && isVisible;
    else if (currentFilter === 'analysed') show = isAnalysed && !isActive && isVisible;
    card.hidden = !show;
  }
  for (const btn of document.querySelectorAll('.cc-filter-btn')) {
    const filter = btn.getAttribute('data-filter') || '';
    if (filter === '') btn.textContent = 'Recent (' + recentCount + ')';
    else if (filter === 'active') btn.textContent = 'Active (' + activeCount + ')';
    else if (filter === 'analysed') btn.textContent = 'Analysed (' + analysedCount + ')';
  }
};

export const reconcileSessionFeed = (groups) => {
  let container = document.getElementById('activity-feed');
  if (!container) {
    const empty = document.querySelector('.cc-empty-section');
    if (empty?.parentNode) {
      container = document.createElement('div');
      container.className = 'sf-feed';
      container.id = 'activity-feed';
      empty.parentNode.replaceChild(container, empty);
    } else { return; }
  }
  const existingCards = {};
  for (const c of container.querySelectorAll('.sf-session[data-session-id]')) {
    existingCards[c.getAttribute('data-session-id')] = c;
  }
  const newIds = {};
  for (let idx = 0; idx < groups.length; idx++) {
    const group = groups[idx];
    const sid = group.session_id;
    newIds[sid] = true;
    if (existingCards[sid]) {
      if (getIsReordering()) {
        const card = existingCards[sid];
        const g = group;
        pushDeferredUpdate(() => updateSessionCard(card, g));
      } else {
        updateSessionCard(existingCards[sid], group);
      }
    } else {
      const card = buildSessionCard(group, idx === 0);
      card.classList.add('sf-session--entering');
      if (container.firstChild) container.insertBefore(card, container.firstChild);
      else container.append(card);
      void card.offsetHeight;
      requestAnimationFrame(() => card.classList.remove('sf-session--entering'));
    }
  }
  for (const sid of Object.keys(existingCards)) {
    if (!newIds[sid]) {
      const stale = existingCards[sid];
      stale.classList.add('sf-session--leaving');
      setTimeout(() => stale.remove(), 350);
    }
  }
  pendingGroups = groups;
  const now = Date.now();
  if (now - lastSortTime >= SORT_INTERVAL_MS) {
    animateReorder(container, groups);
    lastSortTime = now;
    pendingGroups = null;
  } else if (!sortTimerId) {
    sortTimerId = setTimeout(() => {
      if (pendingGroups) {
        const c = document.getElementById('activity-feed');
        if (c) animateReorder(c, pendingGroups);
        lastSortTime = Date.now();
        pendingGroups = null;
      }
      sortTimerId = null;
    }, SORT_INTERVAL_MS - (now - lastSortTime));
  }
  applyFilter();
};
