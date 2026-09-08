export const sheet = new CSSStyleSheet();
sheet.replaceSync(`
  :host { display: none; }
  :host([is-open]) { display: block; }
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--sp-overlay-medium);
    backdrop-filter: blur(4px);
    z-index: var(--sp-z-modal);
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fade-in var(--sp-duration-normal) var(--sp-ease-out);
  }
  .dialog {
    background: var(--sp-bg-surface-overlay);
    border: 1px solid var(--sp-border-default);
    border-radius: var(--sp-radius-card-brand);
    padding: var(--sp-space-7);
    max-width: 400px;
    width: 90%;
    box-shadow: var(--sp-shadow-float);
    color: var(--sp-text-primary);
    animation: dialog-in var(--sp-duration-normal) var(--sp-ease-out);
  }
  h3 { margin: 0 0 var(--sp-space-2); font-size: var(--sp-text-lg); }
  p { margin: 0; color: var(--sp-text-secondary); font-size: var(--sp-text-sm); }
  input {
    width: 100%;
    margin-top: var(--sp-space-3);
    padding: var(--sp-space-2) var(--sp-space-3);
    background: var(--sp-bg-input);
    border: 1px solid var(--sp-border-default);
    border-radius: var(--sp-radius-sm);
    color: var(--sp-text-primary);
    font: inherit;
  }
  input[hidden] { display: none; }
  .actions {
    display: flex;
    gap: var(--sp-space-3);
    justify-content: flex-end;
    margin-top: var(--sp-space-5);
  }
  .btn {
    padding: var(--sp-space-2) var(--sp-space-4);
    border-radius: var(--sp-radius-button);
    border: 1px solid var(--sp-border-default);
    background: var(--sp-bg-surface);
    color: var(--sp-text-primary);
    font: inherit;
    font-weight: 600;
    cursor: pointer;
    transition: background var(--sp-duration-fast) var(--sp-ease-out);
  }
  .btn:hover { background: var(--sp-bg-tertiary); }
  .btn-primary { background: var(--sp-accent); border-color: var(--sp-accent); color: var(--sp-text-on-accent); }
  .btn-primary:hover { background: var(--sp-accent-hover); }
  .btn-danger { background: var(--sp-danger); border-color: var(--sp-danger); color: var(--sp-text-on-accent); }
  .btn-danger:hover { background: color-mix(in oklch, var(--sp-danger) 85%, black); }
  @keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
  @keyframes dialog-in {
    from { opacity: 0; transform: scale(0.95) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }
  @media (prefers-reduced-motion: reduce) {
    .overlay, .dialog { animation-duration: 0.01ms !important; }
  }
`);

export const template = document.createElement('template');
template.innerHTML = `
  <div class="overlay" part="overlay">
    <div class="dialog" role="dialog" aria-modal="true">
      <h3></h3>
      <p></p>
      <input type="text" hidden>
      <div class="actions">
        <button type="button" class="btn" data-role="cancel">Cancel</button>
        <button type="button" class="btn btn-danger" data-role="confirm">Confirm</button>
      </div>
    </div>
  </div>
`;
