import { showToast } from './toast.js';

export const BASE = '/admin';
export const API_BASE = '/api/public/admin';

export const apiFetch = async (path, options = {}) => {
  const url = API_BASE + path;
  const resp = await fetch(url, {
    headers: { 'Content-Type': 'application/json' },
    ...options
  });
  if (!resp.ok) {
    const text = await resp.text();
    let message = resp.statusText;
    try {
      const json = JSON.parse(text);
      message = json.error || json.message || text;
    } catch { message = text || resp.statusText; }
    showToast(message, 'error');
    throw new Error(message);
  }
  const ct = resp.headers.get('content-type') || '';
  if (resp.status === 204 || !ct.includes('application/json')) return null;
  return resp.json();
};

export const apiGet = (path) => apiFetch(path);

export const rawFetch = async (url, options = {}) => {
  const resp = await fetch(url, {
    headers: { 'Content-Type': 'application/json' },
    ...options
  });
  if (!resp.ok) {
    const text = await resp.text();
    let message = resp.statusText;
    try {
      const json = JSON.parse(text);
      message = json.error || json.message || text;
    } catch { message = text || resp.statusText; }
    showToast(message, 'error');
    throw new Error(message);
  }
  const ct = resp.headers.get('content-type') || '';
  if (resp.status === 204 || !ct.includes('application/json')) return null;
  return resp.json();
};
