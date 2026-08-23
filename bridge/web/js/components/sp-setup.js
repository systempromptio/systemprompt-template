import { SpElement, reactive, escapeHtml } from "/assets/js/components/sp-element.js";
import { onBridgeEvent } from "/assets/js/events/bridge-events.js";
import { bridge } from "/assets/js/bridge.js";
import { fmtDuration } from "/assets/js/utils/format.js";
import { tr } from "/assets/js/utils/l10n.js";
import "/assets/js/components/sp-setup-gateway.js";
import "/assets/js/components/sp-setup-agents.js";
import "/assets/js/components/sp-setup-connection.js";

const STEPS = [
  { id: "connect", label: () => tr("astound-step-connect", "Sign in") },
  { id: "agents", label: () => tr("astound-step-agents", "Agents") },
  { id: "done", label: () => tr("astound-step-done", "Done") },
];

function isConfigured(snap) {
  const reachable = snap.gateway_status && snap.gateway_status.state === "reachable";
  const id = snap.verified_identity;
  return !!(reachable && id && id.user_id);
}

function gatewayHost(url) {
  try { return new URL(url).host; } catch (e) { return url || "your gateway"; }
}

export class SpSetup extends SpElement {
  constructor() {
    super();
    this.snapshot = null;
    this.step = "connect";
    this.anyInstalled = false;
    this.firstRunActive = false;
    this._finished = false;
    /** Latched once the app proper is on screen; see `_applySnapshot`. */
    this._leftSetup = false;
    /** Latched by Finish so the closing step survives the next snapshot, which
     *  would otherwise recompute `step` straight back to "agents". */
    this._doneShown = false;
    this._logoFragment = null;
    this._onSetupOpen = () => { document.body.classList.add("is-setup-mode"); };
    this.registerAction("finish", () => this._finish());
    this.registerAction("open-bridge", () => this._openBridge());
  }

  onConnect() {
    const tpl = this.querySelector('template[data-slot="logo"]');
    if (tpl) {
      this._logoFragment = tpl.content;
      tpl.remove();
    }
    bridge.stateSnapshot().then((s) => this._applySnapshot(s)).catch((e) => console.warn("snapshot failed", e));
    this.bridgeSubscribe("state.changed", (s) => this._applySnapshot(s));
    this._unsubOpen = onBridgeEvent("setup-open", this._onSetupOpen);
  }

  onDisconnect() {
    if (this._unsubOpen) { this._unsubOpen(); this._unsubOpen = null; }
  }

  _applySnapshot(snap) {
    this.snapshot = snap;
    if (!snap) { return; }
    const configured = isConfigured(snap);
    const hosts = snap.host_apps || [];
    // Install state for a host is only KNOWN once its probe has completed, at
    // which point `snapshot` is populated. Until every host has a snapshot the
    // result is "unknown" — we must not show onboarding then, or it flashes
    // before detection resolves (the bug where it appeared with agents already
    // installed). Once settled, show the agents step only when none are
    // installed; installing one (anyInstalled) drops straight into the app.
    const settled = hosts.length > 0 && hosts.every((h) => h.snapshot);
    const anyInstalled = hosts.some((h) => h.snapshot?.profile_state?.kind === "installed");
    this.anyInstalled = anyInstalled;

    // First-use provisioning pins the wizard open. Checked before the
    // settled/latched guards below: a run is exactly the window in which host
    // snapshots are still arriving, so those guards would return early and let
    // the app show over a half-installed machine. Ported from core, which
    // gained this after the overlay was last rebased.
    this.firstRunActive = !!(snap.first_run && snap.first_run.active);
    if (this.firstRunActive) {
      this.step = "agents";
      this._leftSetup = false;
      document.body.classList.add("is-setup-mode");
      return;
    }

    // Signing out is the one thing that legitimately sends us back to the
    // splash. Clear both latches so it can.
    if (!snap.verified_identity || !snap.verified_identity.user_id) {
      this._leftSetup = false;
      this._doneShown = false;
    }

    // The closing step is user-driven, not snapshot-derived; it stands until
    // the user leaves it.
    if (this._doneShown) {
      this.step = "done";
      return;
    }

    this.step = configured ? "agents" : "connect";

    // Everything below decides whether to show a full-screen overlay, so it must
    // only ever run on a settled snapshot. `configured` and `anyInstalled` each
    // start out false and flip true as the gateway probe and then the host
    // probes land — evaluating on those partial snapshots is what made the
    // window flick splash → app → splash → app during startup.
    const gatewayProbing = !snap.gateway_status || snap.gateway_status.state === "probing"
      || snap.gateway_status.state === "unknown";
    if (gatewayProbing || !settled) { return; }

    // One-way latch: once the app proper has been shown, a later probe result
    // must not yank the user back into onboarding mid-session.
    if (this._leftSetup) { return; }

    // `agents_onboarded` is now durable (core writes an onboarded.json sentinel
    // on setup.complete), so a user who finished setup stays finished — even if
    // they later uninstall the last profile, which used to re-open the wizard.
    const needAgents = !anyInstalled && !this._finished && !snap.agents_onboarded;
    const inSetup = !configured || needAgents;
    if (!inSetup) { this._leftSetup = true; }
    document.body.classList.toggle("is-setup-mode", inSetup);
  }

  /* Continue does not leave setup any more — it advances to a closing step that
   * reports what was actually accomplished. Leaving is `open-bridge`, an action
   * core registered but never gave a trigger. */
  _finish() {
    if (this.firstRunActive) { return; }
    this._finished = true;
    this._doneShown = true;
    this.step = "done";
    bridge.setupComplete().catch((err) => console.warn("setup complete", err));
    document.body.classList.add("is-setup-mode");
    this.invalidate();
  }

  _openBridge() {
    this._doneShown = false;
    this._leftSetup = true;
    document.body.classList.remove("is-setup-mode");
  }

  afterRender() {
    document.body.dataset.setupStep = this.step;
    const slot = this.querySelector("[data-logo-slot]");
    if (slot && this._logoFragment && !slot.firstElementChild) {
      slot.append(this._logoFragment.cloneNode(true));
    }
  }

  _renderSteps() {
    const active = STEPS.findIndex((s) => s.id === this.step);
    const items = STEPS.map((s, i) => {
      const state = i < active ? "is-done" : i === active ? "is-current" : "";
      const current = i === active ? 'aria-current="step"' : "";
      return `<li class="sp-setup__step-dot ${state}" ${current}><span>${escapeHtml(s.label())}</span></li>`;
    }).join("");
    return `<ol class="sp-setup__steps" aria-label="Setup progress">${items}</ol>`;
  }

  /** One row of the closing summary. */
  _summaryRow(ok, label, value) {
    return `<div class="sp-setup__done-row" data-ok="${ok ? "yes" : "no"}">
      <i aria-hidden="true">${ok ? "✓" : "!"}</i>
      <span>${escapeHtml(label)}</span>
      <b>${escapeHtml(value)}</b>
    </div>`;
  }

  _renderDone() {
    const snap = this.snapshot || {};
    const id = snap.verified_identity || {};
    const hosts = (snap.host_apps || []).filter((h) => h.enabled || !(snap.host_apps || []).some((x) => x.enabled));
    const ready = hosts.filter((h) => h.snapshot?.profile_state?.kind === "installed").length;
    const proxy = snap.local_proxy || {};
    const proxyOk = proxy.state === "Listening";
    const ttl = snap.cached_token ? snap.cached_token.ttl_seconds : null;

    let rows = "";
    rows += this._summaryRow(true, tr("astound-done-identity", "Signed in as"), id.email || id.user_id || "this device");
    rows += this._summaryRow(true, tr("astound-done-gateway", "Governed by"), gatewayHost(snap.gateway_url));
    rows += this._summaryRow(ready > 0, tr("astound-done-agents", "Agents ready"), `${ready} of ${hosts.length}`);
    if (snap.last_sync_summary) {
      rows += this._summaryRow(true, tr("astound-done-synced", "Synced"), snap.last_sync_summary);
    }
    rows += this._summaryRow(proxyOk, tr("astound-done-proxy", "Local proxy"),
      proxyOk ? (proxy.url || "listening") : "not listening yet");
    if (ttl != null) {
      rows += this._summaryRow(ttl >= 600, tr("astound-done-session", "Session"), `expires in ${fmtDuration(ttl)}`);
    }

    return `
      <h1>${escapeHtml(tr("astound-setup-done-heading", "You're set"))}</h1>
      <p class="sp-setup__lede">
        ${escapeHtml(tr("astound-setup-done-lede",
          "Astound Bridge runs from your menu bar from here. It keeps your agents pointed at the gateway and re-syncs your plugins and skills on its own."))}
      </p>
      <div class="sp-setup__done">${rows}</div>
      ${ttl != null && ttl < 600
        ? `<p class="sp-setup__hint">Your session renews automatically the next time an agent
             makes a request — there is nothing to do.</p>`
        : ""}
      <div class="sp-setup__actions">
        <button class="sp-btn-primary" type="button" data-action="open-bridge">
          <span class="sp-btn__label">${escapeHtml(tr("astound-setup-open", "Open Astound Bridge"))}</span>
        </button>
      </div>`;
  }

  render() {
    const step = this.step;
    const snap = this.snapshot || {};
    const version = this.dataset.version || "";
    const platform = this.dataset.platform || "linux";
    const platformDisplay = this.dataset.platformDisplay || "";
    const host = gatewayHost(snap.gateway_url);
    // Continue stays enabled except while first-use provisioning is running:
    // that is the one window where advancing would leave a half-installed
    // machine behind, and core's `setup.complete` no-ops during it anyway.
    // Host install-state is probe-driven and can lag, so it must never gate.
    const finishDisabled = this.firstRunActive ? "disabled" : "";

    return `
      <div class="sp-setup__split">
        <aside class="sp-setup__brand">
          <div class="sp-setup__mark" data-logo-slot data-preserve></div>
          <div class="sp-setup__pitch">
            <p class="sp-setup__pitch-head">Govern every coding agent.</p>
            <p class="sp-setup__pitch-body">One gateway. Every agent. Every tool call audited.</p>
          </div>
          <footer class="sp-setup__brand-foot">
            <p class="sp-setup__demo">
              <strong data-l10n-id="setup-warning-strong">Demo software.</strong>
              <span data-l10n-id="setup-warning-body">This build is provided for demonstration purposes only and is not licensed for production use.</span>
            </p>
            <p class="sp-setup__meta">
              <span class="sp-setup__version">v${escapeHtml(version)}</span>
              <span class="sp-setup__meta-sep">·</span>
              <a class="sp-setup__docs" href="https://systemprompt.io/docs/bridge/${escapeHtml(platform)}" target="_blank" rel="noopener noreferrer">
                Documentation for ${escapeHtml(platformDisplay)} →
              </a>
              <span class="sp-setup__meta-sep">·</span>
              <a href="mailto:ed@systemprompt.io?subject=systemprompt%20bridge%20licensing">Licensing</a>
            </p>
          </footer>
        </aside>

        <section class="sp-setup__panel">
          <div class="sp-setup__panel-inner">
            ${this._renderSteps()}
            <sp-setup-connection ${step === "connect" ? "data-editable" : ""} ${step === "done" ? "hidden" : ""}></sp-setup-connection>

            <div class="sp-setup__step" data-step="connect" ${step !== "connect" ? "hidden" : ""}>
              <h1 id="setup-heading">${escapeHtml(tr("astound-setup-connect-heading", "Connect this Mac"))}</h1>
              <p class="sp-setup__lede">
                ${escapeHtml(tr("astound-setup-connect-lede",
                  "Sign in with your Astound Salesforce account. Your bridge account is created automatically the first time you sign in."))}
              </p>
              <sp-setup-gateway></sp-setup-gateway>
            </div>

            <div class="sp-setup__step" data-step="agents" ${step !== "agents" ? "hidden" : ""}>
              <h1>${escapeHtml(tr("astound-setup-agents-heading", "Set up your agents"))}</h1>
              <p class="sp-setup__lede">
                Setting up an agent points its inference at <strong>${escapeHtml(host)}</strong>, so every
                request and tool call is governed and audited. You can skip any of them and come back later.
              </p>
              <sp-setup-agents></sp-setup-agents>
              <div class="sp-setup__actions">
                <button class="sp-btn-primary" type="button" data-action="finish" ${finishDisabled}>
                  <span class="sp-btn__label">${escapeHtml(this.firstRunActive ? tr("astound-setup-continuing", "Setting up…") : tr("astound-setup-continue", "Continue"))}</span>
                </button>
              </div>
            </div>

            <div class="sp-setup__step" data-step="done" ${step !== "done" ? "hidden" : ""}>
              ${step === "done" ? this._renderDone() : ""}
            </div>
          </div>
        </section>
      </div>
    `;
  }
}

reactive(SpSetup.prototype, ["snapshot", "step", "anyInstalled", "firstRunActive"]);
customElements.define("sp-setup", SpSetup);
