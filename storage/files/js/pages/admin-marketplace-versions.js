import { apiFetch, rawFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';
import { on } from '../services/events.js';
import { renderVersionDetails, loadChangelog } from './admin-marketplace-versions-panel.js';

const versionDetails = {};
const diffCache = {};
let activeDiff = null;

const marketplaceApi = async (userId, path) => {
  return rawFetch('/api/public/marketplace/' + encodeURIComponent(userId) + path);
};

const loadVersionDetail = async (versionId, userId, container) => {
  if (versionDetails[versionId] && versionDetails[versionId] !== 'loading') {
    renderVersionDetails(container, versionId, versionDetails, activeDiff, diffCache);
  } else {
    versionDetails[versionId] = 'loading';
    try { versionDetails[versionId] = await marketplaceApi(userId, '/versions/' + encodeURIComponent(versionId)); }
    catch (e) { versionDetails[versionId] = 'error'; }
    renderVersionDetails(container, versionId, versionDetails, activeDiff, diffCache);
  }
};

const loadCoreDiff = async (skillId, baseSkillId, versionId, container) => {
  const ck = baseSkillId + ':' + skillId;
  let loadFailed = false;
  if (!diffCache[ck]) {
    try { diffCache[ck] = await apiFetch('/skills/' + encodeURIComponent(baseSkillId) + '/base-content'); }
    catch (err) { showToast('Failed to load core skill: ' + (err.message || 'Unknown error'), 'error'); loadFailed = true; }
  }
  if (!loadFailed) {
    activeDiff = { versionId, skillId, cacheKey: ck };
    renderVersionDetails(container, versionId, versionDetails, activeDiff, diffCache);
  }
};

const handleTabClick = async (tabBtn, root, activeTabRef, changelogLoaded) => {
  const nt = tabBtn.getAttribute('data-tab');
  if (activeTabRef.value !== nt) {
    activeTabRef.value = nt;
    for (const b of root.querySelectorAll('[data-tab]')) {
      b.className = b.getAttribute('data-tab') === activeTabRef.value ? 'btn btn-primary' : 'btn btn-secondary';
    }
    document.getElementById('mv-versions-tab').style.display = activeTabRef.value === 'versions' ? '' : 'none';
    document.getElementById('mv-changelog-tab').style.display = activeTabRef.value === 'changelog' ? '' : 'none';
    if (activeTabRef.value === 'changelog') {
      const uid = document.getElementById('mv-user-select')?.value;
      if (uid && !changelogLoaded[uid]) await loadChangelog(uid, changelogLoaded, marketplaceApi);
    }
  }
};

const handleToggle = async (toggleBtn, e) => {
  if (!e.target.closest('[data-restore-version]') && !e.target.closest('[data-compare-skill]')) {
    const card = toggleBtn.closest('.plugin-card');
    const details = card?.querySelector('.plugin-details');
    if (details) {
      const isHidden = details.style.display === 'none';
      details.style.display = isHidden ? '' : 'none';
      const icon = toggleBtn.querySelector('.expand-icon');
      if (icon) icon.style.transform = isHidden ? 'rotate(180deg)' : '';
      if (isHidden) await loadVersionDetail(toggleBtn.getAttribute('data-toggle-version'), toggleBtn.getAttribute('data-version-user'), details);
    }
  }
};

const handleCompare = async (compareBtn, e) => {
  e.stopPropagation();
  const sid = compareBtn.getAttribute('data-compare-skill');
  const bsid = compareBtn.getAttribute('data-base-skill');
  const cvid = compareBtn.getAttribute('data-compare-version');
  const vc = compareBtn.closest('.version-card');
  const dEl = vc?.querySelector('.plugin-details');
  if (dEl) await loadCoreDiff(sid, bsid, cvid, dEl);
};

const handleCloseDiff = (e) => {
  e.stopPropagation();
  activeDiff = null;
  const vc = e.target.closest('.version-card');
  const dEl = vc?.querySelector('.plugin-details');
  const tb = vc?.querySelector('[data-toggle-version]');
  if (dEl && tb) renderVersionDetails(dEl, tb.getAttribute('data-toggle-version'), versionDetails, activeDiff, diffCache);
};

const handleRestore = (restoreBtn, e) => {
  e.stopPropagation();
  const vid = restoreBtn.getAttribute('data-restore-version');
  const vnum = restoreBtn.getAttribute('data-restore-num');
  const uid = restoreBtn.getAttribute('data-restore-user');
  showConfirmDialog('Restore Version?', 'Restore to version ' + vnum + '? Current state will be saved as a new version.', 'Restore', async () => {
    try {
      const data = await rawFetch('/api/public/marketplace/' + encodeURIComponent(uid) + '/restore/' + encodeURIComponent(vid), { method: 'POST' });
      showToast('Restored to version ' + data.restored_version + '. ' + data.skills_restored + ' skills restored.', 'success');
      window.location.reload();
    } catch (err) { showToast(err.message || 'Restore failed', 'error'); }
  }, { btnClass: 'btn-primary' });
};

const handleRootClick = async (e, root, activeTabRef, changelogLoaded) => {
  const tabBtn = e.target.closest('[data-tab]');
  if (tabBtn) {
    await handleTabClick(tabBtn, root, activeTabRef, changelogLoaded);
  } else {
    const toggleBtn = e.target.closest('[data-toggle-version]');
    if (toggleBtn) {
      await handleToggle(toggleBtn, e);
    } else {
      const compareBtn = e.target.closest('[data-compare-skill]');
      if (compareBtn) {
        await handleCompare(compareBtn, e);
      } else if (e.target.closest('[data-close-diff]')) {
        handleCloseDiff(e);
      } else {
        const restoreBtn = e.target.closest('[data-restore-version]');
        if (restoreBtn) { handleRestore(restoreBtn, e); }
      }
    }
  }
};

export const initMarketplaceVersions = (selector) => {
  const root = document.querySelector(selector);
  if (root) {
    const activeTabRef = { value: 'versions' };
    const changelogLoaded = {};
    root.addEventListener('click', (e) => handleRootClick(e, root, activeTabRef, changelogLoaded));
    root.addEventListener('change', (e) => {
      if (e.target.id === 'mv-user-select') {
        const uid = e.target.value;
        for (const g of root.querySelectorAll('.version-user-group')) {
          const versions = g.querySelectorAll('[data-version-user]');
          let has = !uid;
          for (const v of versions) { if (v.getAttribute('data-version-user') === uid) has = true; }
          g.style.display = has ? '' : 'none';
        }
        if (activeTabRef.value === 'changelog' && uid) loadChangelog(uid, changelogLoaded, marketplaceApi);
      }
    });
    const urlUserId = new URLSearchParams(window.location.search).get('user_id');
    if (urlUserId) { const sel = document.getElementById('mv-user-select'); if (sel) { sel.value = urlUserId; sel.dispatchEvent(new Event('change')); } }
  }
};
