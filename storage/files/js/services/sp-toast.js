'use strict';

const ICONS = { success: '\u2713', error: '\u2717', info: '\u24D8', warning: '\u26A0' };

class SpToast extends HTMLElement {
  connectedCallback() {
    if (!this._container) {
      this._container = document.createElement('div');
      this._container.className = 'toast-container';
      this.append(this._container);
    }
  }

  show(message, type = 'info') {
    if (!this._container) this.connectedCallback();
    const icon = ICONS[type] || ICONS.info;
    const el = document.createElement('div');
    el.className = 'toast toast-' + type;
    el.style.animation = 'toastIn 350ms var(--sp-ease-out, ease-out) forwards';
    const iconSpan = document.createElement('span');
    iconSpan.className = 'toast-icon';
    iconSpan.textContent = icon;
    const msgSpan = document.createElement('span');
    msgSpan.className = 'toast-message';
    msgSpan.textContent = message;
    el.append(iconSpan, msgSpan);
    this._container.append(el);
    setTimeout(() => {
      el.classList.add('toast--leaving');
      setTimeout(() => el.remove(), 350);
    }, 4000);
  }
}

if (!customElements.get('sp-toast')) {
  customElements.define('sp-toast', SpToast);
}
