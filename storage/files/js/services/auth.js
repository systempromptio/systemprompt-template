import { escapeHtml } from '../utils/dom.js';
import { API_BASE, BASE } from './api.js';

export const getUser = () => {
  try {
    const cookie = document.cookie.split(';').find((c) => c.trim().startsWith('access_token='));
    if (!cookie) return null;
    const token = cookie.trim().split('=')[1];
    const payload = JSON.parse(atob(token.split('.')[1]));
    return { id: payload.sub, username: payload.username, email: payload.email };
  } catch (_e) {
    return null;
  }
};

export const getUserInitials = (name) => {
  if (!name) return '?';
  return name.split(/[\s@._-]/).filter(Boolean).slice(0, 2).map((s) => s[0].toUpperCase()).join('');
};

export const getUserContext = async () => {
  try {
    const resp = await fetch('/admin/auth/me');
    if (!resp.ok) return null;
    const me = await resp.json();
    const meta = document.getElementById('user-meta');
    if (meta) {
      const parts = (me.roles || [])
        .filter((role) => role !== 'user')
        .map((role) => escapeHtml(role.charAt(0).toUpperCase() + role.slice(1)));
      meta.textContent = parts.join(' \u00b7 ');
    }
    return me;
  } catch (_e) {
    return null;
  }
};

export const initLogout = () => {
  const btn = document.getElementById('btn-logout');
  if (btn) {
    btn.addEventListener('click', () => {
      fetch(API_BASE.replace('/admin', '') + '/auth/session', { method: 'DELETE' })
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
    img.className = 'user-widget__avatar-img';
    av.appendChild(img);
  } else {
    av.appendChild(document.createTextNode(initials));
  }
  const nm = document.getElementById('user-name');
  if (nm && name) nm.textContent = name;
};
