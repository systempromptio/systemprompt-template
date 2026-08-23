import { tr } from "/assets/js/utils/l10n.js";

export function probeView(snap) {
  const status = (snap && snap.gateway_status) || { state: "unknown" };
  if (status.state === "reachable") {
    return { dot: "sp-dot--ok", muted: false, text: tr("setup-gateway-reachable", `reachable · ${status.latency_ms}ms`, { latency: status.latency_ms }) };
  }
  if (status.state === "probing") {
    return { dot: "sp-dot--probing", muted: true, text: tr("setup-gateway-probing", "probing…") };
  }
  if (status.state === "unreachable") {
    return { dot: "sp-dot--err", muted: false, text: tr("setup-gateway-unreachable", `unreachable · ${status.reason || "unknown"}`, { reason: status.reason || "unknown" }) };
  }
  const empty = !(snap && snap.gateway_url);
  return { dot: "sp-dot--unknown", muted: true, text: empty ? (tr("setup-gateway-empty", "enter a URL to probe…")) : (tr("setup-gateway-not-probed", "not probed yet")) };
}

export function probeErrorMessage(snap) {
  if (!snap) { return ""; }
  const status = snap.gateway_status || { state: "unknown" };
  const verified = snap.verified_identity && snap.verified_identity.user_id;
  if (status.state === "reachable" && snap.pat_present && !verified) {
    return tr("astound-gateway-token-rejected", "Your access token was rejected. Issue a fresh one and try again.");
  }
  if (status.state === "unreachable" && snap.pat_present) {
    return tr("astound-gateway-unreachable", `Gateway unreachable: ${status.reason || "unknown error"}`);
  }
  return "";
}

export function isPendingResolved(snap, pendingSinceMs) {
  if (!snap) { return false; }
  const probeState = (snap.gateway_status && snap.gateway_status.state) || "unknown";
  const configured = probeState === "reachable" && snap.verified_identity && snap.verified_identity.user_id;
  const elapsed = pendingSinceMs > 0 ? (Date.now() - pendingSinceMs) : 0;
  return configured || probeState === "unreachable" || elapsed > 15000;
}

export function patLinkFor(gateway) {
  const gw = (gateway || "").trim().replace(/\/+$/, "");
  if (gw) { return `${gw}/admin/login`; }
  return "#";
}

function escapeHtml(s) {
  if (s == null) { return ""; }
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

function gatewayHost(url) {
  try { return new URL(url).host; } catch (e) { return "the gateway"; }
}

export function renderGatewayForm(state) {
  const link = patLinkFor(state.gateway);
  const linkDisabled = link === "#";
  const editBtn = state.patSaved ? `<button class="sp-btn-ghost" type="button" data-action="edit-pat">${escapeHtml(tr("astound-gateway-edit", "Edit"))}</button>` : "";
  const errBlock = state.error
    ? `<div class="sp-setup__error" role="alert"><b aria-hidden="true">✗</b><span>${escapeHtml(state.error)}</span></div>`
    : "";
  const btnLabel = state.pending ? (tr("setup-connecting", "Connecting…")) : "Connect";
  const snap = state.snapshot || {};
  const signInLabel = snap.sign_in_label || "Sign in to your gateway";
  const signInBusy = state.signingIn;
  const signInText = signInBusy ? tr("setup-signing-in", "Waiting for your browser…") : signInLabel;
  const keepChecked = state.keepSignedIn === false ? "" : "checked";
  const host = gatewayHost(snap.gateway_url || state.gateway);
  // The device-link flow round-trips through the gateway's browser login, so an
  // unreachable gateway can only ever fail — gate the button and say why rather
  // than opening a browser at a dead host.
  const reachable = (snap.gateway_status || {}).state === "reachable";
  const signInDisabled = signInBusy || state.pending || !reachable;
  const cancelBtn = signInBusy
    ? `<button class="sp-btn-ghost" type="button" data-action="cancel-sign-in">
        <span class="sp-btn__label">${escapeHtml(tr("setup-sign-in-cancel", "Cancel"))}</span>
      </button>`
    : "";
  // The gateway address, its probe state and the re-check control all live in
  // <sp-setup-connection> above this form now — it is the one place that answers
  // "where am I pointed", and it stays on screen for the whole flow. What is
  // left here is the decision: sign in, or fall back to a pasted token.
  const hint = reachable || signInBusy || state.pending
    ? `<p class="sp-setup__hint">Opens your browser to sign in on ${escapeHtml(host)}.
        This Mac is linked automatically — there is no code to copy.</p>`
    : `<p class="sp-setup__hint sp-setup__hint--gate">Can't reach ${escapeHtml(host)} yet, so signing
        in would fail. Check the address above, or your VPN, then re-check.</p>`;

  return `
    <div class="sp-setup__actions">
      <button class="sp-btn-primary" type="button" ${signInDisabled ? "disabled" : ""} data-action="sign-in">
        <span class="sp-btn__label">${escapeHtml(signInText)}</span>
      </button>
      ${cancelBtn}
      <label class="sp-setup__keep">
        <input id="setup-keep" type="checkbox" ${keepChecked} ${signInBusy ? "disabled" : ""} data-input="keep" />
        <span>${escapeHtml(tr("astound-gateway-keep", "Stay signed in on this Mac"))}</span>
      </label>
      ${hint}
    </div>
    ${errBlock}
    <details class="sp-setup__advanced">
      <summary>${escapeHtml(tr("astound-gateway-advanced", "Use an access token instead"))}</summary>
      <div class="sp-setup__field">
        <label for="setup-pat" data-l10n-id="setup-pat-label">Personal access token</label>
        <input id="setup-pat" type="password" placeholder="sp-live-…" autocomplete="off" spellcheck="false" data-input="pat" />
        <p class="sp-setup__hint">
          <span data-l10n-id="setup-pat-hint">Don't have one yet?</span>
          <a class="sp-setup__pat-link ${linkDisabled ? "is-disabled" : ""}" href="${escapeHtml(link)}" target="_blank" rel="noopener noreferrer" aria-disabled="${linkDisabled}">${escapeHtml(tr("astound-gateway-pat-link", "Open the gateway login \u2192"))}</a>
          ${editBtn}
        </p>
      </div>
      <div class="sp-setup__actions">
        <button class="sp-btn-ghost" type="button" ${state.pending ? "disabled" : ""} data-action="connect">
          <span class="sp-btn__label">${escapeHtml(btnLabel)}</span>
        </button>
      </div>
    </details>
  `;
}
