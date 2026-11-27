//! The account operations the pill and the page share: make active, sign out,
//! remove, trust a key and Sync now. The accounts write their store on each
//! change, so every call runs on the blocking pool and reports back as a
//! notice.

use gpui_kit::{Context, Entity, SharedString, Task};
use systemprompt_device::accounts::AccountsError;
use systemprompt_device::protocol::ids::AccountId;
use systemprompt_device::view::KeyFingerprint;
use systemprompt_harness_shared::Tone;

use crate::app::{Services, runtime};
use crate::governance::feeds;
use crate::state::AppState;
use crate::strings::governance as text;

pub(crate) fn notify<T: 'static>(
    app: &Entity<AppState>,
    tone: Tone,
    message: impl Into<SharedString>,
    cx: &mut Context<T>,
) {
    let message = message.into();
    app.update(cx, |_, cx| AppState::notify_user(tone, message, cx));
}

pub(crate) fn activate<T: 'static>(
    services: &Services,
    app: Entity<AppState>,
    target: (AccountId, String),
    cx: &Context<T>,
) -> Task<()> {
    let accounts = services.accounts().clone();
    let (id, name) = target;
    runtime::blocking(
        cx,
        services.runtime(),
        move || accounts.activate(&id),
        move |_, result, cx| {
            report(
                &app,
                result,
                &text::switched(&name),
                text::ACTIVATE_FAILED,
                cx,
            );
        },
    )
}

pub(crate) fn sign_out<T: 'static>(
    services: &Services,
    app: Entity<AppState>,
    id: AccountId,
    cx: &Context<T>,
) -> Task<()> {
    let accounts = services.accounts().clone();
    runtime::blocking(
        cx,
        services.runtime(),
        move || accounts.sign_out(&id),
        move |_, result, cx| {
            report_failure(&app, result, text::SIGN_OUT_FAILED, cx);
        },
    )
}

pub(crate) fn remove<T: 'static>(
    services: &Services,
    app: Entity<AppState>,
    id: AccountId,
    cx: &Context<T>,
) -> Task<()> {
    let accounts = services.accounts().clone();
    runtime::blocking(
        cx,
        services.runtime(),
        move || accounts.remove(&id),
        move |_, result, cx| {
            report_failure(&app, result, text::REMOVE_FAILED, cx);
        },
    )
}

pub(crate) fn trust<T: 'static>(
    services: &Services,
    app: Entity<AppState>,
    target: (AccountId, KeyFingerprint),
    cx: &Context<T>,
) -> Task<()> {
    let accounts = services.accounts().clone();
    let (id, fingerprint) = target;
    runtime::blocking(
        cx,
        services.runtime(),
        move || accounts.trust(&id, &fingerprint),
        move |_, result, cx| {
            report(&app, result, text::KEY_TRUSTED, text::TRUST_FAILED, cx);
        },
    )
}

pub(crate) fn sync_now<T: 'static>(
    services: &Services,
    app: &Entity<AppState>,
    cx: &mut Context<T>,
) {
    match feeds::sync_engine(services) {
        Some(engine) => engine.sync_now(),
        None => notify(app, Tone::Warning, text::SYNC_UNAVAILABLE, cx),
    }
}

fn report<T: 'static>(
    app: &Entity<AppState>,
    result: Result<(), AccountsError>,
    success: &str,
    failure: &'static str,
    cx: &mut Context<T>,
) {
    match result {
        Ok(()) => notify(app, Tone::Success, success.to_owned(), cx),
        Err(error) => {
            tracing::warn!(error = %error, "account change failed");
            notify(app, Tone::Danger, failure, cx);
        },
    }
}

fn report_failure<T: 'static>(
    app: &Entity<AppState>,
    result: Result<(), AccountsError>,
    failure: &'static str,
    cx: &mut Context<T>,
) {
    if let Err(error) = result {
        tracing::warn!(error = %error, "account change failed");
        notify(app, Tone::Danger, failure, cx);
    }
}
