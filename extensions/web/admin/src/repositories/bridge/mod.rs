//! Persistence for the bridge control plane: API keys, device certificates, and
//! exchange codes.

pub mod api_keys;
pub mod bridge_users;
pub mod device_certs;
pub mod error;
pub mod exchange_codes;

pub use api_keys::{
    BridgeApiKeyRow, BridgeIssuedApiKey, EnrollDeviceParams, EnrolledDevice, enroll_device,
    issue_bridge_api_key, list_api_keys_for_user, revoke_bridge_api_key,
};
pub use bridge_users::{BridgeIdentityRow, find_bridge_user};
pub use device_certs::{DeviceCertRow, revoke_device_cert};
pub use error::{BridgeRepoError, Result};
pub use exchange_codes::{EXCHANGE_CODE_TTL_SECONDS, IssuedExchangeCode, issue_exchange_code};
