class SpConfirmDialog extends HTMLElement {
  constructor() {
    super();
    this._resolver = null;
    this._mode = 'confirm';
  }

  connectedCallback() {
    if (this._built) return;
    this._built = true;
    this.className = 'sp-confirm-dialog';
    this.style.display = 'none';
    this.innerHTML = ''
      + '<div class="confirm-overlay" data-role="overlay">'
      + '<div class="confirm-dialog" role="dialog" aria-modal="true">'
      + '<h3 data-role="title"></h3>'
      + '<p data-role="message"></p>'
      + '<input type="text" class="field-input" data-role="input" style="display:none;margin-top:var(--sp-space-3,0.75rem)">'
      + '<div class="confirm-actions" style="display:flex;gap:var(--sp-space-3,0.75rem);justify-content:flex-end;margin-top:var(--sp-space-5,1.25rem)">'
      + '<button type="button" class="btn btn-secondary" data-role="cancel">Cancel</button>'
      + '<button type="button" class="btn btn-danger" data-role="confirm">Confirm</button>'
      + '</div>'
      + '</div>'
      + '</div>';

    this._overlay = this.querySelector('[data-role="overlay"]');
    this._titleEl = this.querySelector('[data-role="title"]');
    this._messageEl = this.querySelector('[data-role="message"]');
    this._inputEl = this.querySelector('[data-role="input"]');
    this._confirmBtn = this.querySelector('[data-role="confirm"]');
    this._cancelBtn = this.querySelector('[data-role="cancel"]');

    this._confirmBtn.addEventListener('click', () => this._resolve(this._mode === 'prompt' ? this._inputEl.value : true));
    this._cancelBtn.addEventListener('click', () => this._resolve(this._mode === 'prompt' ? null : false));
    this._overlay.addEventListener('click', (e) => { if (e.target === this._overlay) this._resolve(this._mode === 'prompt' ? null : false); });
    this.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') this._resolve(this._mode === 'prompt' ? null : false);
      if (e.key === 'Enter' && this._mode === 'prompt') this._resolve(this._inputEl.value);
    });
  }

  _open() {
    this.style.display = 'block';
    this.setAttribute('open', '');
  }

  _close() {
    this.style.display = 'none';
    this.removeAttribute('open');
  }

  _resolve(value) {
    const r = this._resolver;
    this._resolver = null;
    this._close();
    if (r) r(value);
  }

  confirm(title, message, confirmLabel, opts = {}) {
    this.connectedCallback();
    this._mode = 'confirm';
    this._titleEl.textContent = title || 'Confirm';
    this._messageEl.textContent = message || '';
    this._inputEl.style.display = 'none';
    this._confirmBtn.textContent = confirmLabel || 'Confirm';
    this._confirmBtn.className = 'btn ' + (opts.primary ? 'btn-primary' : 'btn-danger');
    this._open();
    this._confirmBtn.focus();
    return new Promise((resolve) => { this._resolver = resolve; });
  }

  prompt(title, message, defaultValue) {
    this.connectedCallback();
    this._mode = 'prompt';
    this._titleEl.textContent = title || 'Input';
    this._messageEl.textContent = message || '';
    this._inputEl.style.display = '';
    this._inputEl.value = defaultValue || '';
    this._confirmBtn.textContent = 'OK';
    this._confirmBtn.className = 'btn btn-primary';
    this._open();
    this._inputEl.focus();
    return new Promise((resolve) => { this._resolver = resolve; });
  }
}

if (!customElements.get('sp-confirm-dialog')) {
  customElements.define('sp-confirm-dialog', SpConfirmDialog);
}
