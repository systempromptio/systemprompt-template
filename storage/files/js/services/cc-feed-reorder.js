let isReordering = false;
let deferredUpdates = [];

export const getIsReordering = () => isReordering;

export const pushDeferredUpdate = (fn) => {
  deferredUpdates.push(fn);
};

const finishReorder = () => {
  isReordering = false;
  const updates = deferredUpdates.splice(0);
  for (const fn of updates) fn();
};

const reorderCards = (container, groups) => {
  for (let i = 0; i < groups.length; i++) {
    const card = container.querySelector('.sf-session[data-session-id="' + groups[i].session_id_short + '"]:not(.sf-session--leaving)');
    if (card && card !== container.children[i]) container.insertBefore(card, container.children[i]);
  }
};

export const animateReorder = (container, groups) => {
  isReordering = true;
  const selector = '.sf-session[data-session-id]:not(.sf-session--leaving)';
  const cards = container.querySelectorAll(selector);

  const firstRects = {};
  for (const c of cards) firstRects[c.getAttribute('data-session-id')] = c.getBoundingClientRect();

  reorderCards(container, groups);

  const deltas = [];
  for (const c of container.querySelectorAll(selector)) {
    const sid = c.getAttribute('data-session-id');
    const first = firstRects[sid];
    if (!first) continue;
    const deltaY = first.top - c.getBoundingClientRect().top;
    if (Math.abs(deltaY) < 2) continue;
    deltas.push({ card: c, deltaY });
  }

  for (const { card, deltaY } of deltas) {
    card.style.transition = 'none';
    card.style.transform = 'translateY(' + deltaY + 'px)';
  }

  void container.offsetHeight;

  let pending = deltas.length;
  for (const { card } of deltas) {
    card.style.transition = 'transform var(--sp-duration-slow, 0.4s) var(--sp-ease-spring, cubic-bezier(0.25, 0.1, 0.25, 1))';
    card.style.transform = 'translateY(0)';
    card.addEventListener('transitionend', function cleanup(e) {
      if (e.propertyName === 'transform') {
        card.style.transform = '';
        card.style.transition = '';
        card.removeEventListener('transitionend', cleanup);
        if (--pending === 0) finishReorder();
      }
    });
  }

  if (pending === 0) finishReorder();

  setTimeout(() => {
    if (isReordering) finishReorder();
  }, 600);
};
