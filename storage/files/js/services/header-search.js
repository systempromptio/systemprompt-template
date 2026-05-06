// Global header search: ID-only quick jump to trace or request detail page.

const ID_SHAPE = /^[A-Za-z0-9_\-:.]{6,128}$/;

export function initHeaderSearch() {
    const form = document.getElementById('admin-header-search-form');
    const input = document.getElementById('admin-header-search-input');
    const status = document.getElementById('admin-header-search-status');
    if (!form || !input) return;

    form.addEventListener('submit', async (event) => {
        event.preventDefault();
        await runResolve(input, status, form);
    });

    document.addEventListener('keydown', (event) => {
        if (event.key !== '/' || event.metaKey || event.ctrlKey || event.altKey) return;
        const el = document.activeElement;
        if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.isContentEditable)) return;
        event.preventDefault();
        input.focus();
        input.select();
    });

    input.addEventListener('input', () => {
        form.classList.remove('admin-header__search--error');
        if (status) status.textContent = '';
    });
}

async function runResolve(input, status, form) {
    const raw = (input.value || '').trim();
    if (!raw) return;

    if (!ID_SHAPE.test(raw)) {
        showError(form, status, 'Not a valid ID');
        return;
    }

    form.classList.remove('admin-header__search--error');
    form.classList.add('admin-header__search--loading');
    if (status) status.textContent = 'Resolving…';

    try {
        const url = `/admin/api/search/resolve?q=${encodeURIComponent(raw)}`;
        const res = await fetch(url, { headers: { Accept: 'application/json' } });
        if (!res.ok) {
            showError(form, status, 'Lookup failed');
            return;
        }
        const data = await res.json();
        if (data && data.url) {
            window.location.assign(data.url);
            return;
        }
        showError(form, status, 'Not found');
    } catch (_err) {
        showError(form, status, 'Lookup failed');
    } finally {
        form.classList.remove('admin-header__search--loading');
    }
}

function showError(form, status, message) {
    form.classList.add('admin-header__search--error');
    if (status) status.textContent = message;
}
