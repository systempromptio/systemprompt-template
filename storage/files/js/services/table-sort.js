import { on } from './events.js';

export const initTableSort = () => {
  for (const table of document.querySelectorAll('.data-table')) {
    setupTableSort(table);
  }
  on('click', '.data-table thead th[data-sort-col]', handleSortClick);
};

const setupTableSort = (table) => {
  if (!table.getAttribute('data-sort-init')) {
    table.setAttribute('data-sort-init', '1');
    const headers = table.querySelectorAll('thead th');
    if (headers.length) {
      for (let i = 0; i < headers.length; i++) {
        const th = headers[i];
        const text = th.textContent.trim();
        if (!text || th.classList.contains('col-actions') || th.classList.contains('col-chevron')) continue;
        if (!th.querySelector('.sort-icon')) {
          const icon = document.createElement('span');
          icon.className = 'sort-icon';
          icon.textContent = '\u25B4';
          th.append(icon);
        }
        th.setAttribute('data-sort-col', String(i));
      }
    }
  }
};

const resetHeaders = (table) => {
  for (const h of table.querySelectorAll('thead th')) {
    h.classList.remove('sorted');
    h.removeAttribute('data-sort-dir');
    const icon = h.querySelector('.sort-icon');
    if (icon) icon.textContent = '\u25B4';
  }
};

const partitionRows = (tbody) => {
  const sortable = [];
  const detailMap = {};
  for (const row of tbody.querySelectorAll('tr')) {
    const detailFor = row.getAttribute('data-detail-for');
    if (detailFor) detailMap[detailFor] = row;
    else sortable.push(row);
  }
  return { sortable, detailMap };
};

const handleSortClick = (e, th) => {
  const table = th.closest('.data-table');
  if (table) {
    const colIndex = parseInt(th.getAttribute('data-sort-col'), 10);
    const tbody = table.querySelector('tbody');
    if (tbody) {
      const wasAsc = th.classList.contains('sorted') && th.getAttribute('data-sort-dir') === 'asc';
      const dir = wasAsc ? 'desc' : 'asc';
      resetHeaders(table);
      th.classList.add('sorted');
      th.setAttribute('data-sort-dir', dir);
      const activeIcon = th.querySelector('.sort-icon');
      if (activeIcon) activeIcon.textContent = dir === 'asc' ? '\u25B4' : '\u25BE';
      const { sortable, detailMap } = partitionRows(tbody);
      sortable.sort((a, b) => compareRows(a, b, colIndex, dir));
      for (const row of sortable) {
        tbody.append(row);
        const eventId = row.getAttribute('data-event-id');
        if (eventId && detailMap[eventId]) tbody.append(detailMap[eventId]);
      }
    }
  }
};

const compareRows = (a, b, colIndex, dir) => {
  const aVal = getSortValue(a.cells[colIndex]);
  const bVal = getSortValue(b.cells[colIndex]);
  const aNum = parseFloat(aVal);
  const bNum = parseFloat(bVal);
  if (!isNaN(aNum) && !isNaN(bNum)) return dir === 'asc' ? aNum - bNum : bNum - aNum;
  const cmp = aVal.localeCompare(bVal, undefined, { sensitivity: 'base' });
  return dir === 'asc' ? cmp : -cmp;
};

const getSortValue = (cell) => {
  if (!cell) return '';
  if (cell.title) return cell.title.toLowerCase();
  const sv = cell.getAttribute('data-sort-value');
  if (sv) return sv.toLowerCase();
  return (cell.textContent || '').trim().toLowerCase();
};
