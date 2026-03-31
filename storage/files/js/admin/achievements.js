(function(app) {
    'use strict';

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
        items.forEach((a) => {
            const cat = a.category || 'Other';
            if (!groups[cat]) groups[cat] = [];
            groups[cat].push(a);
        });
        return groups;
    }
    function renderAchievementCard(a) {
        const unlocked = a.total_unlocked > 0;
        const card = document.createElement('div');
        card.className = unlocked ? 'achievement-card unlocked' : 'achievement-card locked';
        const pct = unlocked ? 100 : (a.unlock_percentage || 0);
        const icon = CATEGORY_ICONS[a.category] || '\u2B50';

        const iconDiv = document.createElement('div');
        iconDiv.className = 'achievement-icon';
        iconDiv.textContent = icon;

        const nameDiv = document.createElement('div');
        nameDiv.style.cssText = 'font-weight:600;font-size:var(--sp-text-sm);color:var(--sp-text-primary)';
        nameDiv.textContent = a.name;

        const descDiv = document.createElement('div');
        descDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)';
        descDiv.textContent = a.description;

        const countDiv = document.createElement('div');
        countDiv.style.cssText = 'font-size:var(--sp-text-xs);color:var(--sp-text-tertiary);margin-top:var(--sp-space-1)';
        countDiv.textContent = a.total_unlocked + ' unlocked';

        const bar = document.createElement('div');
        bar.className = 'unlock-bar';
        const barFill = document.createElement('div');
        barFill.className = 'unlock-bar-fill';
        barFill.style.width = pct + '%';
        bar.append(barFill);

        card.append(iconDiv, nameDiv, descDiv, countDiv, bar);
        return card;
    }
    function renderAchievementsContent(data, root) {
        const items = Array.isArray(data) ? data : (data.achievements || []);
        root.replaceChildren();
        if (!items.length) {
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = 'No achievements defined.';
            empty.append(p);
            root.append(empty);
            return;
        }
        const groups = groupByCategory(items);
        const categories = Object.keys(groups);
        categories.forEach((cat) => {
            const section = document.createElement('div');
            section.style.marginBottom = 'var(--sp-space-6)';

            const title = document.createElement('div');
            title.className = 'section-title';
            title.textContent = cat;

            const grid = document.createElement('div');
            grid.className = 'achievement-grid';
            groups[cat].forEach((a) => {
                grid.append(renderAchievementCard(a));
            });

            section.append(title, grid);
            root.append(section);
        });
    }
    app.renderAchievements = () => {
        const root = document.getElementById('achievements-content');
        if (!root) return;
        root.replaceChildren();
        const loadingDiv = document.createElement('div');
        loadingDiv.className = 'loading-center';
        const spinner = document.createElement('div');
        spinner.className = 'loading-spinner';
        loadingDiv.append(spinner);
        root.append(loadingDiv);
        app.api('/gamification/achievements').then((data) => {
            renderAchievementsContent(data, root);
        }).catch((err) => {
            root.replaceChildren();
            const empty = document.createElement('div');
            empty.className = 'empty-state';
            const p = document.createElement('p');
            p.textContent = 'Failed to load achievements.';
            empty.append(p);
            root.append(empty);
            app.Toast.show(err.message || 'Failed to load achievements', 'error');
        });
    };
})(window.AdminApp);
