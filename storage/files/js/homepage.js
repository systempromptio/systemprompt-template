(function() {

  document.addEventListener('DOMContentLoaded', init);

  function init() {
    initCopyButtons();
    initStatusCard();
  }

  function initCopyButtons() {
    document.querySelectorAll('.copy-btn').forEach((btn) => {
      btn.addEventListener('click', async function() {
        const code = this.getAttribute('data-code');
        await navigator.clipboard.writeText(code);
        this.classList.add('copied');
        setTimeout(() => this.classList.remove('copied'), 2000);
      });
    });
  }

  // ============================================
  // STATUS CARD - Live System Status
  // ============================================

  const STATUS_CONFIG = {
    refreshInterval: 30000,
    retryDelay: 5000,
    maxRetries: 3
  };

  function initStatusCard() {
    const statusCard = document.getElementById('system-status-card');
    if (!statusCard) return;

    const mcpEndpoint = statusCard.getAttribute('data-mcp-endpoint');
    const agentsEndpoint = statusCard.getAttribute('data-agents-endpoint');

    if (!mcpEndpoint || !agentsEndpoint) return;

    const manager = new StatusCardManager(statusCard, mcpEndpoint, agentsEndpoint);
    manager.start();
  }

  function StatusCardManager(container, mcpEndpoint, agentsEndpoint) {
    this.container = container;
    this.mcpEndpoint = mcpEndpoint;
    this.agentsEndpoint = agentsEndpoint;
    this.indicator = container.querySelector('#status-indicator');
    this.mcpList = container.querySelector('#mcp-services-list');
    this.mcpCount = container.querySelector('#mcp-count');
    this.agentsList = container.querySelector('#agents-list');
    this.agentsCount = container.querySelector('#agents-count');
    this.retryCount = 0;
    this.intervalId = null;
  }

  StatusCardManager.prototype.start = function() {
    const self = this;
    this.setLoadingState();
    this.fetchAllStatus();

    this.intervalId = setInterval(() => {
      self.fetchAllStatus();
    }, STATUS_CONFIG.refreshInterval);

    window.addEventListener('beforeunload', () => {
      if (self.intervalId) clearInterval(self.intervalId);
    });

    document.addEventListener('visibilitychange', () => {
      if (document.hidden && self.intervalId) {
        clearInterval(self.intervalId);
        self.intervalId = null;
      } else if (!document.hidden && !self.intervalId) {
        self.fetchAllStatus();
        self.intervalId = setInterval(() => {
          self.fetchAllStatus();
        }, STATUS_CONFIG.refreshInterval);
      }
    });
  };

  StatusCardManager.prototype.setLoadingState = function() {
    if (this.indicator) {
      this.indicator.className = 'status-card__indicator status-card__indicator--loading';
    }
  };

  StatusCardManager.prototype.setOnlineState = function() {
    if (this.indicator) {
      this.indicator.className = 'status-card__indicator';
    }
  };

  StatusCardManager.prototype.setOfflineState = function() {
    if (this.indicator) {
      this.indicator.className = 'status-card__indicator status-card__indicator--offline';
    }
  };

  StatusCardManager.prototype.fetchAllStatus = function() {
    const self = this;

    Promise.all([
      this.fetchWithRetry(this.mcpEndpoint),
      this.fetchWithRetry(this.agentsEndpoint)
    ]).then((results) => {
      self.renderMcpServices(results[0]);
      self.renderAgents(results[1]);
      self.setOnlineState();
      self.retryCount = 0;
    }).catch(() => {
      self.handleFetchError();
    });
  };

  StatusCardManager.prototype.fetchWithRetry = function(endpoint, retries) {
    retries = retries || STATUS_CONFIG.maxRetries;

    return new Promise((resolve, reject) => {
      let attempt = 0;

      function tryFetch() {
        fetch(endpoint, {
          method: 'GET',
          headers: {
            'Accept': 'application/json'
          }
        }).then((response) => {
          if (!response.ok) {
            throw new Error('HTTP ' + response.status);
          }
          return response.json();
        }).then((data) => {
          resolve(data);
        }).catch((error) => {
          attempt++;
          if (attempt >= retries) {
            reject(error);
          } else {
            setTimeout(tryFetch, STATUS_CONFIG.retryDelay);
          }
        });
      }

      tryFetch();
    });
  };

  StatusCardManager.prototype.renderMcpServices = function(data) {
    if (!this.mcpList) return;

    const services = Array.isArray(data) ? data : (data.data || data.servers || data.services || []);

    if (services.length === 0) {
      this.mcpList.innerHTML =
        '<li class="status-card__item">' +
          '<span class="status-card__item-indicator status-card__item-indicator--warning"></span>' +
          '<span class="status-card__item-name">No services registered</span>' +
        '</li>';
      if (this.mcpCount) this.mcpCount.textContent = '0';
      return;
    }

    let html = '';
    const baseUrl = window.location.origin;
    for (let i = 0; i < services.length; i++) {
      const service = services[i];
      const statusClass = this.getStatusClass(service);
      const name = this.escapeHtml(service.name || service.id || 'Unknown');
      const scopes = service.oauth_scopes || service.scopes || [];
      const scopeText = scopes.length ? ' [' + scopes.join(', ') + ']' : '';
      const endpoint = service.endpoint || '';
      const fullUrl = endpoint ? baseUrl + endpoint : '';
      const connectBtn = endpoint ? '<button class="status-card__connect-btn" data-url="' + this.escapeHtml(fullUrl) + '" data-name="' + this.escapeHtml(name) + '" title="Copy MCP connection URL">[connect]</button>' : '';
      html +=
        '<li class="status-card__item">' +
          '<span class="status-card__item-indicator' + statusClass + '"></span>' +
          '<span class="status-card__item-name">' + name + '<span class="status-card__item-scope">' + scopeText + '</span></span>' +
          connectBtn +
        '</li>';
    }
    this.mcpList.innerHTML = html;
    this.initConnectButtons();

    if (this.mcpCount) {
      this.mcpCount.textContent = services.length.toString();
    }
  };

  StatusCardManager.prototype.renderAgents = function(data) {
    if (!this.agentsList) return;

    const agents = Array.isArray(data) ? data : (data.data || data.agents || []);

    if (agents.length === 0) {
      this.agentsList.innerHTML =
        '<li class="status-card__item">' +
          '<span class="status-card__item-indicator status-card__item-indicator--warning"></span>' +
          '<span class="status-card__item-name">No agents active</span>' +
        '</li>';
      if (this.agentsCount) this.agentsCount.textContent = '0';
      return;
    }

    let html = '';
    for (let i = 0; i < agents.length; i++) {
      const agent = agents[i];
      const statusClass = this.getStatusClass(agent);
      const name = this.escapeHtml(agent.name || agent.id || 'Unknown');
      let scopes = agent.oauth_scopes || agent.scopes || [];
      if (!scopes.length && agent.security && agent.security[0] && agent.security[0].oauth2) {
        scopes = agent.security[0].oauth2;
      }
      const scopeText = scopes.length ? ' [' + scopes.join(', ') + ']' : '';
      html +=
        '<li class="status-card__item">' +
          '<span class="status-card__item-indicator' + statusClass + '"></span>' +
          '<span class="status-card__item-name">' + name + '<span class="status-card__item-scope">' + scopeText + '</span></span>' +
        '</li>';
    }
    this.agentsList.innerHTML = html;

    if (this.agentsCount) {
      this.agentsCount.textContent = agents.length.toString();
    }
  };

  StatusCardManager.prototype.getStatusClass = function(item) {
    const status = (item.status || item.state || 'running').toLowerCase();
    if (status === 'error' || status === 'failed') {
      return ' status-card__item-indicator--error';
    }
    if (status === 'warning' || status === 'degraded') {
      return ' status-card__item-indicator--warning';
    }
    return ''; // Default green
  };

  StatusCardManager.prototype.handleFetchError = function() {
    this.retryCount++;

    if (this.retryCount >= STATUS_CONFIG.maxRetries) {
      this.setOfflineState();

      if (this.mcpList) {
        this.mcpList.innerHTML =
          '<li class="status-card__item">' +
            '<span class="status-card__item-indicator status-card__item-indicator--error"></span>' +
            '<span class="status-card__item-name">Connection failed</span>' +
          '</li>';
      }

      if (this.agentsList) {
        this.agentsList.innerHTML =
          '<li class="status-card__item">' +
            '<span class="status-card__item-indicator status-card__item-indicator--error"></span>' +
            '<span class="status-card__item-name">Connection failed</span>' +
          '</li>';
      }
    }
  };

  StatusCardManager.prototype.escapeHtml = function(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  };

  StatusCardManager.prototype.initConnectButtons = function() {
    const self = this;
    const buttons = this.mcpList.querySelectorAll('.status-card__connect-btn');
    buttons.forEach((btn) => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        const url = btn.getAttribute('data-url');
        const name = btn.getAttribute('data-name');
        self.showConnectModal(url, name);
      });
    });
  };

  StatusCardManager.prototype.showConnectModal = function(url, name) {
    // Remove existing modal if any
    const existing = document.getElementById('mcp-connect-modal');
    if (existing) existing.remove();

    const modal = document.createElement('div');
    modal.id = 'mcp-connect-modal';
    modal.className = 'mcp-modal';
    modal.innerHTML =
      '<div class="mcp-modal__backdrop"></div>' +
      '<div class="mcp-modal__content">' +
        '<button class="mcp-modal__close" aria-label="Close">&times;</button>' +
        '<h3 class="mcp-modal__title">Connect to ' + this.escapeHtml(name) + '</h3>' +
        '<p class="mcp-modal__desc">Add this MCP server to your AI coding assistant using HTTP transport.</p>' +
        '<div class="mcp-modal__url-row">' +
          '<code class="mcp-modal__url">' + this.escapeHtml(url) + '</code>' +
          '<button class="mcp-modal__copy" data-url="' + this.escapeHtml(url) + '">Copy</button>' +
        '</div>' +
        '<div class="mcp-modal__instructions">' +
          '<h4>Claude Code</h4>' +
          '<pre><code>claude mcp add ' + this.escapeHtml(name) + ' ' + this.escapeHtml(url) + ' --transport http</code></pre>' +
          '<h4>Cursor / VS Code</h4>' +
          '<p>Add to your <code>.cursor/mcp.json</code> or <code>.vscode/mcp.json</code>:</p>' +
          '<pre><code>{\n  "mcpServers": {\n    "' + this.escapeHtml(name) + '": {\n      "url": "' + this.escapeHtml(url) + '"\n    }\n  }\n}</code></pre>' +
        '</div>' +
      '</div>';

    document.body.appendChild(modal);

    // Close handlers
    const closeBtn = modal.querySelector('.mcp-modal__close');
    const backdrop = modal.querySelector('.mcp-modal__backdrop');
    const escHandler = (e) => {
      if (e.key === 'Escape') closeModal();
    };
    const closeModal = () => {
      modal.remove();
      document.removeEventListener('keydown', escHandler);
    };
    closeBtn.addEventListener('click', closeModal);
    backdrop.addEventListener('click', closeModal);

    // Copy button
    const copyBtn = modal.querySelector('.mcp-modal__copy');
    copyBtn.addEventListener('click', () => {
      navigator.clipboard.writeText(url).then(() => {
        copyBtn.textContent = 'Copied!';
        setTimeout(() => { copyBtn.textContent = 'Copy'; }, 2000);
      });
    });

    // ESC to close
    document.addEventListener('keydown', escHandler);
  };
})();
