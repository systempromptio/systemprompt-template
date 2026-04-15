'use strict';

import { showToast } from '/js/services/toast.js';

const errorDiv = document.getElementById('error');
const loadingSection = document.getElementById('loading');
const loadingText = document.getElementById('loading-text');
const retrySection = document.getElementById('retry');
const loginForm = document.getElementById('login-form');
const emailInput = document.getElementById('login-email');
const magicLinkForm = document.getElementById('magic-link-form');
const magicLinkSent = document.getElementById('magic-link-sent');
const magicEmailInput = document.getElementById('magic-email');

export const getEmailInput = () => emailInput;

export async function clearAccessToken() {
  try {
    await fetch('/api/public/auth/session', { method: 'DELETE' });
  } catch (_err) {
    showToast('Failed to clear session. Please try again.', 'error');
  }
  document.cookie = 'access_token=; path=/; max-age=0; SameSite=Lax' +
    (window.location.protocol === 'https:' ? '; Secure' : '');
}

export async function showError(msg) {
  await clearAccessToken();
  errorDiv.textContent = msg;
  errorDiv.hidden = false;
  loadingSection.hidden = true;
  loginForm.hidden = true;
  retrySection.hidden = false;
}

export function showLoginForm() {
  loginForm.hidden = false;
  loadingSection.hidden = true;
  retrySection.hidden = true;
  errorDiv.hidden = true;
}

export function showLoading(msg) {
  loadingText.textContent = msg || 'Processing...';
  loginForm.hidden = true;
  loadingSection.hidden = false;
  retrySection.hidden = true;
  if (magicLinkForm) magicLinkForm.hidden = true;
}

export function showPasskeyError(error) {
  loadingSection.hidden = true;
  loginForm.hidden = false;
  if (error.name === 'NotAllowedError') errorDiv.textContent = 'Authentication was cancelled or not allowed.';
  else if (error.name === 'NotSupportedError') errorDiv.textContent = 'Passkeys are not supported on this device.';
  else errorDiv.textContent = error.message || 'Authentication failed. Please try again.';
  errorDiv.hidden = false;
}

export function showEmailError(msg) {
  errorDiv.textContent = msg;
  errorDiv.hidden = false;
}

export function hasValidAdminToken() {
  try {
    const cookie = document.cookie.split('; ').find(c => c.startsWith('access_token='));
    if (!cookie) return false;
    const token = cookie.split('=').slice(1).join('=');
    const payload = JSON.parse(atob(token.split('.')[1]));
    const scopes = (payload.scope || '').split(' ');
    if (!scopes.includes('user')) return false;
    if (payload.exp && payload.exp * 1000 < Date.now()) return false;
    return true;
  } catch (_err) { return false; }
}

export function initMagicLinkUI() {
  document.getElementById('magic-link-trigger').addEventListener('click', (e) => {
    e.preventDefault();
    loginForm.hidden = true;
    magicLinkForm.hidden = false;
    errorDiv.hidden = true;
    if (emailInput.value.trim()) magicEmailInput.value = emailInput.value.trim();
    magicEmailInput.focus();
  });

  document.getElementById('back-to-passkey').addEventListener('click', (e) => {
    e.preventDefault();
    magicLinkForm.hidden = true;
    loginForm.hidden = false;
    errorDiv.hidden = true;
  });

  document.getElementById('send-magic-btn').addEventListener('click', async () => {
    const email = magicEmailInput.value.trim();
    if (!email) {
      showEmailError('Please enter your email address.');
    } else {
      try {
        errorDiv.hidden = true;
        showLoading('Sending magic link...');
        const response = await fetch('/api/public/auth/magic-link', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ email }),
        });
        if (!response.ok) {
          const data = await response.json().catch(() => ({}));
          throw new Error(data.message || data.error || 'Failed to send magic link');
        }
        loadingSection.hidden = true;
        magicLinkForm.hidden = true;
        magicLinkSent.hidden = false;
      } catch (_err) {
        loadingSection.hidden = true;
        magicLinkForm.hidden = false;
        showToast(_err.message || 'Something went wrong. Please try again.', 'error');
      }
    }
  });

  magicEmailInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      document.getElementById('send-magic-btn').click();
    }
  });
}
