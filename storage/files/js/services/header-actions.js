export const initHeaderActions = () => {
  const actions = document.getElementById('header-actions');
  if (actions) {
    const toggle = actions.querySelector('.header-actions-toggle');
    if (toggle) {
      const close = () => {
        actions.classList.remove('open');
        toggle.setAttribute('aria-expanded', 'false');
      };

      toggle.addEventListener('click', (e) => {
        e.stopPropagation();
        const isOpen = actions.classList.contains('open');
        if (isOpen) {
          close();
        } else {
          actions.classList.add('open');
          toggle.setAttribute('aria-expanded', 'true');
        }
      });

    }
  }
};
