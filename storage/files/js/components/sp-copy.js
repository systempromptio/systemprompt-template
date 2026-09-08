const RESET_MS = 1600;

const ICON_COPY =
    '<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
const ICON_DONE =
    '<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>';

async function writeClipboard(text) {
    if (navigator.clipboard?.writeText) {
        await navigator.clipboard.writeText(text);
        return;
    }
    const scratch = document.createElement('textarea');
    scratch.value = text;
    scratch.setAttribute('readonly', '');
    scratch.style.position = 'fixed';
    scratch.style.opacity = '0';
    document.body.append(scratch);
    scratch.select();
    document.execCommand('copy');
    scratch.remove();
}

function enhance(block) {
    if (block.parentElement?.classList.contains('sp-copy')) return;

    const wrap = document.createElement('div');
    wrap.className = 'sp-copy';
    block.parentNode.insertBefore(wrap, block);
    wrap.append(block);

    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'sp-copy__btn';
    button.innerHTML = ICON_COPY;
    button.setAttribute('aria-label', 'Copy to clipboard');
    wrap.append(button);

    let timer = null;
    button.addEventListener('click', async () => {
        const text = block.textContent.trim();
        if (!text) return;
        try {
            await writeClipboard(text);
            button.innerHTML = ICON_DONE;
            button.classList.add('is-copied');
            button.setAttribute('aria-label', 'Copied');
        } catch {
            button.classList.add('is-failed');
            button.setAttribute('aria-label', 'Copy failed — select the text instead');
            return;
        }
        clearTimeout(timer);
        timer = setTimeout(() => {
            button.innerHTML = ICON_COPY;
            button.classList.remove('is-copied');
            button.setAttribute('aria-label', 'Copy to clipboard');
        }, RESET_MS);
    });
}

function init() {
    document.querySelectorAll('.sp-connect__cmd').forEach(enhance);
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
