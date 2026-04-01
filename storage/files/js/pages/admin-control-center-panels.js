import { applyFilter, setCurrentFilter } from '../services/control-center-feed.js';

export const initFilters = () => {
  for (const btn of document.querySelectorAll('.cc-filter-btn')) {
    btn.addEventListener('click', () => {
      for (const b of document.querySelectorAll('.cc-filter-btn')) b.classList.remove('cc-filter-btn--active');
      btn.classList.add('cc-filter-btn--active');
      setCurrentFilter(btn.getAttribute('data-filter') || '');
      applyFilter();
    });
  }
};

export const initTabs = (renderChart, renderSparklines) => {
  for (const tab of document.querySelectorAll('.cc-tab')) {
    tab.addEventListener('click', () => {
      const target = tab.getAttribute('data-tab');
      for (const t of document.querySelectorAll('.cc-tab')) t.classList.remove('cc-tab--active');
      for (const p of document.querySelectorAll('.cc-tab-panel')) p.classList.remove('cc-tab-panel--active');
      tab.classList.add('cc-tab--active');
      document.getElementById('tab-' + target)?.classList.add('cc-tab-panel--active');
      if (target === 'usage' && typeof renderChart === 'function') renderChart();
      if (target === 'report') renderSparklines();
    });
  }
};

export const initOnboardingTabs = () => {
  for (const tab of document.querySelectorAll('.cc-onboarding-tab')) {
    tab.addEventListener('click', () => {
      const target = 'onboard-' + tab.getAttribute('data-onboard');
      for (const t of document.querySelectorAll('.cc-onboarding-tab')) t.classList.remove('cc-onboarding-tab--active');
      for (const p of document.querySelectorAll('.cc-onboarding-panel')) p.classList.remove('cc-onboarding-panel--active');
      tab.classList.add('cc-onboarding-tab--active');
      document.getElementById(target)?.classList.add('cc-onboarding-panel--active');
    });
  }
};
