//! Unit tests for `systemprompt-web-shared` pure logic:
//! - `CampaignLink::full_url` UTM query assembly and `?`/`&` separator choice
//! - `BlogConfigValidated::validate` base-URL scheme/parse validation

#[cfg(test)]
mod campaign_link_full_url;
#[cfg(test)]
mod config_base_url;
