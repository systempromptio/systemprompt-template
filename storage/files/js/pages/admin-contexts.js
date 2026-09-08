const form = document.getElementById('contexts-filter-form');
const viewInput = document.getElementById('contexts-view-input');

if (form) {
  form.addEventListener('change', (e) => {
    if (e.target instanceof HTMLElement && e.target.matches('[data-autosubmit]')) {
      form.submit();
    }
  });

  let searchTimer = null;
  const searchInput = form.querySelector('input[name="q"]');
  if (searchInput) {
    searchInput.addEventListener('input', () => {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => form.submit(), 350);
    });
  }
}

for (const tab of document.querySelectorAll('.sp-tabs [data-view]')) {
  tab.addEventListener('click', () => {
    const view = tab.dataset.view;
    if (!view || !viewInput || !form) return;
    viewInput.value = view;
    form.submit();
  });
}

for (const btn of document.querySelectorAll('.sp-table__row-toggle')) {
  btn.addEventListener('click', () => {
    const target = document.getElementById(btn.getAttribute('aria-controls') || '');
    if (!target) return;
    const expanded = btn.getAttribute('aria-expanded') === 'true';
    btn.setAttribute('aria-expanded', expanded ? 'false' : 'true');
    target.hidden = expanded;
  });
}
