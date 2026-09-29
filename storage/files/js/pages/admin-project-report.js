// The project report: one printable page. The only behaviour is handing the
// page to the browser's print engine, which is where "Save as PDF" lives on
// every platform; `data-print-on-load` is the listing's Report action asking
// for that dialog as soon as the page has painted.
import { on, initDelegation } from '../services/events.js';

const print = () => window.print();

export const init = () => {
  initDelegation();
  on('click', '[data-action="save-pdf"]', print);
  if (document.querySelector('[data-print-on-load]')) {
    window.requestAnimationFrame(() => window.setTimeout(print, 150));
  }
};

init();
