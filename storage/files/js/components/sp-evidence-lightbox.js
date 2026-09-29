const sheet = new CSSStyleSheet();
sheet.replaceSync(`
  dialog {
    border: none;
    border-radius: var(--sp-radius-lg);
    background: var(--sp-color-surface-variant);
    color: var(--sp-color-text);
    padding: 0;
    max-width: min(96vw, 1200px);
    max-height: 92vh;
    box-shadow: var(--sp-shadow-lg);
  }
  dialog::backdrop {
    background: rgba(0, 0, 0, 0.72);
  }
  figure {
    margin: 0;
    display: flex;
    flex-direction: column;
    max-height: 92vh;
  }
  img {
    display: block;
    max-width: 100%;
    max-height: calc(92vh - 5.5rem);
    object-fit: contain;
    background: var(--sp-color-surface-variant);
    border-radius: var(--sp-radius-lg) var(--sp-radius-lg) 0 0;
  }
  figcaption {
    display: flex;
    align-items: center;
    gap: var(--sp-space-3);
    padding: var(--sp-space-3) var(--sp-space-4);
    font-size: 0.875rem;
    color: var(--sp-color-text-secondary);
  }
  .caption-text { flex: 1; min-width: 0; }
  .counter { white-space: nowrap; color: var(--sp-color-text-muted); }
  button {
    cursor: pointer;
    border: 1px solid var(--sp-color-border);
    border-radius: var(--sp-radius-sm);
    background: var(--sp-color-surface-elevated);
    color: var(--sp-color-text);
    font: inherit;
    line-height: 1;
    padding: var(--sp-space-2) var(--sp-space-3);
  }
  button:hover { background: var(--sp-color-surface); }
  @media (max-width: 48rem) {
    dialog { max-width: 100vw; max-height: 100vh; }
    img { max-height: calc(100vh - 5.5rem); }
  }
`);

const template = document.createElement('template');
template.innerHTML = `
  <dialog aria-label="Screenshot viewer">
    <figure>
      <img alt="" />
      <figcaption>
        <button type="button" data-nav="-1" aria-label="Previous screenshot">&#8592;</button>
        <span class="caption-text"></span>
        <span class="counter"></span>
        <button type="button" data-nav="1" aria-label="Next screenshot">&#8594;</button>
        <button type="button" data-close aria-label="Close">&#10005;</button>
      </figcaption>
    </figure>
  </dialog>
`;

export class SpEvidenceLightbox extends HTMLElement {
  #dialog;
  #img;
  #caption;
  #counter;
  #items = [];
  #index = 0;

  connectedCallback() {
    this.attachShadow({ mode: 'open' });
    this.shadowRoot.adoptedStyleSheets = [sheet];
    this.shadowRoot.append(template.content.cloneNode(true));
    this.#dialog = this.shadowRoot.querySelector('dialog');
    this.#img = this.shadowRoot.querySelector('img');
    this.#caption = this.shadowRoot.querySelector('.caption-text');
    this.#counter = this.shadowRoot.querySelector('.counter');
    this.#dialog.addEventListener('click', (e) => {
      if (e.target === this.#dialog) {
        this.#dialog.close();
      }
    });
    this.#dialog.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowLeft') {
        this.#step(-1);
      } else if (e.key === 'ArrowRight') {
        this.#step(1);
      }
    });
    this.shadowRoot.querySelector('[data-close]').addEventListener('click', () => this.#dialog.close());
    for (const btn of this.shadowRoot.querySelectorAll('[data-nav]')) {
      btn.addEventListener('click', () => this.#step(Number(btn.dataset.nav)));
    }
  }

  setItems(items) {
    this.#items = items;
  }

  openAt(index) {
    this.#index = index;
    this.#render();
    if (this.#dialog.open) {
      this.#dialog.focus();
    } else {
      this.#dialog.showModal();
    }
  }

  #step(delta) {
    const count = this.#items.length;
    this.#index = (this.#index + delta + count) % count;
    this.#render();
  }

  #render() {
    const item = this.#items[this.#index];
    if (item) {
      this.#img.src = item.src;
      this.#img.alt = item.alt;
      this.#caption.textContent = item.alt;
      this.#counter.textContent = `${this.#index + 1} / ${this.#items.length}`;
    }
  }
}

customElements.define('sp-evidence-lightbox', SpEvidenceLightbox);
