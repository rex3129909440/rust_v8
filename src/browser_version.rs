#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BrowserPlatform {
    Desktop,
    Android,
    AndroidWebView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BrowserVersion {
    major: u16,
    platform: BrowserPlatform,
}

impl BrowserVersion {
    pub(crate) const MIN_SUPPORTED: u16 = 140;
    pub(crate) const MAX_SUPPORTED: u16 = 151;
    pub(crate) const MIN_WEBVIEW_SUPPORTED: u16 = 136;
    pub(crate) const EDGE_150: Self = Self {
        major: 150,
        platform: BrowserPlatform::Desktop,
    };

    pub(crate) fn major(self) -> u16 {
        self.major
    }

    pub(crate) fn is_android(self) -> bool {
        matches!(
            self.platform,
            BrowserPlatform::Android | BrowserPlatform::AndroidWebView
        )
    }

    pub(crate) fn is_webview(self) -> bool {
        self.platform == BrowserPlatform::AndroidWebView
    }

    pub(crate) fn from_major(major: u16) -> Result<Self, String> {
        if !(Self::MIN_SUPPORTED..=Self::MAX_SUPPORTED).contains(&major) {
            return Err(format!(
                "browser API surface only supports Chromium/Edge major versions {}-{}; requested {major}",
                Self::MIN_SUPPORTED,
                Self::MAX_SUPPORTED,
            ));
        }
        Ok(Self {
            major,
            platform: BrowserPlatform::Desktop,
        })
    }

    fn from_webview_major(major: u16) -> Result<Self, String> {
        if !(Self::MIN_WEBVIEW_SUPPORTED..=Self::MAX_SUPPORTED).contains(&major) {
            return Err(format!(
                "Android WebView surface only supports Chromium major versions {}-{}; requested {major}",
                Self::MIN_WEBVIEW_SUPPORTED,
                Self::MAX_SUPPORTED,
            ));
        }
        Ok(Self {
            major,
            platform: BrowserPlatform::AndroidWebView,
        })
    }

    pub(crate) fn from_user_agent(user_agent: &str) -> Result<Self, String> {
        let explicit = browser_version_token(user_agent)
            .and_then(|version| version.split('.').next())
            .and_then(|major| major.parse::<u16>().ok());
        let Some(major) = explicit else {
            // Existing application-specific UAs without a Chromium token use
            // the version-150 compatibility surface. Android WebViews often
            // replace the product token while retaining the Android platform
            // marker, so preserve that platform instead of silently selecting
            // the desktop table.
            let mut version = Self::EDGE_150;
            if contains_android_marker(user_agent) {
                version.platform = android_platform(user_agent);
            }
            return Ok(version);
        };
        if contains_android_marker(user_agent)
            && android_platform(user_agent) == BrowserPlatform::AndroidWebView
        {
            return Self::from_webview_major(major);
        }
        Self::from_major(major)
            .map(|mut version| {
                if contains_android_marker(user_agent) {
                    version.platform = BrowserPlatform::Android;
                }
                version
            })
            .map_err(|_| {
            format!(
                "browser API surface only supports Chromium/Edge major versions {}-{}; userAgent requested {major}",
                Self::MIN_SUPPORTED,
                Self::MAX_SUPPORTED,
            )
            })
    }

    pub(crate) fn from_user_agent_with_profile_hint(
        user_agent: &str,
        android_hint: bool,
        webview_hint: bool,
        major_hint: Option<u16>,
    ) -> Result<Self, String> {
        if browser_version_token(user_agent).is_some() {
            let mut version = Self::from_user_agent(user_agent)?;
            if webview_hint && version.is_android() {
                version.platform = BrowserPlatform::AndroidWebView;
            }
            return Ok(version);
        }
        let selected_major = major_hint.unwrap_or(Self::EDGE_150.major);
        let mut version = if webview_hint {
            Self::from_webview_major(selected_major)?
        } else {
            Self::from_major(selected_major)?
        };
        if android_hint || contains_android_marker(user_agent) {
            version.platform = if webview_hint {
                BrowserPlatform::AndroidWebView
            } else {
                android_platform(user_agent)
            };
        }
        Ok(version)
    }

    pub(crate) fn full_version_from_user_agent(user_agent: &str) -> Option<String> {
        browser_version_token(user_agent).map(str::to_owned)
    }
}

fn contains_android_marker(user_agent: &str) -> bool {
    user_agent.to_ascii_lowercase().contains("android")
}

fn android_platform(user_agent: &str) -> BrowserPlatform {
    let lower = user_agent.trim_start().to_ascii_lowercase();
    if lower.contains("; wv") || lower.contains(" version/4.0") {
        BrowserPlatform::AndroidWebView
    } else {
        BrowserPlatform::Android
    }
}

fn browser_version_token(user_agent: &str) -> Option<&str> {
    ["EdgA/", "EdgiOS/", "Edg/", "Chrome/", "CriOS/"]
        .into_iter()
        .find_map(|marker| version_after(user_agent, marker))
}

fn version_after<'a>(user_agent: &'a str, marker: &str) -> Option<&'a str> {
    let tail = user_agent.split_once(marker)?.1;
    let length = tail
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit() || **byte == b'.')
        .count();
    let version = tail.get(..length)?.trim_end_matches('.');
    if version.is_empty()
        || version.split('.').any(|component| {
            component.is_empty() || !component.bytes().all(|byte| byte.is_ascii_digit())
        })
    {
        return None;
    }
    Some(version)
}

#[cfg(test)]
mod tests {
    use super::BrowserVersion;

    #[test]
    fn parses_edge_before_chrome_and_enforces_supported_range() {
        let ua = "Mozilla/5.0 Chrome/149.0.0.0 Safari/537.36 Edg/147.0.0.0";
        assert_eq!(BrowserVersion::from_user_agent(ua).unwrap().major(), 147);
        assert_eq!(
            BrowserVersion::from_user_agent("Mozilla/5.0 Chrome/140.0.0.0")
                .unwrap()
                .major(),
            140
        );
        assert!(BrowserVersion::from_user_agent("Chrome/139.0.0.0").is_err());
        assert_eq!(
            BrowserVersion::from_user_agent("Chrome/151.0.0.0")
                .unwrap()
                .major(),
            151
        );
        let android = BrowserVersion::from_user_agent(
            "Mozilla/5.0 (Linux; Android 10; K) Chrome/149.0.0.0 Mobile Safari/537.36",
        )
        .unwrap();
        assert!(android.is_android());
        assert!(!android.is_webview());
        assert_eq!(
            BrowserVersion::from_user_agent("application-specific-agent")
                .unwrap()
                .major(),
            150
        );
        let android_webview = BrowserVersion::from_user_agent(
            "Mozilla/5.0 (Linux; Android 15; Pixel WebView) MyApplication/9.4",
        )
        .unwrap();
        assert_eq!(android_webview.major(), 150);
        assert!(android_webview.is_android());
        let app_specific = BrowserVersion::from_user_agent_with_profile_hint(
            "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)",
            true,
            false,
            Some(147),
        )
        .unwrap();
        assert_eq!(app_specific.major(), 147);
        assert!(app_specific.is_android());
        assert!(!app_specific.is_webview());
        let app_webview = BrowserVersion::from_user_agent_with_profile_hint(
            "wizz-air/8.1.9 (com.wizzair.WizzAirApp; build:2207; android 9)",
            true,
            true,
            Some(147),
        )
        .unwrap();
        assert!(app_webview.is_webview());
        assert_eq!(
            BrowserVersion::full_version_from_user_agent(
                "Mozilla/5.0 Chrome/149.0.7827.155 Safari/537.36 Edg/147.0.101.2"
            )
            .as_deref(),
            Some("147.0.101.2")
        );
    }
}
