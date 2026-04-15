import { on } from '../services/events.js';

const init = () => {
  on('keydown', '#events-search', (e, el) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      const q = el.value.trim();
      const url = '/admin/events/' + (q ? '?search=' + encodeURIComponent(q) : '');
      window.location.href = url;
    }
  });

  on('click', '.events-row', (e, row) => {
    const id = row.getAttribute('data-event-id');
    const detail = document.querySelector(`[data-detail-for="${id}"]`);
    const chevron = row.querySelector('.expand-chevron');
    if (detail) {
      const isOpen = !detail.hidden;
      detail.hidden = isOpen;
      if (chevron) {
        chevron.classList.toggle('is-expanded', !isOpen);
      }
    }
  });
};

init();
