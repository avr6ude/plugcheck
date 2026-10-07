//! Update check against GitHub Releases. Sends nothing but the request itself.

use serde::Serialize;

pub const RELEASES_URL: &str = "https://github.com/avr6ude/plugcheck/releases/latest";
const API_URL: &str = "https://api.github.com/repos/avr6ude/plugcheck/releases/latest";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Update {
    pub version: String,
    pub url: String,
}

fn parse(v: &str) -> Vec<u64> {
    v.trim_start_matches('v').split('.').map(|n| n.parse().unwrap_or(0)).collect()
}

/// True when `latest` is a newer version than `current` ("v0.2.0" > "0.1.9").
pub fn is_newer(latest: &str, current: &str) -> bool {
    parse(latest) > parse(current)
}

/// Newest published release if it beats this build; `None` when up to date or offline.
pub fn check() -> Option<Update> {
    // ponytail: shells out to the system curl instead of pulling in an HTTP client crate.
    let out = std::process::Command::new("/usr/bin/curl")
        .args(["-fsSL", "-m", "10", "-H", "Accept: application/vnd.github+json", API_URL])
        .output()
        .ok()?;
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let tag = json.get("tag_name")?.as_str()?;
    is_newer(tag, env!("CARGO_PKG_VERSION")).then(|| Update {
        version: tag.trim_start_matches('v').to_string(),
        url: json.get("html_url").and_then(|u| u.as_str()).unwrap_or(RELEASES_URL).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("v0.1.10", "0.1.9"));
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(!is_newer("v0.1.1", "0.1.1"));
        assert!(!is_newer("0.1.0", "0.1.1"));
    }
}
