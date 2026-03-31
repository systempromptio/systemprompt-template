import { executeBatchDelete } from './entity-batch-ops.js';

export function initEntityBatch({
  tableSelector,
  rowSelector,
  idExtractor,
  deleteEndpoint,
  entityLabel,
  bodyBuilder,
  protectionCheck,
}) {
  const table = document.querySelector(tableSelector);
  if (table) {
    const selected = new Set();
    const bar = document.getElementById('entity-batch-bar');
    const countEl = document.getElementById('entity-batch-count');
    const deleteBtn = document.getElementById('entity-batch-delete');
    const selectAll = document.getElementById('entity-select-all');

    if (bar && countEl && deleteBtn) {
      const getVisibleRows = () => {
        const rows = table.querySelectorAll(rowSelector);
        return [...rows].filter((r) => r.style.display !== 'none');
      };

      const getSelectableRows = () => {
        return getVisibleRows().filter((r) => {
          const cb = r.querySelector('.entity-row-checkbox');
          return cb && !cb.disabled;
        });
      };

      const updateBar = () => {
        if (selected.size > 0) {
          bar.hidden = false;
          const label = selected.size === 1 ? entityLabel : entityLabel + 's';
          countEl.textContent = selected.size + ' ' + label + ' selected';
        } else {
          bar.hidden = true;
        }
      };

      const syncSelectAll = () => {
        if (selectAll) {
          const selectable = getSelectableRows();
          const checkedCount = selectable.filter((r) => {
            const id = idExtractor(r);
            return id && selected.has(id);
          }).length;
          selectAll.checked = selectable.length > 0 && checkedCount === selectable.length;
          selectAll.indeterminate = checkedCount > 0 && checkedCount < selectable.length;
        }
      };

      const setRowSelected = (row, isSelected) => {
        row.classList.toggle('entity-row--selected', isSelected);
        const detail = row.nextElementSibling;
        if (detail?.classList.contains('detail-row') || detail?.classList.contains('hook-code-row')) {
          detail.classList.toggle('entity-row--selected', isSelected);
        }
      };

      table.addEventListener('change', (e) => {
        const cb = e.target.closest('.entity-row-checkbox');
        if (cb) {
          const row = cb.closest(rowSelector);
          if (row) {
            const id = idExtractor(row);
            if (id) {
              if (cb.checked) {
                selected.add(id);
                setRowSelected(row, true);
              } else {
                selected.delete(id);
                setRowSelected(row, false);
              }
              updateBar();
              syncSelectAll();
            }
          }
        }
      });

      if (selectAll) {
        selectAll.checked = false;
        selectAll.indeterminate = false;

        selectAll.addEventListener('change', () => {
          const checked = selectAll.checked;
          for (const row of getSelectableRows()) {
            const id = idExtractor(row);
            if (!id) continue;
            const cb = row.querySelector('.entity-row-checkbox');
            if (!cb || cb.disabled) continue;
            cb.checked = checked;
            if (checked) {
              selected.add(id);
              setRowSelected(row, true);
            } else {
              selected.delete(id);
              setRowSelected(row, false);
            }
          }
          updateBar();
        });
      }

      deleteBtn.addEventListener('click', async () => {
        if (selected.size > 0) {
          await executeBatchDelete({
            table, rowSelector, idExtractor, selected,
            deleteEndpoint, entityLabel, bodyBuilder,
            deleteBtn, updateBar, syncSelectAll,
          });
        }
      });
    }
  }
}
