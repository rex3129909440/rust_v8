#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BrowserPlatform {
    Desktop,
    Android,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BrowserVersion {
    major: u16,
    platform: BrowserPlatform,
}

impl BrowserVersion {
    pub(crate) const MIN_SUPPORTED: u16 = 140;
    pub(crate) const MAX_SUPPORTED: u16 = 151;
    pub(crate) const EDGE_150: Self = Self {
        major: 150,
        platform: BrowserPlatform::Desktop,
    };

    pub(crate) fn major(self) -> u16 {
        self.major
    }

    pub(crate) fn is_android(self) -> bool {
        self.platform == BrowserPlatform::Android
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

    pub(crate) fn from_user_agent(user_agent: &str) -> Result<Self, String> {
        let explicit = browser_version_token(user_agent)
            .and_then(|version| version.split('.').next())
            .and_then(|major| major.parse::<u16>().ok());
        let Some(major) = explicit else {
            // Existing application-specific UAs without a Chromium token keep
            // the historical Edge 150 surface. Recognizable tokens are strict.
            return Ok(Self::EDGE_150);
        };
        Self::from_major(major).map(|mut version| {
            if user_agent.contains("Android") {
                version.platform = BrowserPlatform::Android;
            }
            version
        }).map_err(|_| {
            format!(
                "browser API surface only supports Chromium/Edge major versions {}-{}; userAgent requested {major}",
                Self::MIN_SUPPORTED,
                Self::MAX_SUPPORTED,
            )
        })
    }

    pub(crate) fn full_version_from_user_agent(user_agent: &str) -> Option<String> {
        browser_version_token(user_agent).map(str::to_owned)
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
        assert_eq!(
            BrowserVersion::from_user_agent("application-specific-agent")
                .unwrap()
                .major(),
            150
        );
        assert_eq!(
            BrowserVersion::full_version_from_user_agent(
                "Mozilla/5.0 Chrome/149.0.7827.155 Safari/537.36 Edg/147.0.101.2"
            )
            .as_deref(),
            Some("147.0.101.2")
        );
    }
}
