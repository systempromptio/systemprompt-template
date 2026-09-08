import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const zoneNames = (detected, savedValue) => {
  const zones = new Set(Intl.supportedValuesOf('timeZone'));
  zones.add('UTC');
  if (detected) zones.add(detected);
  if (savedValue) zones.add(savedValue);
  return [...zones];
};

const zoneOption = (tz, offsetMinutes) => {
  const option = document.createElement('option');
  option.value = tz;
  option.textContent = `(${formatOffset(offsetMinutes)}) ${tz.replace(/_/g, ' ')}`;
  return option;
};

const populateTimezoneSelect = (select, savedValue) => {
  const now = new Date();
  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const zones = zoneNames(detected, savedValue)
    .map((tz) => ({ tz, offset: getOffsetMinutes(now, tz) }))
    .sort((a, b) => a.offset - b.offset || a.tz.localeCompare(b.tz));

  select.replaceChildren(...zones.map(({ tz, offset }) => zoneOption(tz, offset)));

  const preferred = savedValue || detected || 'UTC';
  if ([...select.options].some((o) => o.value === preferred)) {
    select.value = preferred;
  }
};

const getOffsetMinutes = (date, tz) => {
  try {
    const str = date.toLocaleString('en-US', { timeZone: tz });
    const local = new Date(str);
    const utcStr = date.toLocaleString('en-US', { timeZone: 'UTC' });
    const utc = new Date(utcStr);
    return (local - utc) / 60000;
  } catch {
    return 0;
  }
};

const formatOffset = (mins) => {
  const sign = mins >= 0 ? '+' : '-';
  const abs = Math.abs(mins);
  const h = String(Math.floor(abs / 60)).padStart(2, '0');
  const m = String(abs % 60).padStart(2, '0');
  return `UTC${sign}${h}:${m}`;
};

const updateAvatarPreview = (container, url) => {
  const trimmed = url.trim();
  if (trimmed) {
    const img = document.createElement('img');
    img.src = trimmed;
    img.alt = 'Avatar preview';
    img.addEventListener('error', () => {
      const fallback = document.createElement('span');
      fallback.className = 'sp-p-settings__avatar-fallback';
      fallback.textContent = '!';
      container.replaceChildren(fallback);
    });
    container.replaceChildren(img);
  } else {
    const placeholder = document.createElement('span');
    placeholder.className = 'sp-p-settings__avatar-fallback';
    placeholder.textContent = '?';
    container.replaceChildren(placeholder);
  }
};

const collectFormData = () => ({
  display_name: document.getElementById('settings-display-name')?.value?.trim() || null,
  avatar_url: document.getElementById('settings-avatar-url')?.value?.trim() || null,
  timezone: document.getElementById('settings-timezone')?.value || 'UTC',
});

const saveSettings = async (saveBtn) => {
  saveBtn.disabled = true;
  saveBtn.textContent = 'Saving...';
  try {
    await apiFetch('/user/settings', {
      method: 'PUT',
      body: JSON.stringify(collectFormData()),
    });
    showToast('Settings saved', 'success');
  } catch {
    showToast('Failed to save settings', 'error');
  } finally {
    saveBtn.disabled = false;
    saveBtn.textContent = 'Save settings';
  }
};

const deleteAccount = async (deleteBtn) => {
  deleteBtn.disabled = true;
  deleteBtn.textContent = 'Deleting...';
  try {
    // The route refuses a bodyless DELETE: an irreversible action that any
    // stray request could complete is one nobody meant to trigger.
    await apiFetch('/user/account', {
      method: 'DELETE',
      body: JSON.stringify({ confirm_email: document.getElementById('settings-email')?.value ?? '' }),
    });
    window.location.href = '/';
  } catch {
    showToast('Failed to delete account', 'error');
    deleteBtn.disabled = false;
    deleteBtn.textContent = 'Delete account';
  }
};

const bindDeleteAccount = () => {
  const deleteBtn = document.getElementById('delete-account-btn');
  deleteBtn?.addEventListener('click', () => {
    showConfirmDialog(
      'Delete account',
      'This will permanently delete your account and all your data. This cannot be undone.',
      'Delete my account',
      () => deleteAccount(deleteBtn),
    );
  });
};

const bindForm = () => {
  const form = document.getElementById('settings-form');
  if (!form) return;
  const avatarInput = document.getElementById('settings-avatar-url');
  const avatarPreview = document.getElementById('avatar-preview');
  if (avatarInput && avatarPreview) {
    avatarInput.addEventListener('input', () => {
      updateAvatarPreview(avatarPreview, avatarInput.value);
    });
  }
  form.addEventListener('submit', (ev) => {
    ev.preventDefault();
    saveSettings(document.getElementById('save-settings-btn'));
  });
};

export const initSettingsPage = () => {
  const timezoneSelect = document.getElementById('settings-timezone');
  if (timezoneSelect) {
    populateTimezoneSelect(timezoneSelect, document.getElementById('settings-timezone-saved')?.value || '');
  }
  bindForm();
  bindDeleteAccount();
};

initSettingsPage();
