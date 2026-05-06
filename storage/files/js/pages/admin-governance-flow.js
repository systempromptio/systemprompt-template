// /admin/governance/flow — live decision timeline.
// Renders the SSR-supplied initial rows, then subscribes to /admin/api/sse/audit
// and prepends rows for governance_decisions inserts as they arrive.

const ROW_LIMIT = 500;

const escape = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({
  '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;',
}[c]));

const renderRow = (row) => {
  const decisionClass = row.decision === 'allow' ? 'flow-row-allow' : 'flow-row-deny';
  const time = (() => {
    try { return new Date(row.created_at).toLocaleTimeString(); }
    catch { return row.created_at || ''; }
  })();
  const rules = row.evaluated_rules ? JSON.stringify(row.evaluated_rules) : '';
  return `
    <li class="flow-row ${decisionClass}" data-rules='${escape(rules)}' data-session='${escape(row.session_id)}'>
      <span class="flow-row-time">${escape(time)}</span>
      <span class="flow-row-subject">${escape(row.subject_label)}</span>
      <span class="flow-row-tool"><code>${escape(row.tool_name)}</code></span>
      <span class="flow-row-decision">${escape(row.decision.toUpperCase())}</span>
      <span class="flow-row-policy">${escape(row.policy)}</span>
      <span class="flow-row-reason">${escape(row.reason)}</span>
    </li>
  `;
};

const trimList = (list) => {
  while (list.children.length > ROW_LIMIT) {
    list.removeChild(list.lastElementChild);
  }
};

const showDrawer = (rules, sessionId) => {
  const drawer = document.getElementById('flow-drawer');
  const json = document.getElementById('flow-drawer-json');
  const traceLink = document.getElementById('flow-drawer-trace');
  if (!drawer || !json) return;
  let pretty = '(no evaluated_rules)';
  try { pretty = JSON.stringify(JSON.parse(rules || '{}'), null, 2); }
  catch { pretty = rules || ''; }
  json.textContent = pretty;
  if (traceLink && sessionId) {
    traceLink.href = `/admin/performance/traces/${encodeURIComponent(sessionId)}`;
    traceLink.hidden = false;
  }
  drawer.setAttribute('aria-hidden', 'false');
  drawer.classList.add('open');
};

const hideDrawer = () => {
  const drawer = document.getElementById('flow-drawer');
  if (!drawer) return;
  drawer.setAttribute('aria-hidden', 'true');
  drawer.classList.remove('open');
};

export const initFlowPage = () => {
  const list = document.getElementById('flow-rows');
  const empty = document.getElementById('flow-empty');
  const counter = document.getElementById('flow-row-count');
  const livePill = document.getElementById('flow-live-pill');
  if (!list) return;

  // Render initial set from SSR JSON.
  const initialNode = document.getElementById('flow-initial-data');
  let rows = [];
  if (initialNode) {
    try { rows = JSON.parse(initialNode.textContent || '{}').rows || []; }
    catch { rows = []; }
  }
  list.innerHTML = rows.map(renderRow).join('');
  if (empty && rows.length) empty.hidden = true;

  // Drawer wiring.
  list.addEventListener('click', (e) => {
    const li = e.target.closest('.flow-row');
    if (!li) return;
    showDrawer(li.dataset.rules, li.dataset.session);
  });
  const closeBtn = document.getElementById('flow-drawer-close');
  if (closeBtn) closeBtn.addEventListener('click', hideDrawer);

  // SSE subscription.
  let received = 0;
  const url = '/admin/api/sse/audit';
  const es = new EventSource(url, { withCredentials: true });

  es.addEventListener('audit', (ev) => {
    let payload;
    try { payload = JSON.parse(ev.data); } catch { return; }
    if (payload.table !== 'governance_decisions') return;
    // We don't have the full row from the notify payload — fetch from API,
    // or fall back to rendering with what we have. Cheaper to render minimally.
    const row = {
      id: payload.id,
      user_id: payload.user_id,
      session_id: payload.session_id,
      tool_name: payload.tool_name,
      decision: payload.decision,
      policy: payload.policy,
      reason: payload.severity === 'breach' ? 'secret breach' : '',
      evaluated_rules: null,
      created_at: payload.created_at,
      subject_label: payload.user_id,
    };
    list.insertAdjacentHTML('afterbegin', renderRow(row));
    trimList(list);
    received += 1;
    if (counter) counter.textContent = `${list.children.length} decisions (live +${received})`;
    if (empty) empty.hidden = true;
    if (livePill) livePill.classList.add('flow-live-active');
  });

  es.addEventListener('hello', () => {
    if (livePill) livePill.classList.add('flow-live-active');
  });

  es.addEventListener('error', () => {
    if (livePill) livePill.classList.remove('flow-live-active');
  });
};
