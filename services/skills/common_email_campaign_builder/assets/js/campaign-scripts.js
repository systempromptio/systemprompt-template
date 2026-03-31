/**
 * ENTERPRISE DEMO EMAIL CAMPAIGN BUILDER - SCRIPTS
 * Theme toggle, form validation, and animations
 */

(function() {
  'use strict';

  // ═══════════════════════════════════════════════════════════════════════════
  // THEME MANAGEMENT
  // ═══════════════════════════════════════════════════════════════════════════
  
  const ThemeManager = {
    STORAGE_KEY: 'enterprise-demo-theme',
    DARK: 'dark',
    LIGHT: 'light',
    
    init() {
      this.applyStoredTheme();
      this.setupToggle();
    },
    
    getStoredTheme() {
      return localStorage.getItem(this.STORAGE_KEY);
    },
    
    setTheme(theme) {
      document.documentElement.setAttribute('data-theme', theme);
      localStorage.setItem(this.STORAGE_KEY, theme);
      this.updateToggleIcon(theme);
    },
    
    applyStoredTheme() {
      const stored = this.getStoredTheme();
      if (stored) {
        this.setTheme(stored);
      } else {
        // Check system preference
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        this.setTheme(prefersDark ? this.DARK : this.LIGHT);
      }
    },
    
    toggle() {
      const current = document.documentElement.getAttribute('data-theme') || this.LIGHT;
      const next = current === this.DARK ? this.LIGHT : this.DARK;
      this.setTheme(next);
    },
    
    setupToggle() {
      const toggleBtn = document.querySelector('.theme-toggle');
      if (toggleBtn) {
        toggleBtn.addEventListener('click', () => this.toggle());
        this.updateToggleIcon(this.getStoredTheme() || this.LIGHT);
      }
    },
    
    updateToggleIcon(theme) {
      const toggleBtn = document.querySelector('.theme-toggle');
      if (!toggleBtn) return;
      
      const sunIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="5"/>
        <line x1="12" y1="1" x2="12" y2="3"/>
        <line x1="12" y1="21" x2="12" y2="23"/>
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
        <line x1="1" y1="12" x2="3" y2="12"/>
        <line x1="21" y1="12" x2="23" y2="12"/>
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
      </svg>`;
      
      const moonIcon = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
      </svg>`;
      
      toggleBtn.innerHTML = theme === this.DARK ? sunIcon : moonIcon;
      toggleBtn.style.color = theme === this.DARK ? '#E5B92B' : '#6B4FBB';
    }
  };

  // ═══════════════════════════════════════════════════════════════════════════
  // FORM VALIDATION
  // ═══════════════════════════════════════════════════════════════════════════
  
  const FormValidator = {
    init() {
      this.setupForms();
    },
    
    setupForms() {
      const forms = document.querySelectorAll('[data-validate]');
      forms.forEach(form => {
        form.addEventListener('submit', (e) => this.handleSubmit(e, form));
        
        // Real-time validation
        const inputs = form.querySelectorAll('input, textarea');
        inputs.forEach(input => {
          input.addEventListener('blur', () => this.validateField(input));
          input.addEventListener('input', () => this.clearError(input));
        });
      });
    },
    
    handleSubmit(e, form) {
      e.preventDefault();
      
      let isValid = true;
      const inputs = form.querySelectorAll('input[required], textarea[required]');
      
      inputs.forEach(input => {
        if (!this.validateField(input)) {
          isValid = false;
        }
      });
      
      if (isValid) {
        this.submitForm(form);
      }
    },
    
    validateField(input) {
      const value = input.value.trim();
      const type = input.type;
      let isValid = true;
      let errorMessage = '';
      
      // Required check
      if (input.required && !value) {
        isValid = false;
        errorMessage = 'Este campo es obligatorio';
      }
      // Email validation
      else if (type === 'email' && value) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(value)) {
          isValid = false;
          errorMessage = 'Introduce un email valido';
        }
      }
      // Phone validation
      else if (type === 'tel' && value) {
        const phoneRegex = /^[+]?[\d\s-]{9,}$/;
        if (!phoneRegex.test(value)) {
          isValid = false;
          errorMessage = 'Introduce un telefono valido';
        }
      }
      
      if (!isValid) {
        this.showError(input, errorMessage);
      } else {
        this.clearError(input);
      }
      
      return isValid;
    },
    
    showError(input, message) {
      this.clearError(input);
      input.classList.add('is-invalid');
      
      const errorDiv = document.createElement('div');
      errorDiv.className = 'invalid-feedback';
      errorDiv.textContent = message;
      errorDiv.style.cssText = 'display: block; color: #dc3545; font-size: 0.875rem; margin-top: 0.25rem;';
      
      input.parentNode.appendChild(errorDiv);
    },
    
    clearError(input) {
      input.classList.remove('is-invalid');
      const existingError = input.parentNode.querySelector('.invalid-feedback');
      if (existingError) {
        existingError.remove();
      }
    },
    
    submitForm(form) {
      const submitBtn = form.querySelector('[type="submit"]');
      const originalText = submitBtn.textContent;
      
      // Show loading state
      submitBtn.disabled = true;
      submitBtn.textContent = 'Enviando...';
      
      // Collect form data
      const formData = new FormData(form);
      const data = Object.fromEntries(formData.entries());
      
      // Simulate API call (replace with actual endpoint)
      setTimeout(() => {
        console.log('Form data:', data);
        
        // Show success message
        this.showSuccess(form);
        
        // Reset form
        form.reset();
        submitBtn.disabled = false;
        submitBtn.textContent = originalText;
      }, 1000);
    },
    
    showSuccess(form) {
      const successDiv = document.createElement('div');
      successDiv.className = 'alert-success';
      successDiv.style.cssText = `
        background: rgba(229, 185, 43, 0.1);
        border: 1px solid #E5B92B;
        color: #1A0030;
        padding: 1rem 1.5rem;
        border-radius: 0.75rem;
        margin-top: 1rem;
        font-weight: 500;
      `;
      successDiv.textContent = 'Gracias por tu interes. Te contactaremos pronto.';
      
      form.appendChild(successDiv);
      
      setTimeout(() => {
        successDiv.remove();
      }, 5000);
    }
  };

  // ═══════════════════════════════════════════════════════════════════════════
  // SCROLL ANIMATIONS
  // ═══════════════════════════════════════════════════════════════════════════
  
  const ScrollAnimator = {
    init() {
      this.setupObserver();
    },
    
    setupObserver() {
      const options = {
        root: null,
        rootMargin: '0px',
        threshold: 0.1
      };
      
      const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
          if (entry.isIntersecting) {
            entry.target.classList.add('is-visible');
            observer.unobserve(entry.target);
          }
        });
      }, options);
      
      // Observe elements with animation classes
      const animatedElements = document.querySelectorAll('[data-animate]');
      animatedElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(20px)';
        el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
        observer.observe(el);
      });
      
      // Add visible styles
      const style = document.createElement('style');
      style.textContent = `
        [data-animate].is-visible {
          opacity: 1 !important;
          transform: translateY(0) !important;
        }
      `;
      document.head.appendChild(style);
    }
  };

  // ═══════════════════════════════════════════════════════════════════════════
  // SMOOTH SCROLL
  // ═══════════════════════════════════════════════════════════════════════════
  
  const SmoothScroll = {
    init() {
      document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', (e) => {
          e.preventDefault();
          const targetId = anchor.getAttribute('href');
          if (targetId === '#') return;
          
          const target = document.querySelector(targetId);
          if (target) {
            target.scrollIntoView({
              behavior: 'smooth',
              block: 'start'
            });
          }
        });
      });
    }
  };

  // ═══════════════════════════════════════════════════════════════════════════
  // INITIALIZATION
  // ═══════════════════════════════════════════════════════════════════════════
  
  document.addEventListener('DOMContentLoaded', () => {
    ThemeManager.init();
    FormValidator.init();
    ScrollAnimator.init();
    SmoothScroll.init();
  });

  // Expose for external use if needed
  window.Enterprise DemoCampaign = {
    ThemeManager,
    FormValidator
  };

})();
