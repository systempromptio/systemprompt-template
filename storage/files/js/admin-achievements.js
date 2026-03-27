(function(app) {
    'use strict';

    const escapeHtml = app.escapeHtml;
    const CATEGORY_ICONS = {
        'First Steps': '\u26A1',
        'Milestones': '\uD83D\uDCCA',
        'Exploration': '\uD83D\uDD0D',
        'Creation': '\u2728',
        'Streaks': '\uD83D\uDD25',
        'Ranks': '\uD83C\uDFC6',
        'Special': '\u2B50'
    };
    function groupByCategory(items) {
        const groups = {};
        items.forEach(function(a) {
            const cat = a.category || 'Other';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(a);
        });
        return groups;
    }
    function renderAchievementCard(a) {
        const unlocked = a.total_unlocked > 0;
        const cls = unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
        const pct = unlocked ? 100 : (a.unlock_percentage || 0);
        const icon = CATEGORY_ICONS[a.category] || '\u2B50';
        const bar = '<div class="unlock-bar"><div class="unlock-bar-fill" style="width:' + pct + '%"></div></div>';
        return '<div class="' + cls + '">' +
            '<div class="achievement-icon">' + icon + '</div>' +
            '<div style="font-weight:600;font-size:var(--sp-text-sm);color:var(--sp-text-primary)">' + escapeHtml(a.name) + '</div>' +
            '<div style="font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)">' + escapeHtml(a.description) + '</div>' +
            '<div style="font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)">' + a.total_unlocked + ' unlocked</div>' +
            bar +
        '</div>';
    }
    function renderAchievementsContent(data) {
        const items = Array.isArray(data) ? data : (data.achievements || []);
        if (!items.length) {
            return '<div class="empty-state"><p>No achievements defined.</p></div>';
        }
        const groups = groupByCategory(items);
        const categories = Object.keys(groups);
        let html = '';
        categories.forEach(function(cat) {
            const cards = groups[cat].map(renderAchievementCard).join('');
            html += '<div style="margin-bottom:var(--sp-space-6)">' +
                '<div class="section-title">' + escapeHtml(cat) + '</div>' +
                '<div class="achievement-grid">' + cards + '</div>' +
            '</div>';
        });
        return html;
    }
    app.renderAchievements = function() {
        const root = document.getElementById('achievements-content');
        if (!root) return;
        root.innerHTML = '<div class="loading-center"><div class="loading-spinner"></div></div>';
        app.api('/gamification/achievements').then(function(data) {
            root.innerHTML = renderAchievementsContent(data);
        }).catch(function(err) {
            root.innerHTML = '<div class="empty-state"><p>Failed to load achievements.</p></div>';
            app.Toast.show(err.message || 'Failed to load achievements', 'error');
        });
    };
})(window.AdminApp);
