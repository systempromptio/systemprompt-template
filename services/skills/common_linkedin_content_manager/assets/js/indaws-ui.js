/**
 * FOODLES UI - LinkedIn Content Manager
 * Interactive functionality for the UI components
 */

(function() {
  'use strict';

  // ===========================================================================
  // THEME MANAGEMENT
  // ===========================================================================
  
  const ThemeManager = {
    STORAGE_KEY: 'foodles-theme',
    
    init() {
      const savedTheme = localStorage.getItem(this.STORAGE_KEY);
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      const theme = savedTheme || (prefersDark ? 'dark' : 'light');
      
      this.setTheme(theme, false);
      this.bindToggle();
      this.watchSystemPreference();
    },
    
    setTheme(theme, save = true) {
      document.documentElement.setAttribute('data-theme', theme);
      if (save) {
        localStorage.setItem(this.STORAGE_KEY, theme);
      }
      this.updateToggleState(theme);
    },
    
    toggle() {
      const current = document.documentElement.getAttribute('data-theme');
      const next = current === 'dark' ? 'light' : 'dark';
      this.setTheme(next);
    },
    
    bindToggle() {
      const toggles = document.querySelectorAll('.theme-toggle');
      toggles.forEach(toggle => {
        toggle.addEventListener('click', () => this.toggle());
      });
    },
    
    updateToggleState(theme) {
      const labels = document.querySelectorAll('.theme-toggle__label');
      labels.forEach(label => {
        label.textContent = theme === 'dark' ? 'Oscuro' : 'Claro';
      });
    },
    
    watchSystemPreference() {
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (!localStorage.getItem(this.STORAGE_KEY)) {
          this.setTheme(e.matches ? 'dark' : 'light', false);
        }
      });
    }
  };

  // ===========================================================================
  // CALENDAR COMPONENT
  // ===========================================================================
  
  const Calendar = {
    currentDate: new Date(),
    events: [],
    
    init(containerId, events = []) {
      this.container = document.getElementById(containerId);
      if (!this.container) return;
      
      this.events = events;
      this.render();
      this.bindNavigation();
    },
    
    render() {
      const year = this.currentDate.getFullYear();
      const month = this.currentDate.getMonth();
      
      const firstDay = new Date(year, month, 1);
      const lastDay = new Date(year, month + 1, 0);
      const startPadding = firstDay.getDay() === 0 ? 6 : firstDay.getDay() - 1;
      
      const monthNames = [
        'Enero', 'Febrero', 'Marzo', 'Abril', 'Mayo', 'Junio',
        'Julio', 'Agosto', 'Septiembre', 'Octubre', 'Noviembre', 'Diciembre'
      ];
      
      const dayNames = ['Lun', 'Mar', 'Mie', 'Jue', 'Vie', 'Sab', 'Dom'];
      
      let html = `
        <div class="calendar-nav d-flex justify-between align-center mb-md">
          <button class="btn-foodles btn-ghost btn-sm" data-action="prev">Anterior</button>
          <h3 class="mb-0">${monthNames[month]} ${year}</h3>
          <button class="btn-foodles btn-ghost btn-sm" data-action="next">Siguiente</button>
        </div>
        <div class="calendar-grid">
          ${dayNames.map(d => `<div class="calendar-header-cell">${d}</div>`).join('')}
      `;
      
      // Previous month padding
      const prevMonth = new Date(year, month, 0);
      for (let i = startPadding - 1; i >= 0; i--) {
        const day = prevMonth.getDate() - i;
        html += `<div class="calendar-cell calendar-cell--other-month">
          <div class="calendar-day">${day}</div>
        </div>`;
      }
      
      // Current month days
      const today = new Date();
      for (let day = 1; day <= lastDay.getDate(); day++) {
        const isToday = today.getDate() === day && 
                        today.getMonth() === month && 
                        today.getFullYear() === year;
        
        const dateStr = `${year}-${String(month + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`;
        const dayEvents = this.events.filter(e => e.date.startsWith(dateStr));
        
        html += `
          <div class="calendar-cell ${isToday ? 'calendar-cell--today' : ''}">
            <div class="calendar-day">${day}</div>
            ${dayEvents.map(e => `
              <div class="calendar-event calendar-event--${e.status}" 
                   title="${e.title}"
                   data-event-id="${e.id}">
                ${e.title}
              </div>
            `).join('')}
          </div>
        `;
      }
      
      // Next month padding
      const remainingCells = 42 - (startPadding + lastDay.getDate());
      for (let i = 1; i <= remainingCells && remainingCells < 7; i++) {
        html += `<div class="calendar-cell calendar-cell--other-month">
          <div class="calendar-day">${i}</div>
        </div>`;
      }
      
      html += '</div>';
      this.container.innerHTML = html;
    },
    
    bindNavigation() {
      this.container.addEventListener('click', (e) => {
        const action = e.target.dataset.action;
        if (action === 'prev') {
          this.currentDate.setMonth(this.currentDate.getMonth() - 1);
          this.render();
        } else if (action === 'next') {
          this.currentDate.setMonth(this.currentDate.getMonth() + 1);
          this.render();
        }
        
        const eventId = e.target.dataset.eventId;
        if (eventId) {
          this.onEventClick(eventId);
        }
      });
    },
    
    onEventClick(eventId) {
      const event = this.events.find(e => e.id === eventId);
      if (event && typeof this.eventClickHandler === 'function') {
        this.eventClickHandler(event);
      }
    },
    
    setEventClickHandler(handler) {
      this.eventClickHandler = handler;
    }
  };

  // ===========================================================================
  // STATS ANIMATION
  // ===========================================================================
  
  const StatsAnimator = {
    init() {
      const statValues = document.querySelectorAll('.stat-value[data-value]');
      
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            this.animateValue(entry.target);
            observer.unobserve(entry.target);
          }
        });
      }, { threshold: 0.5 });
      
      statValues.forEach(el => observer.observe(el));
    },
    
    animateValue(element) {
      const target = parseInt(element.dataset.value, 10);
      const duration = 1000;
      const start = performance.now();
      const suffix = element.dataset.suffix || '';
      
      const animate = (currentTime) => {
        const elapsed = currentTime - start;
        const progress = Math.min(elapsed / duration, 1);
        
        // Easing function
        const eased = 1 - Math.pow(1 - progress, 3);
        const current = Math.round(target * eased);
        
        element.textContent = current.toLocaleString('es-ES') + suffix;
        
        if (progress < 1) {
          requestAnimationFrame(animate);
        }
      };
      
      requestAnimationFrame(animate);
    }
  };

  // ===========================================================================
  // FORM VALIDATION
  // ===========================================================================
  
  const FormValidator = {
    init(formId) {
      const form = document.getElementById(formId);
      if (!form) return;
      
      form.addEventListener('submit', (e) => {
        if (!this.validate(form)) {
          e.preventDefault();
        }
      });
      
      // Real-time validation
      form.querySelectorAll('.form-control-glass').forEach(input => {
        input.addEventListener('blur', () => this.validateField(input));
      });
    },
    
    validate(form) {
      const inputs = form.querySelectorAll('[required]');
      let valid = true;
      
      inputs.forEach(input => {
        if (!this.validateField(input)) {
          valid = false;
        }
      });
      
      return valid;
    },
    
    validateField(input) {
      const value = input.value.trim();
      const isValid = value.length > 0;
      
      input.classList.toggle('is-invalid', !isValid);
      input.classList.toggle('is-valid', isValid);
      
      return isValid;
    }
  };

  // ===========================================================================
  // TOAST NOTIFICATIONS
  // ===========================================================================
  
  const Toast = {
    container: null,
    
    init() {
      this.container = document.createElement('div');
      this.container.className = 'toast-container';
      this.container.style.cssText = `
        position: fixed;
        bottom: 20px;
        right: 20px;
        z-index: 9999;
        display: flex;
        flex-direction: column;
        gap: 10px;
      `;
      document.body.appendChild(this.container);
    },
    
    show(message, type = 'info', duration = 3000) {
      const toast = document.createElement('div');
      toast.className = `glass-card animate-slide-in`;
      toast.style.cssText = `
        padding: 12px 20px;
        min-width: 250px;
        border-left: 4px solid ${this.getColor(type)};
      `;
      toast.textContent = message;
      
      this.container.appendChild(toast);
      
      setTimeout(() => {
        toast.style.opacity = '0';
        toast.style.transform = 'translateX(20px)';
        setTimeout(() => toast.remove(), 300);
      }, duration);
    },
    
    getColor(type) {
      const colors = {
        success: '#22c55e',
        error: '#ef4444',
        warning: '#E5B92B',
        info: '#6B4FBB'
      };
      return colors[type] || colors.info;
    }
  };

  // ===========================================================================
  // MODAL COMPONENT
  // ===========================================================================
  
  const Modal = {
    show(title, content, actions = []) {
      const overlay = document.createElement('div');
      overlay.className = 'modal-overlay';
      overlay.style.cssText = `
        position: fixed;
        inset: 0;
        background: rgba(26, 0, 48, 0.5);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 10000;
        animation: fadeIn 200ms ease;
      `;
      
      const modal = document.createElement('div');
      modal.className = 'glass-card';
      modal.style.cssText = `
        max-width: 500px;
        width: 90%;
        animation: fadeIn 300ms ease;
      `;
      
      modal.innerHTML = `
        <div class="card-header">
          <h3 class="card-title">${title}</h3>
          <button class="btn-foodles btn-ghost btn-sm modal-close" aria-label="Cerrar">X</button>
        </div>
        <div class="modal-content mb-lg">${content}</div>
        <div class="modal-actions d-flex justify-between gap-md">
          ${actions.map(a => `
            <button class="btn-foodles ${a.class || 'btn-secondary'}" data-action="${a.action}">
              ${a.label}
            </button>
          `).join('')}
        </div>
      `;
      
      overlay.appendChild(modal);
      document.body.appendChild(overlay);
      
      // Close handlers
      overlay.addEventListener('click', (e) => {
        if (e.target === overlay || e.target.classList.contains('modal-close')) {
          overlay.remove();
        }
        
        const action = e.target.dataset.action;
        if (action) {
          const actionDef = actions.find(a => a.action === action);
          if (actionDef && actionDef.handler) {
            actionDef.handler();
          }
          overlay.remove();
        }
      });
      
      // ESC to close
      const escHandler = (e) => {
        if (e.key === 'Escape') {
          overlay.remove();
          document.removeEventListener('keydown', escHandler);
        }
      };
      document.addEventListener('keydown', escHandler);
    }
  };

  // ===========================================================================
  // DATA TABLE
  // ===========================================================================
  
  const DataTable = {
    init(tableId, options = {}) {
      this.table = document.getElementById(tableId);
      if (!this.table) return;
      
      this.options = {
        sortable: true,
        searchable: true,
        ...options
      };
      
      if (this.options.sortable) {
        this.initSorting();
      }
    },
    
    initSorting() {
      const headers = this.table.querySelectorAll('thead th[data-sortable]');
      
      headers.forEach((th, index) => {
        th.style.cursor = 'pointer';
        th.addEventListener('click', () => this.sortByColumn(index));
      });
    },
    
    sortByColumn(columnIndex) {
      const tbody = this.table.querySelector('tbody');
      const rows = Array.from(tbody.querySelectorAll('tr'));
      const isAsc = this.table.dataset.sortDir !== 'asc';
      
      rows.sort((a, b) => {
        const aVal = a.cells[columnIndex].textContent.trim();
        const bVal = b.cells[columnIndex].textContent.trim();
        
        const aNum = parseFloat(aVal.replace(/[^\d.-]/g, ''));
        const bNum = parseFloat(bVal.replace(/[^\d.-]/g, ''));
        
        if (!isNaN(aNum) && !isNaN(bNum)) {
          return isAsc ? aNum - bNum : bNum - aNum;
        }
        
        return isAsc ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
      });
      
      this.table.dataset.sortDir = isAsc ? 'asc' : 'desc';
      rows.forEach(row => tbody.appendChild(row));
    }
  };

  // ===========================================================================
  // INITIALIZATION
  // ===========================================================================
  
  document.addEventListener('DOMContentLoaded', () => {
    ThemeManager.init();
    StatsAnimator.init();
    Toast.init();
  });

  // Expose to global scope
  window.FoodlesUI = {
    Theme: ThemeManager,
    Calendar,
    StatsAnimator,
    FormValidator,
    Toast,
    Modal,
    DataTable
  };

})();
