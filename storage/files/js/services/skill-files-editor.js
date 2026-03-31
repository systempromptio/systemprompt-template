const categoryLabels = {
  script: 'Scripts',
  reference: 'References',
  template: 'Templates',
  diagnostic: 'Diagnostics',
  data: 'Data',
  config: 'Config',
  asset: 'Assets',
};

const CATEGORY_ORDER = [
  'script', 'reference', 'template', 'diagnostic', 'data', 'config', 'asset',
];

const groupByCategory = (fileList) => {
  const groups = {};
  for (const f of fileList) {
    const cat = f.category || 'config';
    if (!groups[cat]) groups[cat] = [];
    groups[cat].push(f);
  }
  return groups;
};

export const validateContent = (content, lang) => {
  if (!content || !lang) return null;
  const l = lang.toLowerCase();
  try {
    if (l === 'json') { JSON.parse(content); return null; }
    if (l === 'yaml' || l === 'yml') {
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
  } catch (e) { return e.message; }
  return null;
};

export const buildFileListEl = (files, selectedFile) => {
  const container = document.createElement('div');
  if (!files.length) {
    const empty = document.createElement('div');
    empty.className = 'empty-state skill-editor-empty';
    const p = document.createElement('p');
    p.textContent = 'No files found for this skill.';
    const hint = document.createElement('p');
    hint.className = 'skill-editor-hint';
    hint.textContent = 'Click "Sync Files" to scan the filesystem.';
    empty.append(p);
    empty.append(hint);
    container.append(empty);
    return container;
  }
  const groups = groupByCategory(files);
  for (const cat of CATEGORY_ORDER) {
    const group = groups[cat];
    if (!group?.length) continue;
    const section = document.createElement('div');
    section.className = 'skill-editor-section';
    const catLabel = document.createElement('div');
    catLabel.className = 'skill-file-category';
    catLabel.textContent = (categoryLabels[cat] || cat) + ' (' + group.length + ')';
    section.append(catLabel);
    for (const f of group) {
      const item = document.createElement('div');
      item.className = 'skill-file-item' + (selectedFile?.id === f.id ? ' selected' : '');
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
      section.append(item);
    }
    container.append(section);
  }
  return container;
};

const buildEditorHeader = (selectedFile) => {
  const header = document.createElement('div');
  header.className = 'skill-editor-header';
  const pathSpan = document.createElement('span');
  pathSpan.className = 'skill-editor-path';
  pathSpan.textContent = selectedFile.file_path;
  header.append(pathSpan);
  const langBadge = document.createElement('span');
  langBadge.className = 'badge badge-blue skill-editor-badge';
  langBadge.textContent = selectedFile.language || 'text';
  header.append(langBadge);
  if (selectedFile.executable) {
    const execBadge = document.createElement('span');
    execBadge.className = 'badge badge-green skill-editor-badge';
    execBadge.textContent = 'executable';
    header.append(execBadge);
  }
  const sizeSpan = document.createElement('span');
  sizeSpan.className = 'skill-editor-size';
  sizeSpan.textContent = selectedFile.size_bytes + ' bytes';
  header.append(sizeSpan);
  return header;
};

const buildEditorFooter = () => {
  const footer = document.createElement('div');
  footer.className = 'skill-editor-footer';
  const validation = document.createElement('span');
  validation.id = 'skill-file-validation';
  validation.className = 'skill-editor-validation';
  footer.append(validation);
  const saveBtn = document.createElement('button');
  saveBtn.className = 'btn btn-primary btn-sm skill-modal-btn';
  saveBtn.id = 'skill-file-save';
  saveBtn.textContent = 'Save';
  footer.append(saveBtn);
  return footer;
};

export const buildEditorEl = (selectedFile) => {
  const wrapper = document.createElement('div');
  if (!selectedFile) {
    wrapper.className = 'skill-editor-placeholder';
    wrapper.textContent = 'Select a file to view its contents';
    return wrapper;
  }
  wrapper.className = 'skill-editor-wrapper';
  wrapper.append(buildEditorHeader(selectedFile));
  const textarea = document.createElement('textarea');
  textarea.id = 'skill-file-editor';
  textarea.className = 'skill-editor-textarea';
  textarea.value = selectedFile.content || '';
  wrapper.append(textarea);
  wrapper.append(buildEditorFooter());
  return wrapper;
};
