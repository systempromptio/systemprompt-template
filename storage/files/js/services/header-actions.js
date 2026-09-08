export const initHeaderActions = () => {
  const actions = document.getElementById('sp-topbar__actions');
  if (actions) {
    const toggle = actions.querySelector('.sp-topbar__actions-toggle');
    if (toggle) {
      const close = () => {
        actions.classList.remove('is-open');
        toggle.setAttribute('aria-expanded', 'false');
      };

      toggle.addEventListener('click', (e) => {
        e.stopPropagation();
        const isOpen = actions.classList.contains('is-open');
        if (isOpen) {
          close();
        } else {
          actions.classList.add('is-open');
          toggle.setAttribute('aria-expanded', 'true');
        }
      });

    }
  }
};
