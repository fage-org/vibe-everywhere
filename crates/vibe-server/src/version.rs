use semver::{Version, VersionReq};
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub server_version: String,
    pub git_sha: Option<String>,
    pub build_timestamp: Option<String>,
    pub ios_up_to_date: String,
    pub android_up_to_date: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionCheckResponse {
    pub update_url: Option<String>,
}

impl VersionInfo {
    pub fn current(config: &Config) -> Self {
        Self {
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            git_sha: option_env!("GIT_SHA").map(str::to_string),
            build_timestamp: option_env!("BUILD_TIMESTAMP").map(str::to_string),
            ios_up_to_date: config.ios_up_to_date.clone(),
            android_up_to_date: config.android_up_to_date.clone(),
        }
    }
}

pub fn check_update_url(config: &Config, platform: &str, version: &str) -> VersionCheckResponse {
    let update_url = match platform.to_ascii_lowercase().as_str() {
        "ios" => update_url_for_req(&config.ios_up_to_date, version, &config.ios_store_url),
        "android" => update_url_for_req(
            &config.android_up_to_date,
            version,
            &config.android_store_url,
        ),
        _ => None,
    };

    VersionCheckResponse { update_url }
}

fn update_url_for_req(requirement: &str, version: &str, store_url: &str) -> Option<String> {
    let req = VersionReq::parse(requirement).ok()?;
    let parsed = Version::parse(version).ok();
    match parsed {
        Some(version) if req.matches(&version) => None,
        _ => Some(store_url.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::{VersionInfo, check_update_url};

    fn config() -> Config {
        Config {
            host: "0.0.0.0".parse().unwrap(),
            port: 3005,
            master_secret: "secret".into(),
            ios_up_to_date: ">=1.4.1".into(),
            android_up_to_date: ">=1.4.1".into(),
            ios_store_url: "ios-store".into(),
            android_store_url: "android-store".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        }
    }

    #[test]
    fn version_helper_exposes_build_metadata() {
        let info = VersionInfo::current(&config());
        assert_eq!(info.server_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.ios_up_to_date, ">=1.4.1");
    }

    #[test]
    fn version_route_returns_store_url_for_outdated_ios() {
        let res = check_update_url(&config(), "ios", "1.0.0");
        assert_eq!(res.update_url.as_deref(), Some("ios-store"));
    }

    #[test]
    fn version_route_returns_null_when_current() {
        let res = check_update_url(&config(), "android", "1.4.1");
        assert_eq!(res.update_url, None);
    }
}
