import { on } from './events.js';

export const initInstallWidget = () => {
  on('click', '.install-trigger', (e, trigger) => {
    const menu = document.getElementById('install-menu');
    if (menu) {
      const isOpen = menu.classList.contains('open');
      menu.classList.toggle('open', !isOpen);
      trigger.setAttribute('aria-expanded', String(!isOpen));
    }
  }, { exclusive: true });

  on('click', '[data-install-tab]', (e, tabBtn) => {
    const widget = tabBtn.closest('.install-menu');
    if (widget) {
      const tabId = tabBtn.getAttribute('data-install-tab');
      for (const b of widget.querySelectorAll('[data-install-tab]')) {
        b.classList.toggle('active', b === tabBtn);
      }
      for (const p of widget.querySelectorAll('[data-install-panel]')) {
        p.hidden = p.getAttribute('data-install-panel') !== tabId;
      }
    }
  }, { exclusive: true });

  on('click', '[data-copy]', async (e, copyBtn) => {
    const text = copyBtn.getAttribute('data-copy');
    await navigator.clipboard.writeText(text);
    const orig = copyBtn.textContent;
    copyBtn.textContent = '\u2713 Copied';
    setTimeout(() => { copyBtn.textContent = orig; }, 2000);
  }, { exclusive: true });

  on('click', '[data-action="toggle-token"]', (e, btn) => {
    const box = btn.closest('.install-token-box');
    if (!box) return;
    const code = box.querySelector('.install-token-value');
    if (!code) return;
    const masked = code.getAttribute('data-masked') === 'true';
    if (masked) {
      code.style.filter = 'none';
      code.setAttribute('data-masked', 'false');
      btn.title = 'Hide token';
    } else {
      code.style.filter = 'blur(4px)';
      code.setAttribute('data-masked', 'true');
      btn.title = 'Show token';
    }
  }, { exclusive: true });

  // Mask tokens on init
  const tokenEl = document.querySelector('.install-token-value[data-masked="true"]');
  if (tokenEl) tokenEl.style.filter = 'blur(4px)';
};
