import { apiFetch, BASE, API_BASE } from '../services/api.js';
import { showToast } from '../services/toast.js';

const buildQueryString = () => {
  const params = new URLSearchParams();
  const from = document.getElementById('audit-from');
  const to = document.getElementById('audit-to');
  const user = document.getElementById('audit-user');
  const skill = document.getElementById('audit-skill');
  const format = document.getElementById('audit-format');
  if (from && from.value) params.set('from', from.value);
  if (to && to.value) params.set('to', to.value);
  if (user && user.value.trim()) params.set('user_id', user.value.trim());
  if (skill && skill.value.trim()) params.set('skill', skill.value.trim());
  if (format && format.value) params.set('format', format.value);
  return params.toString();
};

const addTimestamps = (qs) =>
  qs.replace(/from=([^&]+)/, (m, v) => 'from=' + v + 'T00:00:00Z')
    .replace(/to=([^&]+)/, (m, v) => 'to=' + v + 'T23:59:59Z');

export const initAuditPage = () => {
  if (document.getElementById('audit-from')) {
    const previewBtn = document.getElementById('btn-preview');
    if (previewBtn) {
      previewBtn.addEventListener('click', () => {
        window.location.href = BASE + '/audit/?' + buildQueryString();
      });
    }

    const downloadBtn = document.getElementById('btn-download');
    if (downloadBtn) {
      downloadBtn.addEventListener('click', async () => {
        const format = document.getElementById('audit-format').value;
        const qs = addTimestamps(buildQueryString());
        if (format === 'csv') {
          window.open(API_BASE + '/export/usage?' + qs, '_blank');
        } else {
          try {
            const rows = await apiFetch('/export/usage?' + qs);
            const blob = new Blob([JSON.stringify(rows, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'usage-export.json';
            a.click();
            URL.revokeObjectURL(url);
          } catch (err) {
            showToast(err.message || 'Download failed', 'error');
          }
        }
      });
    }
  }
};
