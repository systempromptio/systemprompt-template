const DISMISS_KEY = 'sp-onboarding-banner-dismissed';

export const initOnboardingBanner = () => {
  const banner = document.getElementById('onboarding-banner');
  const dismiss = document.getElementById('onboarding-banner-dismiss');
  if (!banner || !dismiss) {
    return;
  }

  if (localStorage.getItem(DISMISS_KEY)) {
    banner.hidden = true;
    return;
  }

  dismiss.addEventListener('click', () => {
    banner.hidden = true;
    localStorage.setItem(DISMISS_KEY, Date.now().toString());
  });
};
