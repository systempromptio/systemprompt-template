// Why: the vocabulary of the connectors page — status labels, tones, action
// names and the per-provider glyph family — kept apart from the rendering so
// a wording change never touches the DOM code.
export const labels = { verification_required: 'Verification required', no_auth_required: 'Built in', not_configured: 'Not configured', not_connected: 'Not connected', connected: 'Connected', reconnect_required: 'Reconnect required', temporarily_unavailable: 'Temporarily unavailable' };
export const tones = { connected: 'ok', reconnect_required: 'warn', verification_required: 'warn', temporarily_unavailable: 'err' };
export const names = { atlassian: 'Atlassian', github: 'GitHub' };
export const actionLabels = { connect: 'Connect', reconnect: 'Reconnect', disconnect: 'Disconnect', test: 'Test connection', manual_token: 'Use personal token' };
const notes = { provider_reprovisioned: 'An administrator reset this connector — reconnect to continue', verification_failed: 'Authorization saved, but the last verification failed', grant_rejected: 'The provider rejected the saved grant', configuration_changed: 'The connector configuration changed — reconnect to authorize again', provider_unavailable: 'The provider did not answer the last check', provider_permission_denied: 'The provider refused the permissions this connector needs' };
export const groups = [
    { key: 'attention', title: 'Needs your attention', note: 'Broken or unverified — fix these first.' },
    { key: 'ready', title: 'Ready to connect', note: 'Authorize once; every client you connect uses the same account.' },
    { key: 'connected', title: 'Connected', note: 'Verified and available to every client.' },
    { key: 'quiet', title: 'Nothing to do', note: 'Built in, or not open to your account.' }
];
export const steps = [
    { id: 'token', title: 'Credential', pending: 'Checking the saved grant' },
    { id: 'initialize', title: 'MCP session', pending: 'Opening a session with the provider' },
    { id: 'tools', title: 'Tools', pending: 'Listing the tools this account can call' },
    { id: 'identity', title: 'Identity', pending: 'Confirming who the provider says you are' }
];
export const attention = ['reconnect_required', 'verification_required', 'temporarily_unavailable'];
export function relative(iso) {
    const minutes = Math.round((Date.now() - new Date(iso).getTime()) / 60000);
    if (minutes < 1) return 'just now';
    if (minutes < 60) return `${minutes} min ago`;
    if (minutes < 60 * 24) return `${Math.round(minutes / 60)} h ago`;
    return new Date(iso).toLocaleDateString();
}
export function host(value) { try { return new URL(value).host; } catch (_) { return value; } }
export function displayName(account) { return account.display_name || names[account.provider] || account.provider; }
export function groupOf(account) {
    if (attention.includes(account.status) && account.entitled) return 'attention';
    if (account.status === 'not_connected' && account.entitled) return 'ready';
    if (account.status === 'connected') return 'connected';
    return 'quiet';
}
export function glyph(provider) {
    if (provider === 'atlassian') return 'atlassian';
    if (provider === 'github') return 'github';
    if (provider === 'systemprompt') return 'systemprompt';
    return 'generic';
}
export function monogram(family, name) { return { atlassian: 'At', github: 'Gh', systemprompt: 'Sp' }[family] || name.slice(0, 2); }
export function kind(family) { return { atlassian: 'Atlassian Cloud', github: 'GitHub', systemprompt: 'Control plane' }[family] || 'MCP server'; }
export function note(account) {
    if (!account.entitled) return account.session_attested ? 'Not open to your account' : 'An active account is required to connect providers';
    return notes[account.error_code] || '';
}
export function variant(action, status) {
    if (action === 'connect') return 'sp-btn--primary';
    if (action === 'reconnect') return status === 'connected' ? 'sp-btn--outline' : 'sp-btn--primary';
    if (action === 'disconnect') return 'sp-btn--outline-danger';
    if (action === 'test') return status === 'connected' ? 'sp-btn--primary' : 'sp-btn--outline';
    return 'sp-btn--ghost';
}
