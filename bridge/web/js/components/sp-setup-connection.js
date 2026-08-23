/* Persistent connection strip for the setup overlay.
 *
 * Astound-only component: core's onboarding shows a single probe dot and hides
 * the topbar, footer and shell while `is-setup-mode` is on, so during the one
 * flow where the user most needs to know where they are pointed, every status
 * surface in the app is switched off. This strip answers the four questions
 * that matter — which gateway, who am I, how long is this session good for, is
 * the local proxy up — on every step.
 *
 * It invents no data. `reachability`/`identity` mirror sp-cloud-status.js and
 * `proxy` mirrors sp-proxy-status.js, all read off the same `state.snapshot`
 * the rest of the app uses.
 *
 * It also owns the gateway URL input on the sign-in step (`data-editable`).
 * That input keeps core's `#setup-gateway` id: sp-setup-gateway guards its own
 * `this.gateway` on `document.activeElement.id`, a document-level check that
 * still holds with the field living here. The value round-trips through
 * `gateway.set` → snapshot → sp-setup-gateway, and sign-in is gated on a
 * reachable probe, so the button cannot be pressed before that trip completes.
 */
import { SpElement, reactive, escapeHtml } from "/assets/js/components/sp-element.js";
import { bridge } from "/assets/js/bridge.js";
import { probeView } from "/assets/js/utils/gateway.js";
import { fmtRelative } from "/assets/js/utils/format.js";
import { tr } from "/assets/js/utils/l10n.js";

const PERSIST_DEBOUNCE_MS = 600;

/** `fmtDuration` renders 3480 as "58m 0s"; a session lifetime reads better
 *  without the trailing zero unit. */
function fmtTtl(seconds) {
  const s = Math.max(0, Math.floor(seconds || 0));
  if (s < 60) { return `${s}s`; }
  const m = Math.floor(s / 60);
  if (m < 60) { return `${m} min`; }
  const rem = m % 60;
  return rem ? `${Math.floor(m / 60)} h ${rem} min` : `${Math.floor(m / 60)} h`;
}

/** The read-only row has one panel column to work with, and the host is the part
 *  a reader needs; the full URL stays on the element's title. */
function hostOf(url) {
  try { return new URL(url).host; } catch (e) { return url || "not set"; }
}

/** Mirrors sp-cloud-status.js `identityView`. */
function identityView(snap) {
  const reachable = (snap.gateway_status || {}).state === "reachable";
  const id = snap.verified_identity;
  if (!reachable) {
    // No aside: the gateway row's own reason line already says why, and
    // repeating it here overflowed the strip on a narrow window.
    return { dot: "sp-dot--unknown", text: "—", aside: "", muted: true };
  }
  if (id && (id.email || id.user_id)) {
    return { dot: "sp-dot--ok", text: id.email || id.user_id, aside: id.tenant_id || "", muted: false };
  }
  if (snap.pat_present) {
    return { dot: "sp-dot--probing", text: tr("astound-conn-identity-checking", "checking your credentials…"), aside: "", muted: true };
  }
  return { dot: "sp-dot--warn", text: tr("astound-conn-identity-none", "not signed in yet"), aside: "", muted: true };
}

/** Mirrors sp-proxy-status.js `proxyView`, worded for a first-run reader. */
function proxyView(snap) {
  const p = snap.local_proxy || {};
  switch (p.state) {
    case "Listening":
      return { dot: "sp-dot--ok", text: p.url || "listening", aside: p.latency_ms != null ? `${p.latency_ms} ms` : "" };
    case "Refused":
      return { dot: "sp-dot--err", text: tr("astound-conn-proxy-refused", "not accepting connections"), aside: p.url || "", tone: "err" };
    case "Timeout":
      return { dot: "sp-dot--err", text: tr("astound-conn-proxy-timeout", "timed out"), aside: p.url || "", tone: "err" };
    case "HttpError":
      return { dot: "sp-dot--err", text: p.error || tr("astound-conn-proxy-http", "returned an error"), aside: p.url || "", tone: "err" };
    case "Unconfigured":
      return { dot: "sp-dot--unknown", text: tr("astound-conn-proxy-idle", "starts once an agent is set up"), aside: "", muted: true };
    default:
      return { dot: "sp-dot--unknown", text: tr("astound-conn-proxy-checking", "checking…"), aside: "", muted: true };
  }
}

/** Session lifetime. Under ten minutes is worth flagging, but it is not a fault
 *  — the proxy re-authenticates per request, so it renews itself. */
function sessionView(snap) {
  const tok = snap.cached_token;
  if (!tok) { return null; }
  const ttl = tok.ttl_seconds || 0;
  if (ttl < 600) {
    return { dot: "sp-dot--warn", text: `expires in ${fmtTtl(ttl)}`, aside: tr("astound-conn-session-renews", "renews on next use"), tone: "warn" };
  }
  return { dot: "sp-dot--ok", text: `expires in ${fmtTtl(ttl)}`, aside: "" };
}

function row(view, key, valueHtml, asideHtml) {
  const tone = view.tone ? ` sp-conn__val--${view.tone}` : (view.muted ? " sp-conn__val--muted" : "");
  return `
    <div class="sp-conn__row">
      <span class="sp-dot ${view.dot}" aria-hidden="true"></span>
      <span class="sp-conn__key">${escapeHtml(key)}</span>
      <span class="sp-conn__val${tone}">${valueHtml}</span>
      <span class="sp-conn__aside">${asideHtml || ""}</span>
    </div>`;
}

export class SpSetupConnection extends SpElement {
  static get observedAttributes() { return ["data-editable"]; }

  constructor() {
    super();
    this.snapshot = null;
    this._debounce = null;
    this._lastSaved = "";
    this.registerAction("reprobe", () => {
      bridge.gatewayProbe().catch((e) => console.warn("gateway probe", e));
    });
    this.registerAction("input:gateway", (input) => {
      if (this._debounce) { clearTimeout(this._debounce); }
      this._debounce = setTimeout(() => this._persist(input.value), PERSIST_DEBOUNCE_MS);
    });
    this.addEventListener("blur", (e) => {
      if (e.target && e.target.id === "setup-gateway") {
        if (this._debounce) { clearTimeout(this._debounce); }
        this._persist(e.target.value);
      }
    }, true);
  }

  onConnect() {
    this.setAttribute("role", "status");
    bridge.stateSnapshot().then((s) => { this.snapshot = s; })
      .catch((e) => console.warn("snapshot failed", e));
    this.bridgeSubscribe("state.changed", (s) => { this.snapshot = s; });
  }

  onDisconnect() {
    if (this._debounce) { clearTimeout(this._debounce); this._debounce = null; }
  }

  attributeChangedCallback() { this.invalidate(); }

  async _persist(raw) {
    const url = (raw || "").trim();
    if (!url || url === this._lastSaved) { return; }
    this._lastSaved = url;
    try { await bridge.gatewaySet(url); }
    catch (e) { console.warn("gateway set", e); }
  }

  afterRender() {
    // The reconciler patches `value` only when the rendered attribute changed,
    // so a field the user is editing is never clobbered — but a field they are
    // NOT in still has to follow the snapshot.
    const input = this.querySelector("#setup-gateway");
    if (input && document.activeElement !== input) {
      const url = (this.snapshot && this.snapshot.gateway_url) || "";
      if (input.value !== url) { input.value = url; }
    }
  }

  render() {
    const snap = this.snapshot;
    if (!snap) { return `<div class="sp-conn sp-conn--empty"></div>`; }

    const editable = this.hasAttribute("data-editable");
    const probe = probeView(snap);
    const url = snap.gateway_url || "";

    const gatewayValue = editable
      ? `<input id="setup-gateway" class="sp-conn__edit" type="url"
                value="${escapeHtml(url)}" data-input="gateway"
                autocomplete="off" spellcheck="false"
                placeholder="https://gateway.example.com"
                aria-label="${escapeHtml(tr("astound-conn-gateway-aria", "Gateway address"))}" />`
      : `<span class="sp-conn__url" title="${escapeHtml(url)}">${escapeHtml(hostOf(url))}</span>`;

    const status = (snap.gateway_status || {}).state;
    const latency = status === "reachable" && snap.gateway_status.latency_ms != null
      ? `<span class="sp-conn__probe">${escapeHtml(snap.gateway_status.latency_ms)} ms</span>`
      : "";
    const recheck = `<button type="button" class="sp-conn__btn" data-action="reprobe">${escapeHtml(tr("astound-conn-recheck", "Re-check"))}</button>`;
    // Reachable needs no words — the dot says it. Anything else does.
    const reason = status === "reachable"
      ? ""
      : `<p class="sp-conn__reason" data-state="${escapeHtml(status || "unknown")}">${escapeHtml(probe.text)}</p>`;

    const identity = identityView(snap);
    const proxy = proxyView(snap);
    const session = sessionView(snap);

    let html = `<div class="sp-conn">`;
    html += row({ dot: probe.dot, muted: probe.muted }, tr("astound-conn-key-gateway", "Gateway"), gatewayValue, latency + recheck);
    html += reason;
    html += row(identity, tr("astound-conn-key-identity", "Signed in"),
      escapeHtml(identity.text),
      identity.aside ? escapeHtml(identity.aside) : "");
    if (session) {
      html += row(session, tr("astound-conn-key-session", "Session"), escapeHtml(session.text), escapeHtml(session.aside));
    }
    html += row(proxy, tr("astound-conn-key-proxy", "Proxy"), escapeHtml(proxy.text), escapeHtml(proxy.aside || ""));

    if (snap.last_probe_at_unix) {
      html += `<p class="sp-conn__foot">${escapeHtml(tr("astound-conn-last-checked", "Last checked"))} ${escapeHtml(fmtRelative(snap.last_probe_at_unix))}.</p>`;
    }
    html += `</div>`;
    return html;
  }
}

reactive(SpSetupConnection.prototype, ["snapshot"]);
customElements.define("sp-setup-connection", SpSetupConnection);
