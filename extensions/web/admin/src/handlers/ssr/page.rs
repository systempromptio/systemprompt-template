//! The per-request values every server-rendered admin page reads.

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::request::Parts;

use crate::env::AdminEnv;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

// Why: Who is looking, what marketplace scope they hold, and the engine that
// renders for them — the three request extensions every SSR handler needs
// before it can render anything, taken as one extractor so a handler's
// argument list is the page's own inputs.
#[derive(Debug, Clone)]
pub(crate) struct Page {
    pub(crate) engine: AdminTemplateEngine,
    pub(crate) user: UserContext,
    pub(crate) marketplace: MarketplaceContext,
}

impl<S: Send + Sync> FromRequestParts<S> for Page {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(engine) =
            Extension::<AdminTemplateEngine>::from_request_parts(parts, state).await?;
        let Extension(user) = Extension::<UserContext>::from_request_parts(parts, state).await?;
        let Extension(marketplace) =
            Extension::<MarketplaceContext>::from_request_parts(parts, state).await?;
        Ok(Self {
            engine,
            user,
            marketplace,
        })
    }
}

// Why: the environment and the signed-in caller, the pair every console write
// reads before it touches anything else.
#[derive(Debug, Clone)]
pub(crate) struct Caller {
    pub(crate) env: AdminEnv,
    pub(crate) user: UserContext,
}

impl<S: Send + Sync> FromRequestParts<S> for Caller {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(env) = Extension::<AdminEnv>::from_request_parts(parts, state).await?;
        let Extension(user) = Extension::<UserContext>::from_request_parts(parts, state).await?;
        Ok(Self { env, user })
    }
}

// Why: a page handler that also acts on the environment — a form post that
// renders the page it came from.
#[derive(Debug, Clone)]
pub(crate) struct Console {
    pub(crate) env: AdminEnv,
    pub(crate) page: Page,
}

impl<S: Send + Sync> FromRequestParts<S> for Console {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(env) = Extension::<AdminEnv>::from_request_parts(parts, state).await?;
        let page = Page::from_request_parts(parts, state).await?;
        Ok(Self { env, page })
    }
}
