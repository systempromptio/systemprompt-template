import { on } from '../services/events.js';

const init = () => {
  on('keydown', '#events-search', (e, el) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const q = el.value.trim();
      const url = '/admin/my/activity' + (q ? '?search=' + encodeURIComponent(q) : '');
      window.location.href = url;
    }
  });
};

init();
