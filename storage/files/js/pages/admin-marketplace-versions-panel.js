import { truncate } from '../utils/dom.js';
import { renderDiffPanel, buildChangelogTable } from './admin-marketplace-versions-panel-helpers.js';

const mk = (tag, props = {}, children = []) => { const node = Object.assign(document.createElement(tag), props); for (const c of children) node.append(typeof c === 'string' ? document.createTextNode(c) : c); return node; };

export const setEmpty = (el, msg) => { el.textContent = ''; const empty = mk('div', { className: 'empty-state' }, [mk('p', { textContent: msg })]); el.append(empty); };

export const showLoadingSpinner = (container) => {
  container.textContent = '';
  const spinner = mk('div', { className: 'loading-spinner' });
  spinner.setAttribute('role', 'status');
  spinner.append(mk('span', { className: 'sr-only', textContent: 'Loading...' }));
  container.append(mk('div', { className: 'loading-center' }, [spinner]));
};

export const renderSkillRow = (skill, versionId, activeDiff) => {
  const hasBase = skill.base_skill_id && skill.base_skill_id !== 'null';
  const info = mk('div', { className: 'detail-item-info' });
  const nameDiv = mk('div', { className: 'detail-item-name' });
  nameDiv.append(document.createTextNode(skill.name || skill.skill_id));
  nameDiv.append(document.createTextNode(' '));
  nameDiv.append(mk('span', { className: hasBase ? 'badge badge-yellow' : 'badge badge-gray', textContent: hasBase ? 'customized' : 'custom' }));
  nameDiv.append(document.createTextNode(' '));
  nameDiv.append(mk('span', { className: skill.enabled === false ? 'badge badge-red' : 'badge badge-green', textContent: skill.enabled === false ? 'disabled' : 'enabled' }));
  info.append(nameDiv);
  const metaDiv = mk('div', { style: 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:2px' });
  metaDiv.append(mk('code', { style: 'background:var(--sp-bg-surface-raised);padding:1px 6px;border-radius:var(--sp-radius-xs)', textContent: skill.skill_id }));
  if (skill.version) metaDiv.append(mk('span', { textContent: ' v' + skill.version }));
  if (skill.description) metaDiv.append(document.createTextNode(' \u2014 ' + truncate(skill.description, 80)));
  info.append(metaDiv);
  const item = mk('div', { className: 'detail-item' }, [info]);
  if (hasBase) {
    const isActive = activeDiff && activeDiff.versionId === versionId && activeDiff.skillId === skill.skill_id;
    const btn = mk('button', { className: 'btn btn-secondary btn-sm', style: 'font-size:var(--sp-text-xs);padding:2px 8px;white-space:nowrap', textContent: isActive ? 'Viewing Diff' : 'Compare to Core' });
    btn.setAttribute('data-compare-skill', skill.skill_id);
    btn.setAttribute('data-compare-version', versionId);
    btn.setAttribute('data-base-skill', skill.base_skill_id);
    if (isActive) btn.disabled = true;
    item.append(btn);
  }
  return item;
};

export const renderVersionDetails = (container, versionId, versionDetails, activeDiff, diffCache) => {
  const detail = versionDetails[versionId];
  if (detail && detail !== 'loading') {
    if (detail === 'error') {
      container.textContent = '';
      const w = mk('div', { style: 'padding:var(--sp-space-4)' }, [mk('div', { className: 'empty-state' }, [mk('p', { textContent: 'Failed to load version details.' })])]);
      container.append(w);
    } else {
      let skills = [];
      if (Array.isArray(detail.skills_snapshot)) skills = detail.skills_snapshot;
      else if (typeof detail.skills_snapshot === 'string') { try { skills = JSON.parse(detail.skills_snapshot); } catch (e) { skills = []; } }
      container.textContent = '';
      const wrapper = mk('div', { style: 'padding:var(--sp-space-4)' });
      wrapper.append(mk('div', { style: 'font-size:var(--sp-text-sm);font-weight:600;margin-bottom:var(--sp-space-2);color:var(--sp-text-secondary)', textContent: 'Skills Snapshot (' + skills.length + ')' }));
      if (skills.length) { for (const s of skills) wrapper.append(renderSkillRow(s, versionId, activeDiff)); }
      else { wrapper.append(mk('div', { className: 'empty-state', style: 'padding:var(--sp-space-4)' }, [mk('p', { textContent: 'No skills in this snapshot.' })])); }
      container.append(wrapper);
      if (activeDiff?.versionId === versionId && diffCache[activeDiff.cacheKey]) { const us = skills.find((s) => s.skill_id === activeDiff.skillId); if (us) container.append(renderDiffPanel(us, diffCache[activeDiff.cacheKey])); }
    }
  }
};

export const loadChangelog = async (userId, loaded, marketplaceApi) => {
  const container = document.getElementById('mv-changelog-tab');
  if (!container || !userId) {
    if (container) setEmpty(container, 'Select a user to view changelog.');
  } else {
    showLoadingSpinner(container);
    try {
      const changelog = await marketplaceApi(userId, '/changelog');
      loaded[userId] = true;
      if (!changelog?.length) {
        setEmpty(container, 'No changelog entries found for this user.');
      } else {
        container.textContent = '';
        container.append(buildChangelogTable(changelog));
      }
    } catch (e) { setEmpty(container, 'Failed to load changelog.'); }
  }
};
