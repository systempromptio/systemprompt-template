import { initEditForm } from '../services/list-page.js';
import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const initSecretSave = (currentSkillId) => {
  for (const btn of document.querySelectorAll('.save-secret-btn')) {
    btn.addEventListener('click', async () => {
      const row = btn.closest('tr');
      const secretName = row.getAttribute('data-secret-name');
      const input = row.querySelector('.secret-value-input');
      const value = input.value.trim();
      if (!value) {
        showToast('Please enter a value', 'error');
        return;
      }
      try {
        await apiFetch(
          `/user/skills/${encodeURIComponent(currentSkillId)}/secrets`,
          {
            method: 'PUT',
            body: JSON.stringify({ var_name: secretName, var_value: value }),
          },
        );
        showToast('Secret saved', 'success');
        input.value = '';
        const badge = row.querySelector('.badge');
        if (badge) {
          badge.className = 'badge badge-green';
          badge.textContent = 'Configured';
        }
      } catch {
      }
    });
  }
};

const initSecretDelete = (currentSkillId) => {
  for (const btn of document.querySelectorAll('.delete-secret-btn')) {
    btn.addEventListener('click', () => {
      const row = btn.closest('tr');
      const secretName = row.getAttribute('data-secret-name');
      showConfirmDialog(
        'Delete Secret',
        `Delete secret ${secretName}?`,
        'Delete',
        async () => {
          try {
            await apiFetch(
              `/user/skills/${encodeURIComponent(currentSkillId)}/secrets/${encodeURIComponent(secretName)}`,
              { method: 'DELETE' },
            );
            showToast('Secret deleted', 'success');
            const badge = row.querySelector('.badge');
            if (badge) {
              badge.className = 'badge badge-red';
              badge.textContent = 'Missing';
            }
            btn.remove();
          } catch {
          }
        },
      );
    });
  }
};

const init = () => {
  initEditForm('my-skill-form', {
    buildBody: (form, formData) => {
      const tagsRaw = formData.get('tags') || '';
      const tags = tagsRaw.split(',').map(t => t.trim()).filter(Boolean);
      return {
        skill_id: formData.get('skill_id'),
        name: formData.get('name'),
        description: formData.get('description') || '',
        content: formData.get('content') || '',
        tags,
      };
    },
  });

  const skillIdEl = document.querySelector('[name="skill_id"]');
  const currentSkillId = skillIdEl?.value || '';
  if (currentSkillId) {
    initSecretSave(currentSkillId);
    initSecretDelete(currentSkillId);
  }
};

init();
