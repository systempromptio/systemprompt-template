import { errorMessage, rawResponse } from '/js/services/api.js';
import { showConfirmDialog } from '/js/services/confirm.js';
// Account state comes only from the server. Credentials are submitted once and
// never kept in browser storage, URL parameters, or rendered response payloads.
const root = document.querySelector('[data-connected-accounts]');
const labels = { verification_required: 'Authorization saved — verification required', not_configured: 'Not configured', not_connected: 'Not connected', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
const names = { atlassian: 'Atlassian', github: 'GitHub', salesforce: 'Salesforce' };
let revision = -1;
let sequence = 0;
let pending = false;
let lastChecked = '';
function element(tag, text) { const node = document.createElement(tag); if (text) node.textContent = text; return node; }
function message(text, tone = '') {
    const node = root.querySelector('[data-connection-message]');
    node.textContent = text; node.hidden = !text; node.className = `sp-notice sp-connection__message${tone ? ` sp-notice--${tone}` : ''}`;
}
const tones = { connected: 'ok', reconnect_required: 'warn', verification_required: 'warn', temporarily_unavailable: 'err' };
const actionLabels = { connect: 'Connect', reconnect: 'Reconnect', disconnect: 'Disconnect', test: 'Test', manual_token: 'Use personal token' };
function relative(iso) {
    const minutes = Math.round((Date.now() - new Date(iso).getTime()) / 60000);
    if (minutes < 1) return 'verified just now';
    if (minutes < 60) return `verified ${minutes} min ago`;
    if (minutes < 60 * 24) return `verified ${Math.round(minutes / 60)} h ago`;
    return `verified ${new Date(iso).toLocaleDateString()}`;
}
function host(value) { try { return new URL(value).host; } catch (_) { return value; } }
function meta(account) {
    const line = element('div'); line.className = 'sp-connection__meta';
    if (!account.entitled) { line.append(element('span', 'India Development marketplace access required')); return line; }
    if (account.account_name) { const id = element('span', account.account_name); id.className = 'sp-u-mono'; id.title = account.account_name; line.append(id); }
    if (account.resource_name) { const site = element('span', host(account.resource_name)); site.title = account.resource_name; line.append(site); }
    if (account.verified_at) { const when = element('span', relative(account.verified_at)); when.title = new Date(account.verified_at).toLocaleString(); line.append(when); }
    if (!line.childElementCount) line.append(element('span', account.configured ? 'Authorize once for every enrolled machine' : 'Provider setup required'));
    return line;
}
function variant(action, status) {
    if (['connect', 'reconnect'].includes(action)) return status === 'connected' ? 'sp-btn--outline' : 'sp-btn--primary';
    if (action === 'disconnect') return 'sp-btn--outline-danger';
    if (action === 'test') return 'sp-btn--outline';
    return 'sp-btn--ghost';
}
function actionsFor(account, user) {
    const actions = element('div'); actions.className = 'sp-connection__actions';
    const order = ['connect', 'reconnect', 'test', 'disconnect', 'manual_token'];
    for (const action of order.filter(name => account.actions.includes(name))) {
        const button = element('button', actionLabels[action]); button.type = 'button'; button.className = `sp-btn sp-btn--sm ${variant(action, account.status)}`;
        button.addEventListener('click', () => perform(account.provider, action, user)); actions.append(button);
    }
    return actions;
}
function render(data) {
    const expected = new URLSearchParams(window.location.search).get('expected_user');
    if (expected && expected !== data.user_id) { root.querySelector('[data-connection-list]').replaceChildren(); message('Sign in to the same Systemprompt account as your bridge.', 'err'); return; }
    if (data.version !== 1) { message('Update the server and bridge to manage connected accounts.', 'err'); return; }
    if (data.revision < revision) return;
    revision = data.revision;
    const list = root.querySelector('[data-connection-list]');
    list.replaceChildren();
    const connected = data.connections.filter(account => account.status === 'connected').length;
    root.querySelector('[data-connection-count]').textContent = `${connected} of ${data.connections.length} connected`;
    for (const account of data.connections) {
        const row = element('li'); row.className = 'sp-connection'; row.id = `connection-${account.provider}`; row.dataset.status = account.status;
        const mark = element('span', names[account.provider].slice(0, 1)); mark.className = 'sp-connection__mark'; mark.setAttribute('aria-hidden', 'true');
        const body = element('div'); body.className = 'sp-connection__body';
        const name = element('h3', names[account.provider]); name.className = 'sp-connection__name';
        body.append(name, meta(account));
        const badge = element('span', labels[account.status] || 'Unavailable'); badge.className = `sp-badge sp-connection__status sp-badge--${tones[account.status] || 'muted'}`;
        row.append(mark, body, badge, actionsFor(account, data.user_id)); list.append(row);
    }
}
async function refresh() {
    if (!root || pending || root.querySelector('form')) return;
    const request = ++sequence;
    try {
        const response = await rawResponse('/api/public/account/connections', { credentials: 'same-origin', cache: 'no-store' });
        if (response.status === 401) {
            root.querySelectorAll('button').forEach(button => { button.disabled = true; });
            message('Your login session is no longer active. Sign in again to connect accounts.', 'err');
            const link = element('a', 'Sign in again'); link.href = '/admin/login'; link.className = 'sp-btn sp-btn--sm sp-btn--primary';
            root.querySelector('[data-connection-message]').append(' ', link); return;
        }
        if (!response.ok) throw new Error(await errorMessage(response));
        const data = await response.json();
        if (request !== sequence) return;
        render(data); lastChecked = new Date().toLocaleTimeString(); message('');
    } catch (error) { message(`Last known state${lastChecked ? ` (${lastChecked})` : ''}. ${error.message}`, 'warn'); }
}
function field(form, label, name, type = 'text') {
    const wrapper = element('label', label); const input = element('input'); input.name = name; input.type = type; input.className = 'sp-field-input';
    input.autocomplete = 'off'; wrapper.append(input); form.append(wrapper); return input;
}
function formFor(provider, action, user) {
    const card = root.querySelector(`#connection-${provider}`);
    if (card.querySelector('form')) return;
    const form = element('form'); form.className = 'sp-connection__form';
    if (action === 'manual_token') {
        field(form, 'Personal token', 'token', 'password').required = true;
        if (provider === 'atlassian') field(form, 'Atlassian account email', 'email', 'email').required = true;
    }
    if (provider === 'atlassian') {
        const site = field(form, 'Atlassian site address', 'resource_id', 'url');
        site.placeholder = 'https://your-team.atlassian.net'; site.required = true;
    }
    const actions = element('div'); actions.className = 'sp-connection__actions'; form.append(actions);
    const submit = element('button', action === 'manual_token' ? 'Verify and save' : 'Continue to Atlassian'); submit.type = 'submit'; submit.className = 'sp-btn sp-btn--sm sp-btn--primary'; actions.append(submit);
    const cancel = element('button', 'Cancel'); cancel.type = 'button'; cancel.className = 'sp-btn sp-btn--sm sp-btn--ghost'; cancel.addEventListener('click', () => form.remove()); actions.append(cancel);
    form.addEventListener('submit', async (event) => {
        event.preventDefault(); const values = Object.fromEntries(new FormData(form));
        if (action !== 'manual_token') { connect(provider, user, values.resource_id); return; }
        await mutate(provider, 'manual-token', values); form.reset(); form.remove(); await refresh();
    }); card.append(form);
}
function connect(provider, user, resource = '') {
    const url = new URL(`/api/public/connectors/${provider}/start`, window.location.origin);
    url.searchParams.set('expected_user', user);
    if (resource) url.searchParams.set('resource_id', resource);
    window.location.assign(url);
}
async function mutate(provider, action, body) {
    pending = true; ++sequence;
    try {
        const response = await rawResponse(`/api/public/account/connections/${provider}/${action}`, {
            method: 'POST', credentials: 'same-origin', cache: 'no-store',
            headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body || {})
        });
        if (!response.ok) throw new Error(await errorMessage(response));
        render(await response.json()); message('Account updated on the server. Enrolled bridges will refresh automatically.');
    } catch (error) { message(error.message, 'err'); } finally { pending = false; }
}
async function perform(provider, action, user) {
    if (pending) return;
    if (action === 'manual_token' || (provider === 'atlassian' && ['connect', 'reconnect'].includes(action))) { formFor(provider, action, user); return; }
    if (['connect', 'reconnect'].includes(action)) { connect(provider, user); return; }
    if (action === 'disconnect') { await showConfirmDialog('Disconnect account', `Disconnect ${names[provider]} on every enrolled device?`, 'Disconnect', () => mutate(provider, action)); return; }
    await mutate(provider, action);
}
if (root) {
    refresh();
    window.setInterval(() => { if (!document.hidden) refresh(); }, 15000);
    window.addEventListener('focus', refresh);
}
