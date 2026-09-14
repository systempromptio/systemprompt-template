// The transcript renders and reads completely with this file absent: every
// collapsible is a <details>. All this adds is bulk control over them, so the
// buttons stay hidden until it runs rather than sitting there inert.
const CONTROLS = '[data-transcript-controls]';
const SECTIONS =
    '.sp-conversation__tool, .sp-conversation__tool-part, .sp-conversation__system, .sp-conversation__request, .sp-conversation__more';

function setAll(root, open) {
    root.querySelectorAll(SECTIONS).forEach((el) => {
        el.open = open;
    });
}

function enhance(root) {
    const controls = root.querySelector(CONTROLS);
    if (!controls) return;
    controls.hidden = false;
    controls
        .querySelector('[data-transcript-expand]')
        ?.addEventListener('click', () => setAll(root, true));
    controls
        .querySelector('[data-transcript-collapse]')
        ?.addEventListener('click', () => setAll(root, false));
}

function init() {
    document.querySelectorAll('[data-transcript]').forEach(enhance);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
