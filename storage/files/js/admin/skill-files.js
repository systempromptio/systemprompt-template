(function(app) {
    'use strict';

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

    const buildFileList = () => {
        const frag = document.createDocumentFragment();
        if (!files.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            empty.style.cssText = 'padding:var(--sp-space-6)';
            const p1 = document.createElement('p');
            p1.textContent = 'No files found for this skill.';
            const p2 = document.createElement('p');
            p2.style.cssText = 'font-size:var(--sp-text-sm);color:var(--sp-text-tertiary);margin-top:var(--sp-space-2)';
            p2.textContent = 'Click "Sync Files" to scan the filesystem.';
            empty.append(p1, p2);
            frag.append(empty);
            return frag;
        }
        const groups = groupByCategory(files);
        categoryOrder.forEach((cat) => {
            const group = groups[cat];
            if (!group || !group.length) return;
            const wrapper = document.createElement('div');
            wrapper.style.cssText = 'margin-bottom:var(--sp-space-3)';
            const catDiv = document.createElement('div');
            catDiv.className = 'skill-file-category';
            catDiv.textContent = (categoryLabels[cat] || cat) + ' (' + group.length + ')';
            wrapper.append(catDiv);
            group.forEach((f) => {
                const isSelected = selectedFile && selectedFile.id === f.id;
                const item = document.createElement('div');
                item.className = 'skill-file-item' + (isSelected ? ' selected' : '');
                item.setAttribute('data-file-id', f.id);
                const nameSpan = document.createElement('span');
                nameSpan.className = 'skill-file-name';
                nameSpan.textContent = f.file_path;
                item.append(nameSpan);
                if (f.language) {
                    const langSpan = document.createElement('span');
                    langSpan.className = 'skill-file-lang';
                    langSpan.textContent = f.language;
                    item.append(langSpan);
                }
                wrapper.append(item);
            });
            frag.append(wrapper);
        });
        return frag;
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

    const buildEditor = () => {
        if (!selectedFile) {
            const placeholder = document.createElement('div');
            placeholder.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary);font-size:var(--sp-text-sm)';
            placeholder.textContent = 'Select a file to view its contents';
            return placeholder;
        }
        const wrapper = document.createElement('div');
        wrapper.style.cssText = 'display:flex;flex-direction:column;height:100%';

        const header = document.createElement('div');
        header.style.cssText = 'display:flex;align-items:center;gap:var(--sp-space-2);padding:var(--sp-space-2) var(--sp-space-3);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';

        const pathSpan = document.createElement('span');
        pathSpan.style.cssText = 'font-family:monospace;font-size:var(--sp-text-sm);font-weight:600';
        pathSpan.textContent = selectedFile.file_path;

        const langBadge = document.createElement('span');
        langBadge.className = 'badge badge-blue';
        langBadge.style.cssText = 'font-size:var(--sp-text-xs)';
        langBadge.textContent = selectedFile.language || 'text';

        header.append(pathSpan, langBadge);

        if (selectedFile.executable) {
            const execBadge = document.createElement('span');
            execBadge.className = 'badge badge-green';
            execBadge.style.cssText = 'font-size:var(--sp-text-xs)';
            execBadge.textContent = 'executable';
            header.append(execBadge);
        }

        const sizeSpan = document.createElement('span');
        sizeSpan.style.cssText = 'margin-left:auto;font-size:var(--sp-text-xs);color:var(--sp-text-tertiary)';
        sizeSpan.textContent = selectedFile.size_bytes + ' bytes';
        header.append(sizeSpan);

        const textarea = document.createElement('textarea');
        textarea.id = 'skill-file-editor';
        textarea.style.cssText = 'flex:1;width:100%;border:none;padding:var(--sp-space-3);font-family:monospace;font-size:var(--sp-text-sm);line-height:1.5;resize:none;background:var(--sp-bg-surface);color:var(--sp-text-primary);outline:none;box-sizing:border-box';
        textarea.value = selectedFile.content || '';

        const footer = document.createElement('div');
        footer.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-2) var(--sp-space-3);border-top:1px solid var(--sp-border-subtle);flex-shrink:0';

        const validationSpan = document.createElement('span');
        validationSpan.id = 'skill-file-validation';
        validationSpan.style.cssText = 'font-size:var(--sp-text-xs);flex:1';

        const saveBtn = document.createElement('button');
        saveBtn.className = 'btn btn-primary btn-sm';
        saveBtn.id = 'skill-file-save';
        saveBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        saveBtn.textContent = 'Save';

        footer.append(validationSpan, saveBtn);
        wrapper.append(header, textarea, footer);
        return wrapper;
    };

    const buildModal = () => {
        const outer = document.createElement('div');
        outer.style.cssText = 'display:flex;flex-direction:column;height:100%';

        const topBar = document.createElement('div');
        topBar.style.cssText = 'display:flex;align-items:center;padding:var(--sp-space-4);border-bottom:1px solid var(--sp-border-subtle);flex-shrink:0';

        const heading = document.createElement('h2');
        heading.style.cssText = 'margin:0;font-size:var(--sp-text-lg);font-weight:600;color:var(--sp-text-primary)';
        heading.textContent = currentSkillName + ' - Files';

        const btnGroup = document.createElement('div');
        btnGroup.style.cssText = 'margin-left:auto;display:flex;gap:var(--sp-space-2)';

        const syncBtn = document.createElement('button');
        syncBtn.className = 'btn btn-secondary btn-sm';
        syncBtn.id = 'skill-files-sync';
        syncBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        syncBtn.textContent = 'Sync Files';

        const closeBtn = document.createElement('button');
        closeBtn.className = 'btn btn-secondary btn-sm';
        closeBtn.id = 'skill-files-close';
        closeBtn.style.cssText = 'font-size:var(--sp-text-xs)';
        closeBtn.textContent = 'Close';

        btnGroup.append(syncBtn, closeBtn);
        topBar.append(heading, btnGroup);

        const body = document.createElement('div');
        body.style.cssText = 'display:flex;flex:1;min-height:0';

        const listPane = document.createElement('div');
        listPane.id = 'skill-files-list';
        listPane.style.cssText = 'width:280px;overflow-y:auto;border-right:1px solid var(--sp-border-subtle);padding:var(--sp-space-2) 0';
        listPane.append(buildFileList());

        const editorPane = document.createElement('div');
        editorPane.id = 'skill-files-editor';
        editorPane.style.cssText = 'flex:1;min-width:0;overflow:hidden';
        editorPane.append(buildEditor());

        body.append(listPane, editorPane);
        outer.append(topBar, body);
        return outer;
    };

    const updatePanel = () => {
        const panel = overlay && overlay.querySelector('.skill-files-panel');
        if (panel) panel.replaceChildren(buildModal());
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
            badge.style.color = 'var(--sp-danger)';
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
        if (listEl) listEl.replaceChildren(buildFileList());
        if (editorEl) editorEl.replaceChildren(buildEditor());
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

        const panel = document.createElement('div');
        panel.className = 'skill-files-panel';
        panel.style.cssText = 'background:var(--sp-bg-surface);border-radius:var(--sp-radius-lg);width:90vw;max-width:1100px;height:80vh;overflow:hidden;box-shadow:var(--sp-shadow-lg);display:flex;flex-direction:column';

        const loadingDiv = document.createElement('div');
        loadingDiv.style.cssText = 'display:flex;align-items:center;justify-content:center;height:100%;color:var(--sp-text-tertiary)';
        loadingDiv.textContent = 'Loading files...';

        panel.append(loadingDiv);
        overlay.append(panel);
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
