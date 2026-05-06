// /admin/governance/decisions — live indicator.
// Subscribes to /admin/api/sse/audit and reveals a "↑ N new — refresh" pill
// when new governance_decisions arrive while viewing the audit table.

export const initAuditLiveIndicator = () => {
  const pill = document.getElementById('audit-live-pill');
  const counter = document.getElementById('audit-live-pill-count');
  if (!pill || !counter) return;

  let n = 0;
  const es = new EventSource('/admin/api/sse/audit', { withCredentials: true });

  es.addEventListener('audit', (ev) => {
    let payload;
    try { payload = JSON.parse(ev.data); } catch { return; }
    if (payload.table !== 'governance_decisions') return;
    n += 1;
    counter.textContent = String(n);
    pill.hidden = false;
  });

  pill.addEventListener('click', () => {
    window.location.reload();
  });
};
