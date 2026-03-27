(function(app) {
    'use strict';

    function copyToClipboard(text, btn) {
        navigator.clipboard.writeText(text).then(function() {
            const orig = btn.textContent;
            btn.textContent = 'Copied!';
            btn.classList.add('copied');
            setTimeout(function() {
                btn.textContent = orig;
                btn.classList.remove('copied');
            }, 2000);
        }).catch(function() {
            app.Toast.show('Failed to copy to clipboard', 'error');
        });
    }

    const SAFE_PATH_RE = /^[a-zA-Z0-9_\-./]+$/;
    function safeDelimiter(idx) {
        return 'EOF_SP_' + idx;
    }
    function sanitizePath(p) {
        if (!SAFE_PATH_RE.test(p)) {
            throw new Error('Invalid file path: ' + p);
        }
        return p;
    }
    function generateInstallScript(data) {
        const lines = ['#!/bin/bash', '# Install script for Foodles plugins', 'set -e', ''];
        const plugins = data.plugins || [];
        let delimIdx = 0;
        for (let i = 0; i < plugins.length; i++) {
            const plugin = plugins[i];
            const files = plugin.files || [];
            const safeId = sanitizePath(plugin.id);
            lines.push('# Plugin: ' + safeId);
            lines.push('echo "Installing plugin: ' + safeId + '"');
            for (let j = 0; j < files.length; j++) {
                const file = files[j];
                const safePath = sanitizePath(file.path);
                const filePath = '~/.claude/plugins/' + safeId + '/' + safePath;
                const dirPath = filePath.substring(0, filePath.lastIndexOf('/'));
                const delim = safeDelimiter(delimIdx++);
                lines.push('mkdir -p "' + dirPath + '"');
                lines.push("cat > \"" + filePath + "\" << '" + delim + "'");
                lines.push(file.content);
                lines.push(delim);
                if (file.executable) {
                    lines.push('chmod +x "' + filePath + '"');
                }
                lines.push('');
            }
        }
        if (data.marketplace) {
            const mktPath = sanitizePath(data.marketplace.path);
            const mktDelim = safeDelimiter(delimIdx++);
            lines.push('# Marketplace manifest');
            lines.push('mkdir -p ~/.claude/plugins/.claude-plugin');
            lines.push("cat > ~/.claude/plugins/" + mktPath + " << '" + mktDelim + "'");
            lines.push(data.marketplace.content);
            lines.push(mktDelim);
        }
        lines.push('');
        lines.push('echo "All plugins installed successfully."');
        return lines.join('\n');
    }

    function toggleBundle(idx) {
        const details = document.getElementById('bundle-details-' + idx);
        const icon = document.getElementById('bundle-icon-' + idx);
        if (!details) return;
        const open = details.style.display !== 'none';
        details.style.display = open ? 'none' : 'block';
        if (icon) icon.innerHTML = open ? '&#9654;' : '&#9660;';
    }

    async function downloadZip(data) {
        const btn = document.getElementById('btn-download-zip');
        if (!btn) return;
        const origHtml = btn.innerHTML;
        btn.innerHTML = 'Generating...';
        btn.disabled = true;
        try {
            const JSZip = await app.shared.loadJSZip();
            const zip = new JSZip();
            const plugins = data.plugins || [];
            plugins.forEach(function(plugin) {
                const folder = zip.folder(plugin.id);
                (plugin.files || []).forEach(function(file) {
                    const opts = file.executable ? { unixPermissions: '755' } : {};
                    folder.file(file.path, file.content, opts);
                });
            });
            if (data.marketplace) {
                zip.file(data.marketplace.path, data.marketplace.content);
            }
            const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'foodles-plugins.zip';
            document.body.append(a);
            a.click();
            a.remove();
            URL.revokeObjectURL(url);
            btn.innerHTML = origHtml;
            btn.disabled = false;
            app.Toast.show('ZIP downloaded successfully', 'success');
        } catch (err) {
            btn.innerHTML = origHtml;
            btn.disabled = false;
            app.Toast.show('Failed to generate ZIP: ' + err.message, 'error');
        }
    }

    app.exportInteractions = function(exportData) {
        if (!exportData) return;

        app.events.on('click', '#btn-download-zip', function() {
            downloadZip(exportData);
        });

        app.events.on('click', '[data-action="toggle-bundle"]', function(e, el) {
            toggleBundle(el.getAttribute('data-idx'));
        });

        app.events.on('click', '[data-action="copy-content"]', function(e, el) {
            const pluginIdx = parseInt(el.getAttribute('data-plugin-idx'), 10);
            const fileIdx = parseInt(el.getAttribute('data-file-idx'), 10);
            const plugin = (exportData.plugins || [])[pluginIdx];
            if (plugin) {
                const file = (plugin.files || [])[fileIdx];
                if (file) copyToClipboard(file.content, el);
            }
        });

        app.events.on('click', '[data-action="copy-script"]', function(e, el) {
            const script = generateInstallScript(exportData);
            copyToClipboard(script, el);
        });
    };
})(window.AdminApp);
