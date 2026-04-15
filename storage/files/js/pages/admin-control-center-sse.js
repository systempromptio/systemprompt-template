import { createSSEClient } from '../services/sse-client.js';
import {
  setStatValue, updateSkillCards, updateHealthScore,
  updateSessionRatings, updateSkillAdoption,
  updateDailySummary, updateAchievementProgress,
  updateApmMetrics, updatePerformanceSummary,
  updateHourlyCharts, update7dData, update30dData,
} from '../services/control-center-stats.js';
import { reconcileSessionFeed } from '../services/control-center-feed.js';
import { initUsageLimits } from '../services/control-center-limits.js';

export const initSSE = () => {
  const indicator = document.getElementById('cc-sse-indicator');

  createSSEClient('/control-center/api/sse', {
    onConnect: () => indicator?.classList.add('live-dot-connected'),
    onDisconnect: () => indicator?.classList.remove('live-dot-connected'),
    events: {
      'today-stats': (s) => {
        setStatValue('active_now', s.active_now);
        setStatValue('completed', s.completed);
        setStatValue('sessions', s.sessions);
        if (s.has_success_rate) setStatValue('success_rate', s.success_rate + '%');
        setStatValue('errors', s.errors);
        if (s.apm_metrics) updateApmMetrics(s.apm_metrics);
        if (s.gamification?.has_gamification) {
          const g = s.gamification;
          const banner = document.querySelector('.cc-profile-banner');
          if (banner) {
            const xpLabel = banner.querySelector('.cc-xp-label');
            if (xpLabel) xpLabel.textContent = g.total_xp + ' XP \u00B7 ' + g.xp_to_next_rank + ' to ' + g.next_rank_name;
            const xpFill = banner.querySelector('.cc-xp-fill');
            if (xpFill) xpFill.style.setProperty('--sp-xp-pct', g.xp_progress_pct + '%');
            const rankName = banner.querySelector('.cc-rank-name');
            if (rankName) rankName.textContent = g.rank_name;
            const streakVal = banner.querySelector('.cc-profile-stat:first-child .cc-profile-stat-value');
            if (streakVal) streakVal.textContent = g.current_streak;
            const achVal = banner.querySelector('.cc-profile-stat:last-child .cc-profile-stat-value');
            if (achVal) achVal.textContent = g.achievements_count + '/' + g.achievements_total;
          }
        }
      },
      activity: (d) => {
        if (d.session_groups) reconcileSessionFeed(d.session_groups);
      },
      'usage-limits': initUsageLimits(),
      analytics: (d) => {
        if (d.skill_effectiveness) updateSkillCards(d.skill_effectiveness);
        if (d.session_ratings) updateSessionRatings(d.session_ratings);
        if (d.health) updateHealthScore(d.health);
        if (d.skill_adoption) updateSkillAdoption(d.skill_adoption);
        if (d.today_summary) updateDailySummary(d.today_summary);
        if (d.achievement_progress) updateAchievementProgress(d.achievement_progress);
        if (d.hourly_breakdown) updateHourlyCharts(d.hourly_breakdown);
        if (d.daily_7d) update7dData(d.daily_7d);
        if (d.daily_30d) update30dData(d.daily_30d);
        if (d.performance_summary) updatePerformanceSummary(d.performance_summary);
      },
    },
  });
};

export const loadInitialData = () => {
  const initialEl = document.getElementById('cc-initial-data');
  if (initialEl) {
    try {
      const initial = JSON.parse(initialEl.textContent);
      if (initial.hourly) updateHourlyCharts(initial.hourly);
      if (initial.perf) updatePerformanceSummary(initial.perf);
      if (initial.apm_metrics) updateApmMetrics(initial.apm_metrics);
    } catch (_parseErr) { /* non-critical: initial data may be missing or malformed */ }
  }
};
