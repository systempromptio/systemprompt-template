import { showToast } from '../services/toast.js';
import { on } from '../services/events.js';

const SAFE_PATH_RE = /^[a-zA-Z0-9_\-./]+$/;

const sanitizePath = (p) => {
  if (!SAFE_PATH_RE.test(p)) throw new Error('Invalid file path: ' + p);
  return p;
};

const generateInstallScript = (data) => {
  const lines = ['#!/bin/bash', 'set -e', ''];
  let delimIdx = 0;
  for (const plugin of (data.plugins || [])) {
    const safeId = sanitizePath(plugin.id);
    lines.push('echo "Installing plugin: ' + safeId + '"');
    for (const file of (plugin.files || [])) {
      const safePath = sanitizePath(file.path);
      const filePath = '~/.claude/plugins/' + safeId + '/' + safePath;
      const dirPath = filePath.substring(0, filePath.lastIndexOf('/'));
      const delim = 'EOF_SP_' + delimIdx++;
      lines.push('mkdir -p "' + dirPath + '"');
      lines.push("cat > \"" + filePath + "\" << '" + delim + "'");
      lines.push(file.content);
      lines.push(delim);
      if (file.executable) lines.push('chmod +x "' + filePath + '"');
      lines.push('');
    }
  }
  if (data.marketplace) {
    const mktDelim = 'EOF_SP_' + delimIdx++;
    lines.push('mkdir -p ~/.claude/plugins/.claude-plugin');
    lines.push("cat > ~/.claude/plugins/" + sanitizePath(data.marketplace.path) + " << '" + mktDelim + "'");
    lines.push(data.marketplace.content);
    lines.push(mktDelim);
  }
  lines.push('', 'echo "All plugins installed successfully."');
  return lines.join('\n');
};

const copyToClipboard = async (text, btn) => {
  try {
    await navigator.clipboard.writeText(text);
    const orig = btn.textContent;
    btn.textContent = 'Copied!';
    btn.classList.add('copied');
    setTimeout(() => { btn.textContent = orig; btn.classList.remove('copied'); }, 2000);
  } catch (_err) {
    showToast('Failed to copy to clipboard', 'error');
  }
};

const loadJSZip = () => new Promise((resolve, reject) => {
  if (window.JSZip) {
    resolve(window.JSZip);
  } else {
    const script = document.createElement('script');
    script.src = 'https://cdnjs.cloudflare.com/ajax/libs/jszip/3.10.1/jszip.min.js';
    script.integrity = 'sha384-+mbV2IY1Zk/X1p/nWllGySJSUN8uMs+gUAN10Or95UBH0fpj6GfKgPmgC5EXieXG';
    script.crossOrigin = 'anonymous';
    script.onload = () => resolve(window.JSZip);
    script.onerror = () => reject(new Error('Failed to load JSZip'));
    document.head.append(script);
  }
});

const downloadZip = async (data) => {
  const btn = document.getElementById('btn-download-zip');
  if (btn) {
    const origText = btn.textContent;
    btn.textContent = 'Generating...';
    btn.disabled = true;
    try {
      const JSZip = await loadJSZip();
      const zip = new JSZip();
      for (const plugin of (data.plugins || [])) {
        const folder = zip.folder(plugin.id);
        for (const file of (plugin.files || [])) {
          folder.file(file.path, file.content, file.executable ? { unixPermissions: '755' } : {});
        }
      }
      if (data.marketplace) zip.file(data.marketplace.path, data.marketplace.content);
      const blob = await zip.generateAsync({ type: 'blob', platform: 'UNIX' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url; a.download = 'systemprompt-plugins.zip';
      document.body.append(a); a.click(); a.remove();
      URL.revokeObjectURL(url);
      showToast('ZIP downloaded successfully', 'success');
    } catch (err) {
      showToast('Failed to generate ZIP: ' + err.message, 'error');
    } finally {
      btn.textContent = origText; btn.disabled = false;
    }
  }
};

export const exportInteractions = (exportData) => {
  if (exportData) {
    on('click', '#btn-download-zip', () => downloadZip(exportData));

    on('click', '[data-action="toggle-bundle"]', (e, el) => {
      const idx = el.getAttribute('data-idx');
      const details = document.getElementById('bundle-details-' + idx);
      const icon = document.getElementById('bundle-icon-' + idx);
      if (details) {
        const open = !details.hidden;
        details.hidden = open;
        if (icon) icon.textContent = open ? '\u25B6' : '\u25BC';
      }
    });

    on('click', '[data-action="copy-content"]', (e, el) => {
      const pluginIdx = parseInt(el.getAttribute('data-plugin-idx'), 10);
      const fileIdx = parseInt(el.getAttribute('data-file-idx'), 10);
      const plugin = (exportData.plugins || [])[pluginIdx];
      if (plugin) {
        const file = (plugin.files || [])[fileIdx];
        if (file) copyToClipboard(file.content, el);
      }
    });

    on('click', '[data-action="copy-script"]', (e, el) => {
      copyToClipboard(generateInstallScript(exportData), el);
    });
  }
};
