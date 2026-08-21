import { showToast } from './toast.js';

export const copyLinkPath = async (path, label = 'Invite link copied to clipboard') => {
  const url = window.location.origin + path;
  try {
    await navigator.clipboard.writeText(url);
    showToast(label, 'success');
  } catch {
    window.prompt('Copy the link:', url);
  }
  return url;
};
