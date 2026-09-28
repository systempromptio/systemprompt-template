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
