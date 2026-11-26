//! A hop-verified MCP caller for toolset dispatch tests, minted the way
//! core's proxy mints one: a hop token signed by a deployment token authority
//! for the server, the context bound from that token, and the executor's
//! caller verification over the same keys. This is the path the authoring
//! guide names ("Testing an MCP toolset with a verified caller"), built from
//! the public constructors because core's fixture crate is not published.

use std::sync::{Arc, OnceLock};

use systemprompt::extension::mcp::VerifiedCaller;
use systemprompt::identifiers::{JwtToken, McpServerId};
use systemprompt::mcp::{CallerVerification, ServerCallers};
use systemprompt::models::auth::{McpHopExecution, McpHopIdentity, TokenAuthority};
use systemprompt::models::execution::context::RequestContext;
use systemprompt::security::DeploymentTokenAuthority;
use systemprompt::security::keys::{RsaSigningKey, SigningKeys, VerificationKeyring};

pub const HOP_ISSUER: &str = "http://127.0.0.1";

fn signing_keys() -> &'static SigningKeys {
    static KEYS: OnceLock<SigningKeys> = OnceLock::new();
    KEYS.get_or_init(|| {
        SigningKeys::new(
            RsaSigningKey::generate().expect("generate the hop signing key"),
            &[0; 32],
        )
        .expect("build the hop signing keys")
    })
}

fn authority() -> Arc<dyn TokenAuthority> {
    Arc::new(DeploymentTokenAuthority::new(
        signing_keys().clone(),
        HOP_ISSUER,
    ))
}

#[must_use]
pub fn verified_caller(server: &McpServerId, ctx: &RequestContext) -> VerifiedCaller {
    let identity = McpHopIdentity {
        user_id: ctx.user_id().clone(),
        user: ctx.user.clone(),
        session_id: ctx.session_id().clone(),
        user_type: ctx.auth.user_type,
        client_id: ctx.client_id().cloned(),
        act_chain: ctx.act_chain().to_vec(),
        purpose: ctx.caller_purpose().map(str::to_owned),
        interaction: None,
        mcp_session: None,
        execution: Some(McpHopExecution::of(ctx)),
    };
    let token = authority()
        .mint_mcp_hop(server, &identity)
        .expect("mint the mcp hop token");
    VerifiedCaller::presented(JwtToken::new(token), identity.bind(ctx.clone()))
}

#[must_use]
pub fn server_callers(server: &McpServerId) -> ServerCallers {
    ServerCallers {
        server: server.clone(),
        verification: CallerVerification::new(
            VerificationKeyring::new(signing_keys().verification_keys().clone())
                .expect("the hop verification keyring"),
            HOP_ISSUER,
        ),
    }
}
