import { API_BASE, BASE, rawResponse } from './api.js';

export const getUser = () => {
  try {
    const cookie = document.cookie.split(';').find((c) => c.trim().startsWith('access_token='));
    if (!cookie) return null;
    const token = cookie.trim().split('=')[1];
    const payload = JSON.parse(atob(token.split('.')[1]));
    return { id: payload.sub, username: payload.username, email: payload.email };
  } catch {
    return null;
  }
};

export const getUserInitials = (name) => {
  if (!name) return '?';
  return name.split(/[\s@._-]/).filter(Boolean).slice(0, 2).map((s) => s[0].toUpperCase()).join('');
};

export const getUserContext = async () => {
  try {
    const resp = await rawResponse('/admin/auth/me');
    if (!resp.ok) return null;
    const me = await resp.json();
    const meta = document.getElementById('sp-user-meta');
    if (meta) {
      const parts = (me.roles || [])
        .filter((role) => role !== 'user')
        .map((role) => role.charAt(0).toUpperCase() + role.slice(1));
      meta.textContent = parts.join(' \u00b7 ');
    }
    return me;
  } catch {
    return null;
  }
};

let logoutReady = false;

export const initLogout = () => {
  const btn = document.getElementById('btn-logout');
  if (btn && !logoutReady) {
    logoutReady = true;
    btn.addEventListener('click', () => {
      rawResponse(API_BASE.replace('/admin', '') + '/auth/session', { method: 'DELETE' })
        .finally(() => {
          sessionStorage.clear();
          window.location.href = BASE;
        });
    });
  }
};

export const initUserDisplay = async () => {
  const av = document.getElementById('user-avatar');
  if (!av) return;
  const me = await getUserContext();
  const name = (me && (me.username || me.email)) || '';
  const initials = getUserInitials(name);
  av.textContent = '';
  if (me && me.avatar_url) {
    const img = document.createElement('img');
    img.src = me.avatar_url;
    img.alt = name || 'User avatar';
    img.className = 'sp-avatar__img';
    av.append(img);
  } else {
    av.append(initials);
  }
  const nm = document.getElementById('sp-user-name');
  if (nm && name) nm.textContent = name;
};
