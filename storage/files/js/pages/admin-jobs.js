import { on } from '../services/events.js';

let timer = null;

const init = () => {
  on('input', '#job-search', (e, el) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      const q = el.value.toLowerCase().trim();
      for (const row of document.querySelectorAll('.data-table tbody tr')) {
        const name = row.getAttribute('data-name') || row.textContent.toLowerCase();
        row.hidden = q && !name.includes(q);
      }
    }, 200);
  });
};

init();
