(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    let overlay = null;
    let currentSkillId = null;
    let currentSkillName = '';
    let files = [];
    let selectedFile = null;

    const categoryLabels = {
        script: 'Scripts',
        reference: 'References',
        template: 'Templates',
        diagnostic: 'Diagnostics',
        data: 'Data',
        config: 'Config',
        asset: 'Assets'
    };

    const categoryOrder = ['script', 'reference', 'template', 'diagnostic', 'data', 'config', 'asset'];

    const groupByCategory = (fileList) => {
        const groups = {};
        fileList.forEach((f) => {
            const cat = f.category || 'config';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(f);
        });
        return groups;
    };

    const renderFileList = () => {
        if (!files.length) {
            return '<div class="empty-state" style="padding:var(--space-6)"><p>No files found for this skill.</p>' +
                '<p style="font-size:var(--text-sm);color:var(--text-tertiary);margin-top:var(--space-2)">Click "Sync Files" to scan the filesystem.</p></div>';
        }
        const groups = groupByCategory(files);
        let html = '';
        categoryOrder.forEach((cat) => {
            const group = groups[cat];
            if (!group || !group.length) return;
            html += '<div style="margin-bottom:var(--space-3)">' +
                '<div class="skill-file-category">' +
                escapeHtml(categoryLabels[cat] || cat) + ' (' + group.length + ')' +
                '</div>';
            group.forEach((f) => {
                const isSelected = selectedFile && selectedFile.id === f.id;
                html += '<div class="skill-file-item' + (isSelected ? ' selected' : '') + '" data-file-id="' + escapeHtml(f.id) + '">' +
                    '<span class="skill-file-name">' + escapeHtml(f.file_path) + '</span>' +
                    (f.language ? '<span class="skill-file-lang">' + escapeHtml(f.language) + '</span>' : '') +
                '</div>';
            });
            html += '</div>';
        });
        return html;
    };

    const validateContent = (content, lang) => {
        if (!content || !lang) return null;
        lang = lang.toLowerCase();
        try {
            if (lang === 'json') {
                JSON.parse(content);
                return null;
            }
            if (lang === 'yaml' || lang === 'yml') {
                const lines = content.split('\n');
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i];
                    if (line.trim() === '' || line.trim().charAt(0) === '#') continue;
                    if (/\t/.test(line.match(/^(\s*)/)[1])) {
                        return 'Line ' + (i + 1) + ': tabs not allowed in YAML, use spaces';
                    }
                }
                return null;
            }
            if (lang === 'python') {
                return checkBrackets(content, [['(', ')'], ['[', ']'], ['{', '}']]);
            }
            if (lang === 'bash' || lang === 'shell') {
                return checkBrackets(content, [['(', ')'], ['[', ']'], ['{', '}']]);
            }
        } catch (e) {
            return e.message;
        }
        return null;
    };

    const checkBrackets = (content, pairs) => {
        const stack = [];
        const closeMap = {};
        const openSet = {};
        pairs.forEach((p) => { closeMap[p[1]] = p[0]; openSet[p[0]] = p[1]; });
        let inStr = false;
        let strChar = '';
        let escaped = false;
        for (let i = 0; i < content.length; i++) {
            const ch = content[i];
            if (escaped) { escaped = false; continue; }
            if (ch === '\\') { escaped = true; continue; }
            if (inStr) {
                if (ch === strChar) inStr = false;
                continue;
            }
            if (ch === '"' || ch === "'") { inStr = true; strChar = ch; continue; }
            if (ch === '#' && content.charAt(i - 1) !== '$') {
                const nl = content.indexOf('\n', i);
                if (nl === -1) break;
                i = nl;
                continue;
            }
            if (openSet[ch]) { stack.push(ch); }
            else if (closeMap[ch]) {
                if (!stack.length) {
                    const line = content.substring(0, i).split('\n').length;
                    return 'Line ' + line + ': unexpected \'' + ch + '\'';
                }
                const top = stack.pop();
                if (top !== closeMap[ch]) {
                    const line2 = content.substring(0, i).split('\n').length;
                    return 'Line ' + line2 + ': expected \'' + openSet[top] + '\' but found \'' + ch + '\'';
                }
            }
        }
        if (stack.length) {
            return 'Unclosed \'' + stack[stack.length - 1] + '\'';
        }
        return null;
    };

    const renderEditor = () => {
        if (!selectedFile) {
            return '<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--text-tertiary);font-size:var(--text-sm)">Select a file to view its contents</div>';
        }
        return '<div style="display:flex;flex-direction:column;height:100%">' +
            '<div style="display:flex;align-items:center;gap:var(--space-2);padding:var(--space-2) var(--space-3);border-bottom:1px solid var(--border-subtle);flex-shrink:0">' +
                '<span style="font-family:monospace;font-size:var(--text-sm);font-weight:600">' + escapeHtml(selectedFile.file_path) + '</span>' +
                '<span class="badge badge-blue" style="font-size:var(--text-xs)">' + escapeHtml(selectedFile.language || 'text') + '</span>' +
                (selectedFile.executable ? '<span class="badge badge-green" style="font-size:var(--text-xs)">executable</span>' : '') +
                '<span style="margin-left:auto;font-size:var(--text-xs);color:var(--text-tertiary)">' + selectedFile.size_bytes + ' bytes</span>' +
            '</div>' +
            '<textarea id="skill-file-editor" style="flex:1;width:100%;border:none;padding:var(--space-3);font-family:monospace;font-size:var(--text-sm);line-height:1.5;resize:none;background:var(--bg-surface);color:var(--text-primary);outline:none;box-sizing:border-box">' +
                escapeHtml(selectedFile.content || '') +
            '</textarea>' +
            '<div style="display:flex;align-items:center;padding:var(--space-2) var(--space-3);border-top:1px solid var(--border-subtle);flex-shrink:0">' +
                '<span id="skill-file-validation" style="font-size:var(--text-xs);flex:1"></span>' +
                '<button class="btn btn-primary btn-sm" id="skill-file-save" style="font-size:var(--text-xs)">Save</button>' +
            '</div>' +
        '</div>';
    };

    const renderModal = () => {
        return '<div style="display:flex;flex-direction:column;height:100%">' +
            '<div style="display:flex;align-items:center;padding:var(--space-4);border-bottom:1px solid var(--border-subtle);flex-shrink:0">' +
                '<h2 style="margin:0;font-size:var(--text-lg);font-weight:600;color:var(--text-primary)">' + escapeHtml(currentSkillName) + ' - Files</h2>' +
                '<div style="margin-left:auto;display:flex;gap:var(--space-2)">' +
                    '<button class="btn btn-secondary btn-sm" id="skill-files-sync" style="font-size:var(--text-xs)">Sync Files</button>' +
                    '<button class="btn btn-secondary btn-sm" id="skill-files-close" style="font-size:var(--text-xs)">Close</button>' +
                '</div>' +
            '</div>' +
            '<div style="display:flex;flex:1;min-height:0">' +
                '<div id="skill-files-list" style="width:280px;overflow-y:auto;border-right:1px solid var(--border-subtle);padding:var(--space-2) 0">' +
                    renderFileList() +
                '</div>' +
                '<div id="skill-files-editor" style="flex:1;min-width:0;overflow:hidden">' +
                    renderEditor() +
                '</div>' +
            '</div>' +
        '</div>';
    };

    const updatePanel = () => {
        const panel = overlay && overlay.querySelector('.skill-files-panel');
        if (panel) panel.innerHTML = renderModal();
        bindEvents();
    };

    const runValidation = () => {
        if (!overlay || !selectedFile) return;
        const editor = overlay.querySelector('#skill-file-editor');
        const badge = overlay.querySelector('#skill-file-validation');
        if (!editor || !badge) return;
        const err = validateContent(editor.value, selectedFile.language);
        if (err) {
            badge.textContent = err;
            badge.style.color = 'var(--danger)';
        } else {
            badge.textContent = '';
        }
    };

    const bindEditorValidation = () => {
        if (!overlay) return;
        const editor = overlay.querySelector('#skill-file-editor');
        if (editor) {
            editor.addEventListener('input', runValidation);
            runValidation();
        }
    };

    const handleFileClick = (e) => {
        const item = e.currentTarget;
        const fileId = item.getAttribute('data-file-id');
        selectedFile = files.find((f) => f.id === fileId) || null;
        const listEl = overlay.querySelector('#skill-files-list');
        const editorEl = overlay.querySelector('#skill-files-editor');
        if (listEl) listEl.innerHTML = renderFileList();
        if (editorEl) editorEl.innerHTML = renderEditor();
        bindFileItems();
        const newSaveBtn = overlay.querySelector('#skill-file-save');
        if (newSaveBtn) newSaveBtn.addEventListener('click', handleSave);
        bindEditorValidation();
    };

    const bindFileItems = () => {
        if (!overlay) return;
        const fileItems = overlay.querySelectorAll('.skill-file-item');
        fileItems.forEach((item) => {
            item.addEventListener('click', handleFileClick);
        });
    };

    const bindEvents = () => {
        if (!overlay) return;

        const closeBtn = overlay.querySelector('#skill-files-close');
        if (closeBtn) closeBtn.addEventListener('click', close);

        const syncBtn = overlay.querySelector('#skill-files-sync');
        if (syncBtn) syncBtn.addEventListener('click', handleSync);

        const saveBtn = overlay.querySelector('#skill-file-save');
        if (saveBtn) saveBtn.addEventListener('click', handleSave);

        bindFileItems();
        bindEditorValidation();
    };

    const handleSync = async () => {
        const syncBtn = overlay && overlay.querySelector('#skill-files-sync');
        if (syncBtn) {
            syncBtn.disabled = true;
            syncBtn.textContent = 'Syncing...';
        }
        try {
            const result = await app.api('/skills/sync-files', { method: 'POST' });
            app.Toast.show('Synced: ' + (result.created || 0) + ' created, ' + (result.updated || 0) + ' updated', 'success');
            await loadFiles();
            updatePanel();
        } catch (err) {
            app.Toast.show(err.message || 'Sync failed', 'error');
            if (syncBtn) {
                syncBtn.disabled = false;
                syncBtn.textContent = 'Sync Files';
            }
        }
    };

    const handleSave = async () => {
        if (!selectedFile) return;
        const editor = overlay && overlay.querySelector('#skill-file-editor');
        if (!editor) return;
        const content = editor.value;
        const err = validateContent(content, selectedFile.language);
        if (err) {
            app.Toast.show('Fix validation error before saving: ' + err, 'error');
            return;
        }
        const saveBtn = overlay.querySelector('#skill-file-save');
        if (saveBtn) {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';
        }
        try {
            await app.api('/skills/' + encodeURIComponent(currentSkillId) + '/files/' + selectedFile.file_path, {
                method: 'PUT',
                body: JSON.stringify({ content: content }),
                headers: { 'Content-Type': 'application/json' }
            });
            selectedFile.content = content;
            selectedFile.size_bytes = new Blob([content]).size;
            app.Toast.show('File saved', 'success');
        } catch (err) {
            app.Toast.show(err.message || 'Save failed', 'error');
        } finally {
            if (saveBtn) {
                saveBtn.disabled = false;
                saveBtn.textContent = 'Save';
            }
        }
    };

    const loadFiles = async () => {
        try {
            files = await app.api('/skills/' + encodeURIComponent(currentSkillId) + '/files');
            if (!Array.isArray(files)) files = [];
        } catch (err) {
            files = [];
            app.Toast.show(err.message || 'Failed to load files', 'error');
        }
    };

    const close = () => {
        if (overlay) {
            overlay.remove();
            overlay = null;
        }
        currentSkillId = null;
        currentSkillName = '';
        files = [];
        selectedFile = null;
    };

    const open = async (skillId, skillName) => {
        close();
        currentSkillId = skillId;
        currentSkillName = skillName || skillId;

        overlay = document.createElement('div');
        overlay.className = 'confirm-overlay';
        overlay.style.cssText = 'display:flex;align-items:center;justify-content:center;z-index:1000';
        overlay.innerHTML = '<div class="skill-files-panel" style="background:var(--bg-surface);border-radius:var(--radius-lg);width:90vw;max-width:1100px;height:80vh;overflow:hidden;box-shadow:var(--shadow-lg);display:flex;flex-direction:column">' +
            '<div style="display:flex;align-items:center;justify-content:center;height:100%;color:var(--text-tertiary)">Loading files...</div>' +
        '</div>';
        document.body.append(overlay);

        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) close();
        });

        await loadFiles();
        updatePanel();
    };

    app.skillFiles = {
        open: open,
        close: close
    };
})(window.AdminApp);
