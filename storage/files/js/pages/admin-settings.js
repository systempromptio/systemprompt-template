import { apiFetch } from '../services/api.js';
import { showToast } from '../services/toast.js';
import { showConfirmDialog } from '../services/confirm.js';

const COMMON_TIMEZONES = [
  'Pacific/Auckland', 'Australia/Sydney', 'Australia/Adelaide', 'Australia/Perth',
  'Asia/Tokyo', 'Asia/Seoul', 'Asia/Shanghai', 'Asia/Hong_Kong', 'Asia/Singapore',
  'Asia/Kolkata', 'Asia/Dubai', 'Europe/Moscow', 'Europe/Istanbul',
  'Europe/Helsinki', 'Europe/Bucharest', 'Europe/Athens',
  'Europe/Berlin', 'Europe/Paris', 'Europe/Amsterdam', 'Europe/Rome', 'Europe/Zurich',
  'Europe/London', 'Europe/Dublin', 'Europe/Lisbon',
  'Atlantic/Reykjavik', 'America/Sao_Paulo', 'America/Argentina/Buenos_Aires',
  'America/New_York', 'America/Toronto', 'America/Chicago', 'America/Denver',
  'America/Phoenix', 'America/Los_Angeles', 'America/Vancouver', 'America/Anchorage',
  'Pacific/Honolulu', 'UTC',
];

const populateTimezoneSelect = (select, savedValue) => {
  select.innerHTML = '';

  const detected = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const allZones = new Set([...COMMON_TIMEZONES]);
  if (detected) allZones.add(detected);
  if (savedValue && savedValue !== 'UTC') allZones.add(savedValue);

  const sorted = [...allZones].sort((a, b) => {
    const now = new Date();
    const offsetA = getOffsetMinutes(now, a);
    const offsetB = getOffsetMinutes(now, b);
    return offsetA - offsetB;
  });

  for (const tz of sorted) {
    const opt = document.createElement('option');
    opt.value = tz;
    const offset = formatOffset(new Date(), tz);
    opt.textContent = `(${offset}) ${tz.replace(/_/g, ' ')}`;
    select.appendChild(opt);
  }

  const preferred = savedValue || detected || 'UTC';
  if ([...select.options].some(o => o.value === preferred)) {
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

const formatOffset = (date, tz) => {
  const mins = getOffsetMinutes(date, tz);
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
      fallback.className = 'settings-avatar-placeholder';
      fallback.textContent = '!';
      container.replaceChildren(fallback);
    });
    container.replaceChildren(img);
  } else {
    const placeholder = document.createElement('span');
    placeholder.className = 'settings-avatar-placeholder';
    placeholder.textContent = '?';
    container.replaceChildren(placeholder);
  }
};

const collectFormData = () => ({
  display_name: document.getElementById('settings-display-name')?.value?.trim() || null,
  avatar_url: document.getElementById('settings-avatar-url')?.value?.trim() || null,
  notify_daily_summary: document.getElementById('settings-notify-daily-summary')?.checked ?? true,
  notify_achievements: document.getElementById('settings-notify-achievements')?.checked ?? true,
  leaderboard_opt_in: document.getElementById('settings-leaderboard-opt-in')?.checked ?? true,
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
    saveBtn.textContent = 'Save Settings';
  }
};

export const initSettingsPage = () => {
  const form = document.getElementById('settings-form');
  const saveBtn = document.getElementById('save-settings-btn');
  const avatarInput = document.getElementById('settings-avatar-url');
  const avatarPreview = document.getElementById('avatar-preview');
  const timezoneSelect = document.getElementById('settings-timezone');
  const savedTimezone = document.getElementById('settings-timezone-saved')?.value || '';

  if (timezoneSelect) {
    populateTimezoneSelect(timezoneSelect, savedTimezone);
  }

  if (form) {
    if (avatarInput && avatarPreview) {
      avatarInput.addEventListener('input', () => {
        updateAvatarPreview(avatarPreview, avatarInput.value);
      });
    }

    form.addEventListener('submit', (e) => {
      e.preventDefault();
      saveSettings(saveBtn);
    });
  }

  const deleteBtn = document.getElementById('delete-account-btn');
  if (deleteBtn) {
    deleteBtn.addEventListener('click', () => {
      showConfirmDialog(
        'Delete Account',
        'This will permanently delete your account and all your data. This cannot be undone.',
        'Delete My Account',
        async () => {
          deleteBtn.disabled = true;
          deleteBtn.textContent = 'Deleting...';
          try {
            await apiFetch('/user/account', { method: 'DELETE' });
            window.location.href = '/';
          } catch {
            showToast('Failed to delete account', 'error');
            deleteBtn.disabled = false;
            deleteBtn.textContent = 'Delete Account';
          }
        },
      );
    });
  }
};

initSettingsPage();
