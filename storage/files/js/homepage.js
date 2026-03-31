(function() {

  document.addEventListener('DOMContentLoaded', init);

  function init() {
    initCopyButtons();
    initStatusCard();
  }

  function initCopyButtons() {
    document.querySelectorAll('.copy-btn').forEach((btn) => {
      btn.addEventListener('click', async () => {
        const code = btn.getAttribute('data-code');
        await navigator.clipboard.writeText(code);
        btn.classList.add('copied');
        setTimeout(() => btn.classList.remove('copied'), 2000);
      });
    });
  }

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

  StatusCardManager.prototype.createStatusItem = function(indicatorClass, text) {
    const li = document.createElement('li');
    li.className = 'status-card__item';
    const indicator = document.createElement('span');
    indicator.className = 'status-card__item-indicator' + indicatorClass;
    const nameSpan = document.createElement('span');
    nameSpan.className = 'status-card__item-name';
    nameSpan.textContent = text;
    li.append(indicator, nameSpan);
    return li;
  };

  StatusCardManager.prototype.renderMcpServices = function(data) {
    if (!this.mcpList) return;

    const services = Array.isArray(data) ? data : (data.data || data.servers || data.services || []);

    if (services.length === 0) {
      this.mcpList.replaceChildren(
        this.createStatusItem(' status-card__item-indicator--warning', 'No services registered')
      );
      if (this.mcpCount) this.mcpCount.textContent = '0';
      return;
    }

    const frag = document.createDocumentFragment();
    const baseUrl = window.location.origin;
    for (let i = 0; i < services.length; i++) {
      const service = services[i];
      const statusClass = this.getStatusClass(service);
      const name = service.name || service.id || 'Unknown';
      const scopes = service.oauth_scopes || service.scopes || [];
      const scopeText = scopes.length ? ' [' + scopes.join(', ') + ']' : '';
      const endpoint = service.endpoint || '';
      const fullUrl = endpoint ? baseUrl + endpoint : '';

      const li = document.createElement('li');
      li.className = 'status-card__item';
      const ind = document.createElement('span');
      ind.className = 'status-card__item-indicator' + statusClass;
      const nameSpan = document.createElement('span');
      nameSpan.className = 'status-card__item-name';
      nameSpan.textContent = name;
      const scopeSpan = document.createElement('span');
      scopeSpan.className = 'status-card__item-scope';
      scopeSpan.textContent = scopeText;
      nameSpan.append(scopeSpan);
      li.append(ind, nameSpan);

      if (endpoint) {
        const btn = document.createElement('button');
        btn.className = 'status-card__connect-btn';
        btn.setAttribute('data-url', fullUrl);
        btn.setAttribute('data-name', name);
        btn.title = 'Copy MCP connection URL';
        btn.textContent = '[connect]';
        li.append(btn);
      }

      frag.append(li);
    }
    this.mcpList.replaceChildren(frag);
    this.initConnectButtons();

    if (this.mcpCount) {
      this.mcpCount.textContent = services.length.toString();
    }
  };

  StatusCardManager.prototype.renderAgents = function(data) {
    if (!this.agentsList) return;

    const agents = Array.isArray(data) ? data : (data.data || data.agents || []);

    if (agents.length === 0) {
      this.agentsList.replaceChildren(
        this.createStatusItem(' status-card__item-indicator--warning', 'No agents active')
      );
      if (this.agentsCount) this.agentsCount.textContent = '0';
      return;
    }

    const frag = document.createDocumentFragment();
    for (let i = 0; i < agents.length; i++) {
      const agent = agents[i];
      const statusClass = this.getStatusClass(agent);
      const name = agent.name || agent.id || 'Unknown';
      const scopes = agent.oauth_scopes || agent.scopes || [];
      if (!scopes.length && agent.security && agent.security[0] && agent.security[0].oauth2) {
        scopes = agent.security[0].oauth2;
      }
      const scopeText = scopes.length ? ' [' + scopes.join(', ') + ']' : '';

      const li = document.createElement('li');
      li.className = 'status-card__item';
      const ind = document.createElement('span');
      ind.className = 'status-card__item-indicator' + statusClass;
      const nameSpan = document.createElement('span');
      nameSpan.className = 'status-card__item-name';
      nameSpan.textContent = name;
      const scopeSpan = document.createElement('span');
      scopeSpan.className = 'status-card__item-scope';
      scopeSpan.textContent = scopeText;
      nameSpan.append(scopeSpan);
      li.append(ind, nameSpan);
      frag.append(li);
    }
    this.agentsList.replaceChildren(frag);

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
    return '';
  };

  StatusCardManager.prototype.handleFetchError = function() {
    this.retryCount++;

    if (this.retryCount >= STATUS_CONFIG.maxRetries) {
      this.setOfflineState();

      if (this.mcpList) {
        this.mcpList.replaceChildren(
          this.createStatusItem(' status-card__item-indicator--error', 'Connection failed')
        );
      }

      if (this.agentsList) {
        this.agentsList.replaceChildren(
          this.createStatusItem(' status-card__item-indicator--error', 'Connection failed')
        );
      }
    }
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
    const existing = document.getElementById('mcp-connect-modal');
    if (existing) existing.remove();

    const modal = document.createElement('div');
    modal.id = 'mcp-connect-modal';
    modal.className = 'mcp-modal';

    const backdrop = document.createElement('div');
    backdrop.className = 'mcp-modal__backdrop';

    const content = document.createElement('div');
    content.className = 'mcp-modal__content';

    const closeBtn = document.createElement('button');
    closeBtn.className = 'mcp-modal__close';
    closeBtn.setAttribute('aria-label', 'Close');
    closeBtn.textContent = '\u00d7';

    const title = document.createElement('h3');
    title.className = 'mcp-modal__title';
    title.textContent = 'Connect to ' + name;

    const desc = document.createElement('p');
    desc.className = 'mcp-modal__desc';
    desc.textContent = 'Add this MCP server to your AI coding assistant using HTTP transport.';

    const urlRow = document.createElement('div');
    urlRow.className = 'mcp-modal__url-row';
    const urlCode = document.createElement('code');
    urlCode.className = 'mcp-modal__url';
    urlCode.textContent = url;
    const copyBtn = document.createElement('button');
    copyBtn.className = 'mcp-modal__copy';
    copyBtn.setAttribute('data-url', url);
    copyBtn.textContent = 'Copy';
    urlRow.append(urlCode, copyBtn);

    const instructions = document.createElement('div');
    instructions.className = 'mcp-modal__instructions';

    const h4Claude = document.createElement('h4');
    h4Claude.textContent = 'Claude Code';
    const preClaude = document.createElement('pre');
    const codeClaude = document.createElement('code');
    codeClaude.textContent = 'claude mcp add ' + name + ' ' + url + ' --transport http';
    preClaude.append(codeClaude);

    const h4Cursor = document.createElement('h4');
    h4Cursor.textContent = 'Cursor / VS Code';
    const pCursor = document.createElement('p');
    pCursor.textContent = 'Add to your ';
    const codeCursor1 = document.createElement('code');
    codeCursor1.textContent = '.cursor/mcp.json';
    const codeCursor2 = document.createElement('code');
    codeCursor2.textContent = '.vscode/mcp.json';
    pCursor.append(codeCursor1);
    pCursor.append(document.createTextNode(' or '));
    pCursor.append(codeCursor2);
    pCursor.append(document.createTextNode(':'));
    const preCursorJson = document.createElement('pre');
    const codeCursorJson = document.createElement('code');
    codeCursorJson.textContent = '{\n  "mcpServers": {\n    "' + name + '": {\n      "url": "' + url + '"\n    }\n  }\n}';
    preCursorJson.append(codeCursorJson);

    instructions.append(h4Claude, preClaude, h4Cursor, pCursor, preCursorJson);
    content.append(closeBtn, title, desc, urlRow, instructions);
    modal.append(backdrop, content);

    document.body.append(modal);
    const escHandler = (e) => {
      if (e.key === 'Escape') closeModal();
    };
    const closeModal = () => {
      modal.remove();
      document.removeEventListener('keydown', escHandler);
    };
    closeBtn.addEventListener('click', closeModal);
    backdrop.addEventListener('click', closeModal);

    copyBtn.addEventListener('click', () => {
      navigator.clipboard.writeText(url).then(() => {
        copyBtn.textContent = 'Copied!';
        setTimeout(() => { copyBtn.textContent = 'Copy'; }, 2000);
      });
    });

    document.addEventListener('keydown', escHandler);
  };
})();
