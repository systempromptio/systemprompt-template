const buildDetailSection = (title, contentEl) => {
  const sec = document.createElement('div');
  sec.className = 'detail-section';
  const strong = document.createElement('strong');
  strong.textContent = title;
  sec.append(strong);
  sec.append(contentEl);
  return sec;
};

const buildDescriptionSection = (data) => {
  const p = document.createElement('p');
  p.className = 'detail-description';
  p.textContent = data.description || 'No description';
  return buildDetailSection('Description', p);
};

const buildCommandSection = (data) => {
  const pre = document.createElement('pre');
  pre.className = 'detail-command-pre';
  pre.textContent = data.command;
  return buildDetailSection('Command', pre);
};

const buildTagsSection = (tags) => {
  const wrapper = document.createDocumentFragment();
  wrapper.append(document.createElement('br'));
  const badgeRow = document.createElement('div');
  badgeRow.className = 'badge-row detail-tags-row';
  for (const tag of tags) { const badge = document.createElement('span'); badge.className = 'badge badge-gray'; badge.textContent = tag; badgeRow.append(badge); }
  wrapper.append(badgeRow);
  return buildDetailSection('Tags', wrapper);
};

const buildJsonSection = (data) => {
  const sec = document.createElement('div');
  sec.className = 'detail-section';
  const details = document.createElement('details');
  const summary = document.createElement('summary');
  summary.className = 'detail-json-summary';
  summary.textContent = 'JSON Config';
  details.append(summary);
  const jsonPre = document.createElement('pre');
  jsonPre.className = 'json-view';
  jsonPre.textContent = JSON.stringify(data, null, 2);
  details.append(jsonPre);
  sec.append(details);
  return sec;
};

const renderSkillExpand = (skillId, getSkillDetail) => {
  const data = getSkillDetail(skillId);
  if (!data) { const p = document.createElement('p'); p.className = 'text-muted'; p.textContent = 'No detail data available.'; return p; }
  const frag = document.createDocumentFragment();
  frag.append(buildDescriptionSection(data));
  if (data.command) frag.append(buildCommandSection(data));
  if (data.tags?.length) frag.append(buildTagsSection(data.tags));
  frag.append(buildJsonSection(data));
  return frag;
};

const collapseOtherRows = (table, detailRow) => {
  for (const r of table.querySelectorAll('tr.detail-row.visible')) {
    if (r === detailRow) continue;
    r.classList.remove('visible');
    r.previousElementSibling?.querySelector('.expand-indicator')?.classList.remove('expanded');
  }
};

const handleRowClick = (e, table, getSkillDetail) => {
  if (!e.target.closest('[data-no-row-click],.actions-menu,.btn,a,input,.toggle-switch')) {
    const row = e.target.closest('tr.clickable-row');
    if (row) {
      const detailRow = row.nextElementSibling;
      if (detailRow?.classList.contains('detail-row')) {
        const isVisible = detailRow.classList.contains('visible');
        collapseOtherRows(table, detailRow);
        detailRow.classList.toggle('visible', !isVisible);
        row.querySelector('.expand-indicator')?.classList.toggle('expanded', !isVisible);
        if (!isVisible) {
          const content = detailRow.querySelector('[data-skill-expand]');
          if (content && !content.hasAttribute('data-loaded')) {
            content.textContent = '';
            content.append(renderSkillExpand(content.getAttribute('data-skill-expand'), getSkillDetail));
            content.setAttribute('data-loaded', 'true');
          }
        }
      }
    }
  }
};

export const initExpandRows = (getSkillDetail) => {
  const table = document.querySelector('.data-table');
  if (table) {
    table.addEventListener('click', (e) => handleRowClick(e, table, getSkillDetail));
  }
};

const setRowVisibility = (row, match) => {
  row.style.display = match ? '' : 'none';
  const detail = row.nextElementSibling;
  if (detail?.classList.contains('detail-row')) {
    if (!match) { detail.style.display = 'none'; detail.classList.remove('visible'); }
    else detail.style.display = '';
  }
};

const FILTERS = [
  { selectId: 'source-filter', dataAttr: 'data-source' },
  { selectId: 'plugin-filter', dataAttr: 'data-plugins' },
  { selectId: 'tag-filter', dataAttr: 'data-tags' }
];

export const initFilters = () => {
  const table = document.querySelector('.data-table');
  if (table) {
    const apply = () => {
      const q = (document.getElementById('skill-search')?.value || '').toLowerCase().trim();
      const fv = FILTERS.map((f) => ({ attr: f.dataAttr, value: document.getElementById(f.selectId)?.value || '' }));
      for (const row of table.querySelectorAll('tbody tr.clickable-row')) {
        const ms = !q || (row.getAttribute('data-name') || '').includes(q) || (row.getAttribute('data-skill-id') || '').toLowerCase().includes(q) || (row.getAttribute('data-description') || '').includes(q);
        const mf = fv.every((f) => !f.value || (row.getAttribute(f.attr) || '').includes(f.value));
        setRowVisibility(row, ms && mf);
      }
    };
    for (const f of FILTERS) { document.getElementById(f.selectId)?.addEventListener('change', apply); }
    let timer;
    document.getElementById('skill-search')?.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(apply, 200); });
  }
};

const formatTimeAgo = (diff) => {
  if (diff < 60) return 'just now';
  if (diff < 3600) return Math.floor(diff / 60) + 'm ago';
  if (diff < 86400) return Math.floor(diff / 3600) + 'h ago';
  if (diff < 2592000) return Math.floor(diff / 86400) + 'd ago';
  return null;
};

export const initTimeAgo = () => {
  for (const el of document.querySelectorAll('.metadata-timestamp')) {
    const iso = el.getAttribute('title') || el.textContent.trim();
    if (!iso || iso === '--') continue;
    const d = new Date(iso);
    el.textContent = formatTimeAgo(Math.floor((Date.now() - d.getTime()) / 1000)) || d.toLocaleDateString();
    el.setAttribute('title', d.toLocaleString());
  }
};
