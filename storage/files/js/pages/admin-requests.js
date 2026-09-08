// The request log's two conveniences, both optional: the whole row opens the
// call it describes, and changing a filter select submits the toolbar. Neither
// is the only way to do its job — every row already carries a real link in its
// time cell, and the form has a real Apply button — so a page with no
// JavaScript loses nothing but a click.

const rowTarget = (event) => {
  const row = event.target.closest('[data-request-row]');
  if (!row) return null;
  // Why: a click that landed on a link or a control already means something.
  if (event.target.closest('a, button, input, select, label')) return null;
  return row.dataset.href || null;
};

const bindRows = () => {
  const table = document.querySelector('[data-request-row]')?.closest('table');
  if (!table) return;
  table.addEventListener('click', (event) => {
    const href = rowTarget(event);
    if (href) window.location.assign(href);
  });
};

const bindToolbar = () => {
  const form = document.querySelector('form.sp-toolbar[role="search"]');
  if (!form) return;
  for (const select of form.querySelectorAll('select')) {
    select.addEventListener('change', () => form.submit());
  }
};

bindRows();
bindToolbar();
