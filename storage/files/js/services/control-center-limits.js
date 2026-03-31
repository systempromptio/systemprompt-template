export const initUsageLimits = (handler) => (data) => {
  renderUsageLimits(data);
  handler?.(data);
};

const buildLimitRow = (item) => {
  const current = item.current || 0;
  const max = item.max || 0;
  if (max <= 0) return null;

  const isUnlimited = max > 9999999;
  const pct = isUnlimited ? Math.min(current * 0.5, 5) : Math.min((current / max) * 100, 100);
  const colorClass = isUnlimited ? 'cc-limit-bar--green' : getBarColor(pct);
  const currentDisplay = item.fmt === 'bytes' ? formatBytes(current) : formatNumber(current);
  const maxDisplay = isUnlimited ? 'Unlimited' : (item.fmt === 'bytes' ? formatBytes(max) : formatNumber(max));

  const row = document.createElement('div');
  row.className = 'cc-limit-row';

  const info = document.createElement('div');
  info.className = 'cc-limit-info';
  const label = document.createElement('span');
  label.className = 'cc-limit-label';
  label.textContent = item.label;
  const count = document.createElement('span');
  count.className = 'cc-limit-count';
  count.textContent = currentDisplay + ' / ' + maxDisplay;
  info.append(label, count);

  const track = document.createElement('div');
  track.className = 'cc-limit-bar-track';
  const fill = document.createElement('div');
  fill.className = 'cc-limit-bar-fill ' + colorClass;
  fill.style.setProperty('--sp-fill', pct.toFixed(1) + '%');
  track.append(fill);

  row.append(info, track);
  return row;
};

const renderUsageLimits = (data) => {
  const container = document.getElementById('cc-usage-limits');
  if (!container) return;

  const badge = document.getElementById('cc-plan-name');
  if (badge) badge.textContent = data.plan_name || 'Free';

  const grid = document.getElementById('cc-limits-grid');
  if (!grid) return;

  const limits = data.limits || {};
  const usage = data.usage || {};
  const ingestion = limits.ingestion || {};
  const entities = limits.entities || {};

  const items = [
    { label: 'Events Today', current: usage.events_today, max: ingestion.events_per_day, fmt: 'number' },
    { label: 'Data Today', current: usage.content_bytes_today, max: ingestion.content_bytes_per_day, fmt: 'bytes' },
    { label: 'Sessions Today', current: usage.sessions_today, max: ingestion.sessions_per_day, fmt: 'number' },
    { label: 'Skills', current: usage.skills_count, max: entities.max_skills, fmt: 'number' },
    { label: 'Agents', current: usage.agents_count, max: entities.max_agents, fmt: 'number' },
    { label: 'Plugins', current: usage.plugins_count, max: entities.max_plugins, fmt: 'number' },
    { label: 'MCP Servers', current: usage.mcp_servers_count, max: entities.max_mcp_servers, fmt: 'number' },
    { label: 'Hooks', current: usage.hooks_count, max: entities.max_hooks, fmt: 'number' },
  ];

  grid.replaceChildren();

  for (const item of items) {
    const row = buildLimitRow(item);
    if (row) grid.append(row);
  }

  const warningsEl = document.getElementById('global-limit-warnings');
  const warnings = data.warnings || [];
  if (warningsEl) {
    if (warnings.length > 0) {
      warningsEl.hidden = false;
      warningsEl.replaceChildren();
      for (const w of warnings) {
        const isAtLimit = w.usage_pct >= 1.0;
        const div = document.createElement('div');
        div.className = isAtLimit ? 'cc-limit-warning cc-limit-warning--critical' : 'cc-limit-warning cc-limit-warning--caution';
        div.textContent = w.message;
        if (isAtLimit) {
          const cta = document.createElement('a');
          cta.href = '/settings#billing';
          cta.className = 'cc-limit-upgrade-link';
          cta.textContent = 'Upgrade';
          div.append(cta);
        }
        warningsEl.append(div);
      }
    } else {
      warningsEl.hidden = true;
    }
  }
};

const getBarColor = (pct) => {
  if (pct > 95) return 'cc-limit-bar--red';
  if (pct > 80) return 'cc-limit-bar--orange';
  if (pct > 60) return 'cc-limit-bar--yellow';
  return 'cc-limit-bar--green';
};

const formatBytes = (bytes) => {
  if (bytes == null) return '0 B';
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB';
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + ' MB';
  return (bytes / 1073741824).toFixed(1) + ' GB';
};

const formatNumber = (n) => {
  if (n == null) return '0';
  return n.toLocaleString();
};
