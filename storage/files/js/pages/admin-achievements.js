const initAchievementsPage = () => {
  const xpFill = document.getElementById('xp-fill');
  if (xpFill) {
    const target = xpFill.dataset.target || '0';
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        xpFill.style.width = target + '%';
      });
    });
  }

  const ladder = document.getElementById('rank-ladder');
  const current = ladder?.querySelector('.rank-ladder__step--current');
  if (ladder && current) {
    const ladderRect = ladder.getBoundingClientRect();
    const currentRect = current.getBoundingClientRect();
    const scrollLeft = currentRect.left - ladderRect.left - (ladderRect.width / 2) + (currentRect.width / 2);
    ladder.scrollLeft = scrollLeft;
  }

  const navLeft = document.getElementById('rank-nav-left');
  const navRight = document.getElementById('rank-nav-right');

  if (navLeft && ladder) {
    navLeft.addEventListener('click', () => {
      ladder.scrollBy({ left: -200, behavior: 'smooth' });
    });
  }

  if (navRight && ladder) {
    navRight.addEventListener('click', () => {
      ladder.scrollBy({ left: 200, behavior: 'smooth' });
    });
  }

  const categories = document.querySelectorAll('.achievement-category');
  const anyOpen = Array.from(categories).some(d => d.hasAttribute('open'));
  if (!anyOpen && categories.length > 0) {
    categories[0].setAttribute('open', '');
  }
};

initAchievementsPage();
