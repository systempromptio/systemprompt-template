import { on } from '/js/services/events.js';

on('click', '[data-print-report]', () => {
  window.print();
});
