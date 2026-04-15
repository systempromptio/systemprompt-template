(function() {
  'use strict';

  const TYPING_SPEED = 35;
  const TYPING_VARIANCE = 12;
  const LINE_DELAY = 500;

  class TerminalStack {
    constructor() {
      this.stack = document.getElementById('terminal-stack');
      this.prevBtn = document.getElementById('terminal-prev');
      this.nextBtn = document.getElementById('terminal-next');
      this.replayBtn = document.getElementById('terminal-replay');
      this.indicator = document.getElementById('terminal-indicator');

      if (!this.stack) return;

      this.terminals = Array.from(this.stack.querySelectorAll('.terminal'));
      this.currentIndex = 0;
      this.isAnimating = false;
      this.hasStarted = false;

      this.init();
    }

    init() {
      this.resetAllTerminals();
      this.updateControls();
      this.setupEventListeners();
      this.setupIntersectionObserver();
    }

    setupEventListeners() {
      if (this.prevBtn) {
        this.prevBtn.addEventListener('click', () => this.prev());
      }
      if (this.nextBtn) {
        this.nextBtn.addEventListener('click', () => this.next());
      }
      if (this.replayBtn) {
        this.replayBtn.addEventListener('click', () => this.replay());
      }
    }

    setupIntersectionObserver() {
      const options = {
        root: null,
        rootMargin: '0px',
        threshold: 0.5
      };

      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting && !this.hasStarted) {
            this.start();
          }
        });
      }, options);

      observer.observe(this.stack);
    }

    resetAllTerminals() {
      this.terminals.forEach(terminal => {
        terminal.classList.remove('active', 'completed');
        const command = terminal.querySelector('.command');
        const cursor = terminal.querySelector('.cursor');
        if (command) command.textContent = '';
        if (cursor) cursor.classList.remove('hidden');
        terminal.querySelectorAll('.terminal-output').forEach(line => {
          line.classList.remove('visible');
        });
      });
    }

    resetTerminal(terminal) {
      const command = terminal.querySelector('.command');
      const cursor = terminal.querySelector('.cursor');
      if (command) command.textContent = '';
      if (cursor) cursor.classList.remove('hidden');
      terminal.querySelectorAll('.terminal-output').forEach(line => {
        line.classList.remove('visible');
      });
    }

    updateControls() {
      if (this.indicator) {
        this.indicator.textContent = `${this.currentIndex + 1} / ${this.terminals.length}`;
      }
    }

    async start() {
      if (this.hasStarted) return;
      this.hasStarted = true;
      this.currentIndex = 0;
      await this.showTerminal(0);
    }

    async showTerminal(index) {
      if (this.isAnimating) return;
      if (index < 0 || index >= this.terminals.length) return;

      this.isAnimating = true;
      this.currentIndex = index;
      this.updateControls();

      this.terminals.forEach((terminal, i) => {
        terminal.classList.remove('active', 'completed');
        if (i < index) {
          terminal.classList.add('completed');
        } else if (i === index) {
          terminal.classList.add('active');
        }
      });

      const terminal = this.terminals[index];
      this.resetTerminal(terminal);

      await this.sleep(300);
      await this.animateTerminal(terminal);

      this.isAnimating = false;
      this.updateControls();
    }

    async animateTerminal(terminal) {
      const commandEl = terminal.querySelector('.command');
      const cursorEl = terminal.querySelector('.cursor');
      const outputLines = terminal.querySelectorAll('.terminal-output');
      const commandText = commandEl?.dataset.text || '';

      if (commandEl) {
        commandEl.textContent = '';
        for (let i = 0; i < commandText.length; i++) {
          commandEl.textContent += commandText[i];
          const delay = TYPING_SPEED + (Math.random() * TYPING_VARIANCE - TYPING_VARIANCE / 2);
          await this.sleep(delay);
        }
      }

      await this.sleep(400);
      if (cursorEl) {
        cursorEl.classList.add('hidden');
      }

      for (const line of outputLines) {
        await this.sleep(LINE_DELAY);
        line.classList.add('visible');
      }

      await this.sleep(800);
    }

    async prev() {
      const newIndex = this.currentIndex > 0
        ? this.currentIndex - 1
        : this.terminals.length - 1;
      await this.jumpToTerminal(newIndex);
    }

    async next() {
      const newIndex = this.currentIndex < this.terminals.length - 1
        ? this.currentIndex + 1
        : 0;
      await this.jumpToTerminal(newIndex);
    }

    async jumpToTerminal(index) {
      this.isAnimating = false;

      this.currentIndex = index;
      this.updateControls();

      this.terminals.forEach((terminal, i) => {
        terminal.classList.remove('active', 'completed');
        const command = terminal.querySelector('.command');
        const cursor = terminal.querySelector('.cursor');

        if (i < index) {
          terminal.classList.add('completed');
          if (command) command.textContent = command.dataset.text || '';
          if (cursor) cursor.classList.add('hidden');
          terminal.querySelectorAll('.terminal-output').forEach(line => {
            line.classList.add('visible');
          });
        } else if (i === index) {
          terminal.classList.add('active');
          if (command) command.textContent = '';
          if (cursor) cursor.classList.remove('hidden');
          terminal.querySelectorAll('.terminal-output').forEach(line => {
            line.classList.remove('visible');
          });
        } else {
          if (command) command.textContent = '';
          if (cursor) cursor.classList.remove('hidden');
          terminal.querySelectorAll('.terminal-output').forEach(line => {
            line.classList.remove('visible');
          });
        }
      });

      await this.sleep(100);
      await this.animateTerminal(this.terminals[index]);
    }

    async replay() {
      this.resetAllTerminals();
      this.currentIndex = 0;
      this.hasStarted = false;
      this.updateControls();

      await this.sleep(300);
      await this.start();
    }

    sleep(ms) {
      return new Promise(resolve => setTimeout(resolve, ms));
    }
  }

  class MiniTerminal {
    constructor() {
      this.terminal = document.getElementById('hero-terminal');
      if (!this.terminal) return;

      this.commandEl = this.terminal.querySelector('.command');
      this.cursorEl = this.terminal.querySelector('.cursor');
      this.outputLines = this.terminal.querySelectorAll('.mini-terminal-output');
      this.commandText = this.commandEl?.dataset.text || '';
      this.TYPING_SPEED = 30;
      this.LOOP_DELAY = 3000;

      this.setupIntersectionObserver();
    }

    setupIntersectionObserver() {
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            this.startLoop();
            observer.disconnect();
          }
        });
      }, { threshold: 0.3 });
      observer.observe(this.terminal);
    }

    async startLoop() {
      while (true) {
        await this.animate();
        await this.sleep(this.LOOP_DELAY);
        this.reset();
        await this.sleep(500);
      }
    }

    reset() {
      if (this.commandEl) this.commandEl.textContent = '';
      if (this.cursorEl) this.cursorEl.classList.remove('hidden');
      this.outputLines.forEach(line => line.classList.remove('visible'));
    }

    async animate() {
      if (this.commandEl) {
        for (let i = 0; i < this.commandText.length; i++) {
          this.commandEl.textContent += this.commandText[i];
          await this.sleep(this.TYPING_SPEED + Math.random() * 10);
        }
      }
      await this.sleep(300);
      if (this.cursorEl) this.cursorEl.classList.add('hidden');
      for (const line of this.outputLines) {
        await this.sleep(400);
        line.classList.add('visible');
      }
    }

    sleep(ms) {
      return new Promise(resolve => setTimeout(resolve, ms));
    }
  }

  document.addEventListener('DOMContentLoaded', () => {
    new TerminalStack();
    new MiniTerminal();
  });
})();
