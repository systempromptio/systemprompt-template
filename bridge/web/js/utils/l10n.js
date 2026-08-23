/* Translation lookup with a fallback that actually falls back.
 *
 * Two things make localising component-rendered markup a trap in this GUI, and
 * both are easy to get wrong:
 *
 * 1. `t(id)` returns *the id itself* when the key is missing, not an empty
 *    string. So the idiom `t("some-key") || "English"` never reaches the
 *    fallback — a missing key renders the literal text `some-key` on screen.
 * 2. `data-l10n-id` is applied by `hydrate()`, which runs once at init, before
 *    these components first render. Nothing re-hydrates nodes produced by a
 *    later `render()`, so the attribute is inert in component output. It is
 *    still worth carrying as a marker for a future catalogue, but it does not
 *    localise anything on its own.
 *
 * `tr()` handles both: it asks the catalogue, treats an echoed id as a miss,
 * and returns the English the call site supplies. Every user-visible string in
 * the setup overlay goes through it, so the strings are keyed and ready the
 * moment an Astound catalogue exists — and read correctly until then.
 *
 * A full Astound catalogue needs more than this file: `i18n.js` fetches exactly
 * one `bridge.ftl`, and the build overlay replaces whole files, so shipping one
 * would shadow every core string rather than extend them. Adding brand keys
 * upstream, or overlaying the loader, is the next step.
 */
import { t } from "/assets/js/i18n.js";

/**
 * @param {string} id catalogue key
 * @param {string} fallback English to use when the key is absent
 * @param {object} [args] interpolation arguments
 */
export function tr(id, fallback, args) {
  const msg = t(id, args);
  // `t` echoes the id on a miss; an echo means "not translated".
  return (typeof msg === "string" && msg !== id && msg.length > 0) ? msg : fallback;
}
