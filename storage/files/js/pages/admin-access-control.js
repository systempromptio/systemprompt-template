// /admin/access-control unified controller.
//
// Three responsibilities:
//   1. Department/user tree click handling (left pane).
//   2. Render department editor or user permission matrix (center pane).
//   3. Modals: "Show as YAML" + "Create department".
//
// Data flow: YAML is bootstrap config (read at server start). All edits made
// here write to this instance's database via the admin API. They are NOT
// pushed back to YAML or git.

const API = '/api/public/admin';

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

let entityCatalogue = { gateway_routes: [], mcp_servers: [], plugins: [], agents: [] };
let knownRoles = ['admin', 'developer', 'analyst', 'viewer'];

function readEmbedded(id, fallback) {
    const el = document.getElementById(id);
    if (!el) return fallback;
    try {
        return JSON.parse(el.textContent);
    } catch (_) {
        return fallback;
    }
}

function escapeHtml(s) {
    if (s == null) return '';
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

async function api(path, options) {
    const resp = await fetch(API + path, {
        headers: { 'Content-Type': 'application/json' },
        ...options,
    });
    if (!resp.ok) {
        const text = await resp.text().catch(() => resp.statusText);
        throw new Error(text || resp.statusText);
    }
    const ct = resp.headers.get('content-type') || '';
    return ct.includes('application/json') ? resp.json() : resp.text();
}

// ---------- Tree pane ---------- //

function clearActive() {
    $$('.ac-tree-dept .ac-dept-row[aria-pressed="true"]').forEach((b) =>
        b.setAttribute('aria-pressed', 'false'),
    );
    $$('.ac-tree-dept .ac-user-row[aria-pressed="true"]').forEach((b) =>
        b.setAttribute('aria-pressed', 'false'),
    );
}

function showWelcome() {
    $('#ac-welcome').hidden = false;
    $('#ac-dept-editor').hidden = true;
    $('#ac-user-matrix').hidden = true;
}

// ---------- Department editor ---------- //

function renderDeptEditor(deptName) {
    const editor = $('#ac-dept-editor');
    const safe = escapeHtml(deptName);
    const entityOptions = [
        { type: 'gateway_route', label: 'Gateway routes', items: entityCatalogue.gateway_routes },
        { type: 'mcp_server', label: 'MCP servers', items: entityCatalogue.mcp_servers },
        { type: 'plugin', label: 'Plugins', items: entityCatalogue.plugins },
        { type: 'agent', label: 'Agents', items: entityCatalogue.agents },
    ];
    const sections = entityOptions
        .map(
            (s) => `
        <div class="ac-section">
          <h3>${escapeHtml(s.label)}</h3>
          <div class="ac-dept-rule-list" data-entity-type="${s.type}">
            ${
                s.items.length === 0
                    ? '<div class="ac-dept-rule"><em>No entities of this type configured.</em></div>'
                    : s.items
                          .map(
                              (it) => `
              <div class="ac-dept-rule">
                <span class="ac-matrix-entity">${escapeHtml(it.label || it.id)}<span class="id">${escapeHtml(it.id)}</span></span>
                <span class="ac-source">—</span>
                <select data-action="dept-rule-access" data-entity-type="${s.type}" data-entity-id="${escapeHtml(it.id)}" data-dept="${safe}">
                  <option value="inherit">inherit</option>
                  <option value="allow">allow</option>
                  <option value="deny">deny</option>
                </select>
                <span></span>
              </div>`,
                          )
                          .join('')
            }
          </div>
        </div>
    `,
        )
        .join('');

    editor.innerHTML = `
      <div class="ac-user-header">
        <h2>${safe || '<em>Unassigned</em>'}</h2>
        <div class="ac-user-meta">Department editor · changes save to this instance's database</div>
      </div>
      <p>Toggle access for the whole department here. To make a rule permanent across deployments, copy it into <code>services/access-control/departments.yaml</code> in the source repo.</p>
      ${deptName ? sections : '<p><em>The "Unassigned" pseudo-bucket is for users without a department. Assign them to a department to manage their access.</em></p>'}
    `;
    editor.hidden = false;
    $('#ac-welcome').hidden = true;
    $('#ac-user-matrix').hidden = true;

    if (deptName) {
        loadDeptCurrentRules(deptName);
        bindDeptRuleHandlers(deptName);
    }
}

async function loadDeptCurrentRules(deptName) {
    let rules = [];
    try {
        const resp = await api('/access-control');
        rules = resp.rules || [];
    } catch (e) {
        console.error('Failed to load rules', e);
        return;
    }
    rules
        .filter((r) => r.rule_type === 'department' && r.rule_value === deptName)
        .forEach((r) => {
            const sel = $(
                `select[data-action="dept-rule-access"][data-entity-type="${r.entity_type}"][data-entity-id="${CSS.escape(r.entity_id)}"]`,
            );
            if (sel) sel.value = r.access;
            const row = sel?.closest('.ac-dept-rule');
            const src = row?.querySelector('.ac-source');
            if (src) src.textContent = `database (this instance)`;
        });
}

function bindDeptRuleHandlers(deptName) {
    $$('select[data-action="dept-rule-access"]').forEach((sel) => {
        sel.addEventListener('change', async () => {
            const entityType = sel.dataset.entityType;
            const entityId = sel.dataset.entityId;
            const action = sel.value;
            try {
                if (action === 'inherit') {
                    // Remove the rule entirely via apply_template clear.
                    await api('/access-control/bulk-template', {
                        method: 'POST',
                        body: JSON.stringify({
                            entity_type: entityType,
                            subject_type: 'department',
                            subject_value: deptName,
                            action: 'clear',
                        }),
                    });
                } else {
                    await api(
                        `/access-control/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}/rules`,
                        {
                            method: 'POST',
                            body: JSON.stringify({
                                rule_type: 'department',
                                rule_value: deptName,
                                access: action,
                            }),
                        },
                    );
                }
                const row = sel.closest('.ac-dept-rule');
                const src = row?.querySelector('.ac-source');
                if (src) src.textContent = action === 'inherit' ? '—' : 'database (this instance)';
            } catch (e) {
                alert('Failed to save rule: ' + e.message);
            }
        });
    });
}

// ---------- User matrix ---------- //

function layerClass(layer) {
    return ({ user: 'is-user', department: 'is-department', role: 'is-role', default: 'is-default' })[layer] || 'is-default';
}

async function renderUserMatrix(userId, displayName) {
    const matrix = $('#ac-user-matrix');
    matrix.innerHTML = `<p>Loading matrix for <strong>${escapeHtml(displayName)}</strong>…</p>`;
    matrix.hidden = false;
    $('#ac-welcome').hidden = true;
    $('#ac-dept-editor').hidden = true;

    let data;
    try {
        data = await api(`/access-control/users/${encodeURIComponent(userId)}/matrix`);
    } catch (e) {
        matrix.innerHTML = `<p class="error">Failed to load matrix: ${escapeHtml(e.message)}</p>`;
        return;
    }

    const u = data.user;
    const sections = (data.sections || [])
        .map((sec) => {
            if (!sec.rows || sec.rows.length === 0) {
                return `
          <div class="ac-matrix-section">
            <div class="ac-matrix-section-header">${escapeHtml(sec.label)}</div>
            <div class="ac-matrix-row"><em>No entities of this type configured.</em></div>
          </div>`;
            }
            const rows = sec.rows
                .map((row) => {
                    const eff = row.effective || 'deny';
                    const lc = layerClass(row.source.layer);
                    return `
            <div class="ac-matrix-row" data-entity-type="${escapeHtml(sec.entity_type)}" data-entity-id="${escapeHtml(row.entity_id)}">
              <div class="ac-matrix-entity">
                ${escapeHtml(row.entity_name)}
                <span class="id">${escapeHtml(row.entity_id)}</span>
              </div>
              <div class="ac-effective is-${eff}">${escapeHtml(eff)}</div>
              <div class="ac-source">
                <span class="ac-layer-tag ${lc}">${escapeHtml(row.source.layer)}</span>
                ${escapeHtml(row.source.detail)}
              </div>
              <div class="ac-override" data-entity-type="${escapeHtml(sec.entity_type)}" data-entity-id="${escapeHtml(row.entity_id)}">
                <button type="button" class="is-allow" data-override="allow" aria-pressed="${row.source.layer === 'user' && eff === 'allow'}">Allow</button>
                <button type="button" class="is-deny"  data-override="deny"  aria-pressed="${row.source.layer === 'user' && eff === 'deny'}">Deny</button>
                <button type="button" class="is-inherit" data-override="inherit" aria-pressed="${row.source.layer !== 'user'}">Inherit</button>
              </div>
            </div>`;
                })
                .join('');
            return `
        <div class="ac-matrix-section">
          <div class="ac-matrix-section-header">${escapeHtml(sec.label)} <span>${sec.rows.length}</span></div>
          ${rows}
        </div>`;
        })
        .join('');

    matrix.innerHTML = `
      <div class="ac-user-header">
        <h2>${escapeHtml(u.display_name || u.email || u.id)}</h2>
        <div class="ac-user-meta">
          ${escapeHtml(u.email || '')} · roles: ${(u.roles || []).map(escapeHtml).join(', ') || '<em>none</em>'} · department: ${escapeHtml(u.department || '—')}
        </div>
      </div>
      <p>Per-user overrides are saved to this instance's database only and are intentionally never written back to YAML.</p>
      ${sections}
    `;

    matrix.querySelectorAll('.ac-override button').forEach((btn) => {
        btn.addEventListener('click', async () => {
            const cell = btn.closest('.ac-override');
            const entityType = cell.dataset.entityType;
            const entityId = cell.dataset.entityId;
            const action = btn.dataset.override;
            try {
                if (action === 'inherit') {
                    await api('/access-control/bulk-template', {
                        method: 'POST',
                        body: JSON.stringify({
                            entity_type: entityType,
                            subject_type: 'user',
                            subject_value: userId,
                            action: 'clear',
                        }),
                    });
                } else {
                    await api(
                        `/access-control/entity/${encodeURIComponent(entityType)}/${encodeURIComponent(entityId)}/rules`,
                        {
                            method: 'POST',
                            body: JSON.stringify({
                                rule_type: 'user',
                                rule_value: userId,
                                access: action,
                            }),
                        },
                    );
                }
                renderUserMatrix(userId, displayName);
            } catch (e) {
                alert('Failed to save override: ' + e.message);
            }
        });
    });
}

// ---------- Search ---------- //

function bindSearch() {
    const input = $('#ac-search');
    if (!input) return;
    input.addEventListener('input', () => {
        const q = input.value.trim().toLowerCase();
        $$('.ac-tree-dept').forEach((dept) => {
            let anyMatch = false;
            const deptName = (dept.dataset.dept || '').toLowerCase();
            const deptVisible = !q || deptName.includes(q);
            dept.querySelectorAll('.ac-user-row').forEach((row) => {
                const name = (row.dataset.userDisplay || '').toLowerCase();
                const email = (row.dataset.userEmail || '').toLowerCase();
                const visible = !q || name.includes(q) || email.includes(q);
                row.parentElement.style.display = visible ? '' : 'none';
                if (visible) anyMatch = true;
            });
            dept.style.display = deptVisible || anyMatch ? '' : 'none';
        });
    });
}

// ---------- YAML modal ---------- //

function openModal(modalId) {
    $('#ac-modal-overlay').hidden = false;
    $(modalId).hidden = false;
}
function closeModals() {
    $('#ac-modal-overlay').hidden = true;
    $('#ac-yaml-modal').hidden = true;
    $('#ac-new-dept-modal').hidden = true;
}

async function showYamlModal() {
    openModal('#ac-yaml-modal');
    const target = $('#ac-yaml-content');
    target.textContent = 'Loading…';
    try {
        const yaml = await api('/access-control/yaml-snapshot');
        target.textContent = yaml || '# (no role/department rules in DB yet)\n';
    } catch (e) {
        target.textContent = '# Failed to load: ' + e.message;
    }
}

function bindYamlCopy() {
    $('#ac-yaml-copy').addEventListener('click', () => {
        const text = $('#ac-yaml-content').textContent || '';
        navigator.clipboard?.writeText(text).then(
            () => {
                $('#ac-yaml-copy').textContent = 'Copied!';
                setTimeout(() => ($('#ac-yaml-copy').textContent = 'Copy'), 1500);
            },
            () => alert('Copy failed — select the text manually.'),
        );
    });
}

// ---------- New department modal ---------- //

async function saveNewDepartment() {
    const name = $('#ac-new-dept-name').value.trim();
    const desc = $('#ac-new-dept-desc').value.trim();
    if (!name) {
        alert('Name is required');
        return;
    }
    try {
        await api('/management/departments', {
            method: 'POST',
            body: JSON.stringify({ name, description: desc }),
        });
        window.location.reload();
    } catch (e) {
        alert('Failed to create department: ' + e.message);
    }
}

// ---------- Boot ---------- //

function init() {
    entityCatalogue = readEmbedded('ac-entity-catalogue', entityCatalogue);
    knownRoles = readEmbedded('ac-known-roles', knownRoles);

    document.addEventListener('click', (ev) => {
        const target = ev.target.closest('[data-action]');
        if (!target) return;

        if (target.dataset.action === 'select-dept') {
            ev.preventDefault();
            clearActive();
            target.setAttribute('aria-pressed', 'true');
            renderDeptEditor(target.dataset.dept || '');
        } else if (target.dataset.action === 'select-user') {
            ev.preventDefault();
            clearActive();
            target.setAttribute('aria-pressed', 'true');
            renderUserMatrix(target.dataset.userId, target.dataset.userDisplay);
        }
    });

    bindSearch();
    bindYamlCopy();

    $('#ac-show-yaml').addEventListener('click', showYamlModal);
    $('#ac-new-department').addEventListener('click', () => openModal('#ac-new-dept-modal'));
    $('#ac-modal-overlay').addEventListener('click', closeModals);
    $('#ac-yaml-close').addEventListener('click', closeModals);
    $('#ac-new-dept-close').addEventListener('click', closeModals);
    $('#ac-new-dept-cancel').addEventListener('click', closeModals);
    $('#ac-new-dept-save').addEventListener('click', saveNewDepartment);

    document.addEventListener('keydown', (ev) => {
        if (ev.key === 'Escape') closeModals();
    });

    // Honor `?focus=gateway` and `?user=ID` query strings.
    const params = new URLSearchParams(window.location.search);
    if (params.get('user')) {
        const uid = params.get('user');
        const btn = document.querySelector(`.ac-user-row[data-user-id="${CSS.escape(uid)}"]`);
        btn?.click();
    } else if (params.get('focus') === 'gateway') {
        showWelcome();
    } else {
        showWelcome();
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
