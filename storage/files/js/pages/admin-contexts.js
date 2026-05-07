// /admin/entities/contexts — submit filters on change, toggle "By user" tabs,
// and expand per-user nested rows.

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

document.querySelectorAll('.tabs [data-view]').forEach((tab) => {
  tab.addEventListener('click', () => {
    const view = tab.dataset.view;
    if (!view || !viewInput || !form) return;
    viewInput.value = view;
    form.submit();
  });
});

document.querySelectorAll('.row-expand-toggle').forEach((btn) => {
  btn.addEventListener('click', () => {
    const id = btn.getAttribute('aria-controls');
    if (!id) return;
    const target = document.getElementById(id);
    if (!target) return;
    const expanded = btn.getAttribute('aria-expanded') === 'true';
    btn.setAttribute('aria-expanded', expanded ? 'false' : 'true');
    target.hidden = expanded;
    btn.style.transform = expanded ? '' : 'rotate(180deg)';
  });
});
