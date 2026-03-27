(function(app) {
    'use strict';

    app.initOrgHooks = function() {
        const OrgCommon = app.OrgCommon;
        if (!OrgCommon) return;

        OrgCommon.initExpandRows('.data-table');

        const searchInput = document.getElementById('hook-search');
        if (searchInput) {
            searchInput.addEventListener('input', function() {
                const query = this.value.toLowerCase();
                const rows = document.querySelectorAll('.data-table tbody tr.clickable-row');
                rows.forEach((row) => {
                    const name = (row.getAttribute('data-name') || '').toLowerCase();
                    const match = !query || name.includes(query);
                    row.style.display = match ? '' : 'none';
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        if (!match) {
                            detail.classList.remove('visible');
                            detail.style.display = 'none';
                        } else {
                            detail.style.display = '';
                        }
                    }
                });
            });
        }

        app.events.on('click', '[data-hook-json]', (e, el) => {
            const hookId = el.getAttribute('data-hook-json');
            const container = document.querySelector('[data-hook-json-container="' + hookId + '"]');
            if (!container) return;

            if (container.style.display === 'none') {
                const script = document.querySelector('script[data-hook-detail="' + hookId + '"]');
                if (script) {
                    try {
                        const data = JSON.parse(script.textContent);
                        container.innerHTML = OrgCommon.formatJson(data);
                    } catch (err) {
                        container.innerHTML = '<span class="text-muted">Failed to parse JSON</span>';
                    }
                }
                container.style.display = 'block';
                el.textContent = 'Hide JSON';
            } else {
                container.style.display = 'none';
                el.textContent = 'Show JSON';
            }
        });

        app.events.on('click', '[data-action="delete"][data-entity-type="hook"]', (e, el) => {
            const id = el.getAttribute('data-entity-id');
            if (!confirm('Delete this hook? This cannot be undone.')) return;

            app.api('/hooks/' + encodeURIComponent(id), {
                method: 'DELETE'
            }).then(() => {
                app.Toast.show('Hook deleted', 'success');
                const row = document.querySelector('tr[data-entity-id="' + id + '"].clickable-row');
                if (row) {
                    const detail = row.nextElementSibling;
                    if (detail && detail.classList.contains('detail-row')) {
                        detail.remove();
                    }
                    row.remove();
                }
            }).catch((err) => {
                app.Toast.show(err.message || 'Failed to delete hook', 'error');
            });
        });

        app.events.on('click', '[data-hook-details]', (e, el) => {
            const hookId = el.getAttribute('data-hook-details');
            const row = document.querySelector('tr[data-entity-id="' + hookId + '"].clickable-row');
            if (!row) return;
            const detailRow = row.nextElementSibling;
            if (!detailRow || !detailRow.classList.contains('detail-row')) return;
            OrgCommon.handleRowClick(row, detailRow);
        });
    };

})(window.AdminApp = window.AdminApp || {});
