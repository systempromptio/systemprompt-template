const initLeaderboard = () => {
  const selfRow = document.querySelector('.leaderboard-table__self');
  if (selfRow) {
    selfRow.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
};

initLeaderboard();
