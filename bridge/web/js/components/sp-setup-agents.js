/* Agent set-up list for the setup overlay — Astound overlay of core's
 * sp-setup-agents.js.
 *
 * Core reduces every host to one boolean: "Install profile" or "Installed ✓".
 * The probe behind it already distinguishes far more — `app_installed` is a
 * tri-state, `profile_state` has four variants (two of them carrying a reason),
 * and the payload also says whether this gateway serves any model the host can
 * use. None of that reached the screen, so setup could offer to configure an
 * app that is not installed, or report success over a profile that is stale.
 *
 * It also reported failures only through `trigger.title` and a console warning,
 * which is indistinguishable from nothing happening. Failures now render.
 *
 * The badge ladder is core's own, from sp-host-card.js `chooseBadge` — order
 * matters and the first match wins. What is added here is the sentence each
 * rung needs a first-run reader to understand, and the action that clears it.
 */
import { SpElement, reactive, escapeHtml } from "/assets/js/components/sp-element.js";
import { bridge } from "/assets/js/bridge.js";
import { tr } from "/assets/js/utils/l10n.js";

/* `unknown` is not a synonym for absent: it means every detector was
 * inconclusive. Core's host_app.rs is explicit that callers must never render
 * it as "not installed", so it gets its own rung. */
const APP_INSTALLED = "installed";
const APP_NOT_INSTALLED = "not_installed";

function profileState(hs) {
  return (hs && hs.profile_state) || {};
}

function classify(host, snap) {
  const hs = host.snapshot;
  if (!hs) {
    return { state: "probing", badge: "sp-badge--muted", label: tr("astound-agent-badge-checking", "checking"),
      sub: tr("astound-agent-sub-probing", "looking for this app on your Mac"), action: "none" };
  }

  // Before the first sync there is no signed manifest, so the instance's host
  // gate is not authoritative and `host_apps` is every host this build knows
  // rather than the ones this installation permits. Offering to set one up
  // then is the fail-open that let a disabled agent be configured on a fresh
  // install. Show it, name the reason, and withhold the action.
  if (snap.hosts_gated === false) {
    return { state: "probing", badge: "sp-badge--muted", label: "checking",
      sub: tr("astound-agent-sub-ungated", "confirming with your gateway which agents this installation allows"),
      action: "none", ungated: true };
  }

  const app = hs.app_installed || "unknown";
  const ps = profileState(hs);
  const proxy = (snap.local_proxy || {}).state;

  if (app === APP_NOT_INSTALLED) {
    return { state: "err", badge: "sp-badge--err", label: tr("astound-agent-badge-absent-app", "not installed"),
      sub: tr("astound-agent-sub-absent-app", "not found on this Mac"), action: "download" };
  }
  if (app !== APP_INSTALLED) {
    return { state: "warn", badge: "sp-badge--muted", label: tr("astound-agent-badge-undetected", "not detected"),
      sub: tr("astound-agent-sub-undetected", "we could not confirm it is installed — you can still set it up"), action: "install" };
  }
  if (ps.kind === "absent") {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-unset", "not set up"),
      sub: tr("astound-agent-sub-unset", "installed, but not routing through the gateway yet"), action: "install" };
  }
  if (ps.kind === "partial") {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-partial", "incomplete"),
      sub: tr("astound-agent-sub-partial", "its configuration is missing required settings"), action: "repair" };
  }
  if (ps.kind === "stale" && ps.reason === "proxy_port") {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-stale", "out of date"),
      sub: tr("astound-agent-sub-stale-port", "the local proxy moved to a new port — set it up again to refresh"), action: "repair" };
  }
  if (ps.kind === "stale") {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-stale", "out of date"),
      sub: tr("astound-agent-sub-stale-secret", "its credential was rotated — set it up again to refresh"), action: "repair" };
  }
  if (host.models_checked && !host.compatible_models_available) {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-no-model", "no model"),
      sub: tr("astound-agent-sub-no-model", "this gateway serves no model this app can use"), action: "none" };
  }
  // Core's ladder has this rung too: the profile is written, but the loopback
  // proxy has not been configured by a host connecting through it yet. Calling
  // that "ready" would be optimistic — nothing has actually routed.
  if (proxy === "Unconfigured") {
    return { state: "warn", badge: "sp-badge--warn", label: tr("astound-agent-badge-unused", "not used yet"),
      sub: tr("astound-agent-sub-unused", "set up — it will connect the first time you use it"), action: "done" };
  }
  if (proxy === "Refused" || proxy === "Timeout" || proxy === "HttpError") {
    return { state: "err", badge: "sp-badge--err", label: tr("astound-agent-badge-proxy-down", "proxy down"),
      sub: tr("astound-agent-sub-proxy-down", "set up, but the local proxy is not accepting connections"), action: "none" };
  }
  return { state: "ok", badge: "sp-badge--ok", label: tr("astound-agent-badge-ready", "ready"),
    sub: hs.host_running ? tr("astound-agent-sub-running", "set up · running now")
                         : tr("astound-agent-sub-idle", "set up · not running"), action: "done" };
}

function check(result, glyph, html) {
  return `<li class="sp-onb-agent__check" data-result="${result}">
    <i aria-hidden="true">${glyph}</i><span>${html}</span></li>`;
}

/** The evidence behind the badge — every line is a field from the probe. */
function checksFor(host) {
  const hs = host.snapshot;
  if (!hs) { return ""; }
  const app = hs.app_installed || "unknown";
  const ps = profileState(hs);
  let out = "";

  if (app === APP_INSTALLED) {
    out += check("ok", "✓", hs.host_running
      ? "Found on this Mac, running right now"
      : "Found on this Mac, not currently running");
  } else if (app === APP_NOT_INSTALLED) {
    out += check("no", "✗", "Not found on this Mac");
  } else {
    out += check("warn", "?", "Could not tell whether it is installed");
  }

  if (ps.kind === "installed") {
    const src = hs.profile_source ? String(hs.profile_source).split(/[\\/]/).pop() : "";
    out += check("ok", "✓", `Routing through the gateway${src ? ` — <code>${escapeHtml(src)}</code>` : ""}`);
  } else if (ps.kind === "partial") {
    const missing = (ps.missing_required || []).map((k) => `<code>${escapeHtml(k)}</code>`).join(", ");
    out += check("warn", "!", `Configuration incomplete${missing ? ` — missing ${missing}` : ""}`);
  } else if (ps.kind === "stale") {
    out += check("warn", "!", ps.reason === "proxy_port"
      ? "Pointing at a local proxy port that has since changed"
      : "Using a credential that has since been rotated");
  } else {
    out += check("idle", "○", "Not routing through the gateway yet");
  }

  if (host.models_checked) {
    if (host.compatible_models_available) {
      const n = (host.compatible_models || []).length;
      out += check("ok", "✓", `${n} model${n === 1 ? "" : "s"} available through this gateway`);
    } else {
      const providers = (host.unconfigured_providers || []).map((p) => `<code>${escapeHtml(p)}</code>`).join(", ");
      out += check("no", "✗", `No compatible model${providers
        ? ` — this gateway has no ${providers} provider configured`
        : " on this gateway"}`);
    }
  }
  return out;
}

const STAGE_HEADLINE = {
  generate: "Could not build the configuration",
  install: "Could not write the configuration",
};

/* Keyed on `ErrorCode`, which core serialises snake_case. `unauthorized` is
 * what `host.profile.install` returns for an underlying PermissionDenied, so it
 * is the one failure with a remedy the user can act on themselves. Anything
 * absent here shows the raw message, which is still more use than silence. */
const FAILURE_HINT = {
  unauthorized: "Writing managed configuration needs administrator rights. Try again and approve the macOS prompt.",
  unreachable: "The gateway could not be reached, so there was no configuration to install. Check the connection above, then try again.",
  timeout: "The gateway took too long to answer. Try again.",
  not_found: "This agent is not one your gateway recognises. It may have been disabled for your installation.",
};

export class SpSetupAgents extends SpElement {
  constructor() {
    super();
    this.snapshot = null;
    this.firstRun = null;
    this.failures = {};
    this.busy = {};

    this.registerAction("install-host", (trigger) => this._install(trigger.dataset.hostId));
    this.registerAction("dismiss", (trigger) => {
      const next = Object.assign({}, this.failures);
      delete next[trigger.dataset.hostId];
      this.failures = next;
    });
    this.registerAction("recheck", async (trigger) => {
      try { await bridge.hostProbe(trigger.dataset.hostId); }
      catch (e) { console.warn("host probe", e); }
    });
    this.registerAction("download", async (trigger) => {
      const url = trigger.dataset.url;
      if (!url) { return; }
      try { await bridge.openExternalUrl(url); }
      catch (e) { console.warn("download", e); }
    });
    this.registerAction("open-config", async (trigger) => {
      try { await bridge.agentOpenConfig(trigger.dataset.hostId); }
      catch (e) { console.warn("open config", e); }
    });
  }

  onConnect() {
    this.classList.add("sp-onb-list");
    this.setAttribute("aria-live", "polite");
    bridge.stateSnapshot().then((s) => {
      this.snapshot = s;
      // A late-mounting component would otherwise show nothing until the next
      // tick; the snapshot carries the run's current state.
      if (s && s.first_run) { this.firstRun = s.first_run; }
    }).catch((e) => console.warn("snapshot failed", e));
    this.bridgeSubscribe("state.changed", (s) => { this.snapshot = s; });
    this.bridgeSubscribe("setup.progress", (p) => { this.firstRun = p; });
    this.bridgeSubscribe("host.changed", (host) => this._mergeHost(host));
  }

  _mergeHost(host) {
    if (!host || !host.id || !this.snapshot) { return; }
    const list = (this.snapshot.host_apps || []).slice();
    const idx = list.findIndex((h) => h.id === host.id);
    if (idx >= 0) { list[idx] = host; } else { list.push(host); }
    this.snapshot = Object.assign({}, this.snapshot, { host_apps: list });
  }

  /* Generate then install. Core reported a failure of either stage through the
   * button's tooltip; both stages now land in a visible block that names which
   * one failed, because the remedies differ. */
  async _install(id) {
    if (!id || this.busy[id]) { return; }
    this.busy = Object.assign({}, this.busy, { [id]: true });
    const cleared = Object.assign({}, this.failures);
    delete cleared[id];
    this.failures = cleared;

    let stage = "generate";
    try {
      const gen = await bridge.hostProfileGenerate(id);
      const path = gen && (gen.path || gen.profile_path);
      if (!path) { throw new Error("the gateway returned no configuration to install"); }
      stage = "install";
      await bridge.hostProfileInstall(id, path);
    } catch (e) {
      const message = (e && e.message) || String(e);
      // `BridgeError` is {scope, code, message}; keep the typed pair, because
      // the remedy is chosen from the code, never from the wording.
      const code = (e && e.code) || "internal";
      console.warn(`install-host (${stage})`, e);
      this.failures = Object.assign({}, this.failures, { [id]: { stage, code, message } });
    } finally {
      const next = Object.assign({}, this.busy);
      delete next[id];
      this.busy = next;
    }
  }

  /* While first use is provisioning, the per-host status IS the list — the
   * manual buttons would race the run. */
  _renderFirstRun() {
    const fr = this.firstRun;
    const glyphs = {
      pending: "○", probing: "…", generating: "…", installing: "…",
      done: "✓", failed: "✗", skipped: "–",
    };
    const results = {
      done: "ok", failed: "no", skipped: "idle",
    };
    const rows = (fr.hosts || []).map((h) => {
      const detail = h.status === "failed"
        ? (h.error || "failed")
        : (h.status === "skipped" ? "not installed on this Mac" : h.status);
      return check(results[h.status] || "idle", glyphs[h.status] || "○",
        `${escapeHtml(h.display_name)} · ${escapeHtml(detail)}`);
    }).join("");

    const syncLabel = {
      pending: "waiting", installing: "syncing…", done: "synced", failed: "failed",
    }[fr.sync] || fr.sync;

    return `
      <div class="sp-onb-agent" data-state="probing">
        <div class="sp-onb-agent__head">
          <span class="sp-onb-agent__name">${escapeHtml(tr("astound-firstrun-heading", "Setting up your agents"))}</span>
          <span class="sp-onb-agent__spacer"></span>
          <span class="sp-badge sp-badge--muted">${escapeHtml(tr("astound-firstrun-badge", "in progress"))}</span>
        </div>
        <p class="sp-onb-agent__sub">${escapeHtml(tr("astound-firstrun-sub", "This takes a few seconds. Leave the window open."))}</p>
        <ul class="sp-onb-agent__checks">
          ${rows}
          ${check("idle", "…", `Plugins &amp; skills · ${escapeHtml(syncLabel)}`)}
        </ul>
      </div>`;
  }

  _renderActions(host, view) {
    const id = escapeHtml(host.id);
    const busy = !!this.busy[host.id];
    const failure = this.failures[host.id];

    if (failure) {
      return `
        <button type="button" class="sp-btn-primary" data-action="install-host" data-host-id="${id}" ${busy ? "disabled" : ""}>
          <span class="sp-btn__label">${escapeHtml(busy ? tr("astound-agent-retrying", "Retrying…") : tr("astound-agent-retry", "Try again"))}</span>
        </button>
        <button type="button" class="sp-btn-ghost" data-action="dismiss" data-host-id="${id}">${escapeHtml(tr("astound-agent-skip", "Skip for now"))}</button>`;
    }
    if (view.ungated) { return ""; }
    switch (view.action) {
      case "download":
        return `
          <button type="button" class="sp-btn-primary" data-action="download" data-url="${escapeHtml(host.download_url || "")}">
            <span class="sp-btn__label">${escapeHtml(tr("astound-agent-get", "Get"))} ${escapeHtml(host.display_name)}</span>
          </button>
          <button type="button" class="sp-btn-ghost" data-action="recheck" data-host-id="${id}">${escapeHtml(tr("astound-agent-recheck", "Re-check"))}</button>`;
      case "install":
        return `
          <button type="button" class="sp-btn-primary" data-action="install-host" data-host-id="${id}" ${busy ? "disabled" : ""}>
            <span class="sp-btn__label">${escapeHtml(busy ? tr("astound-agent-setting-up", "Setting up…") : tr("astound-agent-set-up", "Set up"))}</span>
          </button>`;
      case "repair":
        return `
          <button type="button" class="sp-btn-primary" data-action="install-host" data-host-id="${id}" ${busy ? "disabled" : ""}>
            <span class="sp-btn__label">${escapeHtml(busy ? tr("astound-agent-repairing", "Repairing…") : tr("astound-agent-repair", "Repair"))}</span>
          </button>`;
      case "done":
        return `
          <button type="button" class="sp-btn-ghost" data-action="install-host" data-host-id="${id}" ${busy ? "disabled" : ""}>${escapeHtml(tr("astound-agent-set-up-again", "Set up again"))}</button>
          <button type="button" class="sp-btn-ghost" data-action="open-config" data-host-id="${id}">${escapeHtml(tr("astound-agent-open-config", "Open config"))}</button>`;
      default:
        return `<button type="button" class="sp-btn-ghost" data-action="recheck" data-host-id="${id}">${escapeHtml(tr("astound-agent-recheck", "Re-check"))}</button>`;
    }
  }

  _renderFailure(host) {
    const failure = this.failures[host.id];
    if (!failure) { return ""; }
    const headline = STAGE_HEADLINE[failure.stage] || "Set-up failed";
    // Chosen from the typed code. Matching on the rendered message is the same
    // defect the governance spine was fixed for: a wording change upstream
    // silently stops the remedy from ever being offered.
    const hint = FAILURE_HINT[failure.code] || "";
    return `
      <div class="sp-onb-agent__error">
        <b>${escapeHtml(headline)}</b>
        <code>${escapeHtml(failure.message)}</code>
        ${hint ? `<span>${escapeHtml(hint)}</span>` : ""}
      </div>`;
  }

  _renderHost(host, snap) {
    const view = classify(host, snap);
    const kind = host.kind === "cli_tool" ? tr("astound-agent-kind-cli", "Command line")
                                    : tr("astound-agent-kind-desktop", "Desktop app");
    const checks = view.ungated ? "" : checksFor(host);
    // macOS profile installs are only *offered* to System Settings, so a
    // returned success is not proof the profile landed. Core already words this
    // correctly in `install_action_label`; it was never shown.
    const label = view.action === "install" || view.action === "repair" ? host.install_action_label : "";
    const actions = this._renderActions(host, view);

    return `
      <div class="sp-onb-agent" data-key="${escapeHtml(host.id)}" data-state="${view.state}">
        <div class="sp-onb-agent__head">
          <span class="sp-onb-agent__name">${escapeHtml(host.display_name)}</span>
          <span class="sp-onb-agent__spacer"></span>
          <span class="sp-badge ${view.badge}">${escapeHtml(view.label)}</span>
        </div>
        <p class="sp-onb-agent__sub">${escapeHtml(kind)} · ${escapeHtml(view.sub)}</p>
        ${checks ? `<ul class="sp-onb-agent__checks">${checks}</ul>` : ""}
        ${this._renderFailure(host)}
        ${actions ? `<div class="sp-onb-agent__foot">${actions}</div>` : ""}
        ${label ? `<p class="sp-onb-agent__note">${escapeHtml(label)}</p>` : ""}
      </div>`;
  }

  render() {
    if (this.firstRun && this.firstRun.active) {
      return this._renderFirstRun();
    }
    const snap = this.snapshot || {};
    const all = snap.host_apps || [];
    // The last-sync manifest gates hosts: once any host is enabled, hide the
    // instance-disabled ones (host.changed merges can re-add them).
    const hosts = all.some((h) => h.enabled) ? all.filter((h) => h.enabled) : all;
    if (hosts.length === 0) {
      return `<p class="sp-u-muted">${escapeHtml(tr("astound-agent-empty", "No agents were found on this Mac."))}</p>`;
    }
    return hosts.map((h) => this._renderHost(h, snap)).join("");
  }
}

reactive(SpSetupAgents.prototype, ["snapshot", "firstRun", "failures", "busy"]);
customElements.define("sp-setup-agents", SpSetupAgents);
