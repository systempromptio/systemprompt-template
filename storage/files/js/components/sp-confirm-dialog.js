import { sheet, template } from './sp-confirm-dialog-view.js';

export class SpConfirmDialog extends HTMLElement {
  #resolver = null;
  #mode = 'confirm';
  #refs = {};

  connectedCallback() {
    if (!this.shadowRoot) {
      this.attachShadow({ mode: 'open' });
      this.shadowRoot.adoptedStyleSheets = [sheet];
      this.shadowRoot.append(template.content.cloneNode(true));
      this.#refs = {
        overlay: this.shadowRoot.querySelector('.overlay'),
        title: this.shadowRoot.querySelector('h3'),
        message: this.shadowRoot.querySelector('p'),
        input: this.shadowRoot.querySelector('input'),
        confirm: this.shadowRoot.querySelector('[data-role="confirm"]'),
        cancel: this.shadowRoot.querySelector('[data-role="cancel"]'),
      };
      this.#refs.confirm.addEventListener('click', () => this.#settle(true));
      this.#refs.cancel.addEventListener('click', () => this.#settle(false));
      this.#refs.overlay.addEventListener('pointerdown', (e) => {
        if (e.target === this.#refs.overlay) {
          this.#settle(false);
        }
      });
      this.shadowRoot.addEventListener('keydown', (e) => this.#onKeydown(e));
    }
  }

  #onKeydown(e) {
    if (e.key === 'Escape') {
      this.#settle(false);
    } else if (e.key === 'Enter' && this.#mode === 'prompt') {
      this.#settle(true);
    }
  }

  #settle(accepted) {
    const resolver = this.#resolver;
    this.#resolver = null;
    this.removeAttribute('is-open');
    if (resolver) {
      if (this.#mode === 'prompt') {
        resolver(accepted ? this.#refs.input.value : null);
      } else {
        resolver(accepted);
      }
    }
  }

  #open(title, message) {
    this.connectedCallback();
    this.#refs.title.textContent = title;
    this.#refs.message.textContent = message || '';
    this.setAttribute('is-open', '');
    return new Promise((resolve) => {
      this.#resolver = resolve;
    });
  }

  confirm(title, message, confirmLabel, opts = {}) {
    this.#mode = 'confirm';
    const promise = this.#open(title || 'Confirm', message);
    this.#refs.input.hidden = true;
    this.#refs.confirm.textContent = confirmLabel || 'Confirm';
    this.#refs.confirm.className = opts.primary ? 'btn btn-primary' : 'btn btn-danger';
    this.#refs.confirm.focus();
    return promise;
  }

  prompt(title, message, defaultValue) {
    this.#mode = 'prompt';
    const promise = this.#open(title || 'Input', message);
    this.#refs.input.hidden = false;
    this.#refs.input.value = defaultValue || '';
    this.#refs.confirm.textContent = 'OK';
    this.#refs.confirm.className = 'btn btn-primary';
    this.#refs.input.focus();
    return promise;
  }
}

if (!customElements.get('sp-confirm-dialog')) {
  customElements.define('sp-confirm-dialog', SpConfirmDialog);
}
