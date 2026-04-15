const TYPING_SPEED = 35;
const TYPING_VARIANCE = 12;
const LINE_DELAY = 500;

export const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export const resetOutputLines = (terminal) => {
  for (const line of terminal.querySelectorAll('.terminal-output')) {
    line.classList.remove('visible');
  }
};

export const animateTerminal = async (terminal) => {
  const commandEl = terminal.querySelector('.command');
  const cursorEl = terminal.querySelector('.cursor');
  const commandText = commandEl?.dataset.text || '';

  if (commandEl) {
    commandEl.textContent = '';
    for (let i = 0; i < commandText.length; i++) {
      commandEl.textContent += commandText[i];
      const delay = TYPING_SPEED + (Math.random() * TYPING_VARIANCE - TYPING_VARIANCE / 2);
      await sleep(delay);
    }
  }

  await sleep(400);
  cursorEl?.classList.add('hidden');

  for (const line of terminal.querySelectorAll('.terminal-output')) {
    await sleep(LINE_DELAY);
    line.classList.add('visible');
  }

  await sleep(800);
};
