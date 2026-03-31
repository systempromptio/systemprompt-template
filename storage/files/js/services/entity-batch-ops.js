import { apiFetch } from './api.js';
import { showToast } from './toast.js';

export const executeBatchDelete = async ({
  table, rowSelector, idExtractor, selected,
  deleteEndpoint, entityLabel, bodyBuilder,
  deleteBtn, updateBar, syncSelectAll,
}) => {
  const ids = [...selected];
  const count = ids.length;
  const label = count === 1 ? entityLabel : entityLabel + 's';

  if (!confirm('Delete ' + count + ' ' + label + '? This cannot be undone.')) {
    return;
  }

  deleteBtn.disabled = true;
  deleteBtn.textContent = 'Deleting...';

  try {
    const body = bodyBuilder ? bodyBuilder(ids) : JSON.stringify({ ids });
    await apiFetch(deleteEndpoint, { method: 'DELETE', body });

    const rows = table.querySelectorAll(rowSelector);
    for (const row of rows) {
      const id = idExtractor(row);
      if (id && selected.has(id)) {
        const detail = row.nextElementSibling;
        if (detail?.classList.contains('detail-row') || detail?.classList.contains('hook-code-row')) {
          detail.remove();
        }
        row.remove();
      }
    }

    selected.clear();
    updateBar();
    syncSelectAll();
    showToast(count + ' ' + label + ' deleted', 'success');
  } catch (err) {
    showToast(err.message || 'Batch delete failed', 'error');
  } finally {
    deleteBtn.disabled = false;
    deleteBtn.textContent = 'Delete';
  }
};
