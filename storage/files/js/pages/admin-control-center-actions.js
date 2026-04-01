import { updateSessionStatus, analyseSession } from '../services/control-center-stats.js';
import { on } from '../services/events.js';

const closePortal = (state) => {
  state.menu?.remove();
  state.menu = null;
  state.sessionId = null;
};

const openPortal = (btn, state) => {
  closePortal(state);
  const actions = btn.closest('.sf-session-actions');
  if (actions) {
    const menu = btn.nextElementSibling;
    if (menu) {
      const rect = btn.getBoundingClientRect();
      const clone = menu.cloneNode(true);
      clone.className = 'sf-action-menu sf-action-menu--open sf-action-menu--portal';
      clone.style.top = (rect.bottom + 4) + 'px';
      clone.style.right = (window.innerWidth - rect.right) + 'px';
      document.body.append(clone);
      state.menu = clone;
      state.sessionId = actions.getAttribute('data-session-id');
    }
  }
};

export const initActions = () => {
  const state = { menu: null, sessionId: null };

  on('click', '.sf-action-btn', (e, btn) => {
    e.preventDefault();
    e.stopPropagation();
    openPortal(btn, state);
  }, { exclusive: true });

  on('click', '.sf-action-menu--portal .sf-action-item', (e, ai) => {
    e.preventDefault();
    e.stopPropagation();
    if (state.sessionId) {
      const action = ai.getAttribute('data-action');
      if (action === 'analyse') {
        analyseSession(state.sessionId);
      } else {
        updateSessionStatus(state.sessionId, action);
      }
    }
    closePortal(state);
  }, { exclusive: true });

  document.addEventListener('click', (e) => {
    if (state.menu && !e.target.closest('.sf-action-menu--portal') && !e.target.closest('.sf-action-btn')) {
      closePortal(state);
    }
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && state.menu) closePortal(state);
  });

  window.addEventListener('scroll', () => {
    if (state.menu) closePortal(state);
  }, true);
};

export const initSuggestionDetails = () => {
  const overlay = document.getElementById('cc-detail-overlay');
  const content = document.getElementById('cc-detail-content');
  if (overlay && content) {
    const open = (sessionId) => {
      const source = document.getElementById('detail-' + sessionId);
      if (source) {
        content.replaceChildren(...source.cloneNode(true).childNodes);
        overlay.hidden = false;
        overlay.classList.add('open');
        overlay.querySelector('.cc-detail-close')?.focus();
      }
    };

    const close = () => {
      overlay.classList.remove('open');
      overlay.hidden = true;
      content.replaceChildren();
    };

    on('click', '.cc-detail-btn', (e, btn) => {
      e.preventDefault();
      open(btn.getAttribute('data-detail'));
    });

    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) close();
      if (e.target.closest('.cc-detail-close')) close();
    });
  }
};

export const initRatings = (fillStars, rateSession, rateSkill) => {
  for (const c of document.querySelectorAll('.cc-rating')) fillStars(c, parseInt(c.getAttribute('data-current') || '0', 10));
  on('click', '.cc-star', (e, star) => {
    const c = star.closest('.cc-rating');
    if (c) {
      const value = parseInt(star.getAttribute('data-value'), 10);
      fillStars(c, value);
      c.setAttribute('data-current', value);
      const sid = c.getAttribute('data-session-id');
      if (sid) {
        rateSession(sid, value);
        const actions = c.closest('.cc-card-actions') || c.closest('.cc-session-meta');
        const flash = actions?.querySelector('.cc-save-flash');
        if (flash) {
          flash.classList.add('cc-save-flash--visible');
          setTimeout(() => flash.classList.remove('cc-save-flash--visible'), 600);
        }
      }
      else { const sn = c.getAttribute('data-skill-name'); if (sn) rateSkill(sn, value); }
    }
  });
  document.addEventListener('mouseover', (e) => {
    const star = e.target.closest('.cc-star');
    if (star) {
      const c = star.closest('.cc-rating');
      if (c) fillStars(c, parseInt(star.getAttribute('data-value'), 10));
    }
  });
  document.addEventListener('mouseout', (e) => {
    const star = e.target.closest('.cc-star');
    if (star) {
      const c = star.closest('.cc-rating');
      if (c) fillStars(c, parseInt(c.getAttribute('data-current') || '0', 10));
    }
  });
};
