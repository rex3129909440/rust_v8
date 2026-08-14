pub const DEFAULT_CHROME_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36";

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UserAgentBrandFingerprint {
    pub brand: String,
    pub version: String,
    pub full_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct UserAgentDataFingerprint {
    pub brands: Vec<UserAgentBrandFingerprint>,
    pub mobile: bool,
    pub platform: String,
    pub architecture: String,
    pub bitness: String,
    pub model: String,
    pub platform_version: String,
    pub ua_full_version: String,
    pub wow64: bool,
    pub form_factors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct NetworkFingerprint {
    pub effective_type: String,
    pub rtt: u32,
    pub downlink: f64,
    pub save_data: bool,
    #[serde(default = "default_connection_type")]
    pub connection_type: String,
    #[serde(default = "default_downlink_max")]
    pub downlink_max: f64,
}

fn default_connection_type() -> String {
    "wifi".to_owned()
}

fn default_downlink_max() -> f64 {
    f64::INFINITY
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SpeechVoiceFingerprint {
    pub voice_uri: String,
    pub name: String,
    pub lang: String,
    pub local_service: bool,
    pub is_default: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SpeechFingerprint {
    pub voices: Vec<SpeechVoiceFingerprint>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct NavigatorFingerprint {
    pub user_agent: String,
    pub app_version: String,
    pub app_code_name: String,
    pub app_name: String,
    pub platform: String,
    pub product: String,
    pub product_sub: String,
    pub vendor: String,
    pub vendor_sub: String,
    pub language: String,
    pub languages: Vec<String>,
    pub hardware_concurrency: u32,
    pub device_memory_gb: f64,
    pub max_touch_points: u32,
    pub cookie_enabled: bool,
    pub on_line: bool,
    pub webdriver: bool,
    pub pdf_viewer_enabled: bool,
    pub do_not_track: Option<String>,
    #[serde(default)]
    pub user_activation_has_been_active: bool,
    #[serde(default)]
    pub user_activation_is_active: bool,
    pub user_agent_data: UserAgentDataFingerprint,
    pub network: NetworkFingerprint,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EdgeFingerprint {
    pub id: String,
    pub locale: crate::fingerprint_surface::LocaleFingerprint,
    pub navigator: NavigatorFingerprint,
    pub screen: crate::fingerprint_surface::ScreenFingerprint,
    pub rendering: crate::fingerprint_surface::RenderingFingerprint,
    pub storage: crate::fingerprint_surface::StorageFingerprint,
    #[serde(default)]
    pub speech: SpeechFingerprint,
    #[serde(default)]
    pub fonts: crate::fingerprint_environment::FontFingerprint,
    #[serde(default)]
    pub css: crate::fingerprint_environment::CssFingerprint,
    #[serde(default)]
    pub document: crate::fingerprint_environment::DocumentFingerprint,
    #[serde(default)]
    pub media: crate::fingerprint_environment::MediaFingerprint,
    #[serde(default)]
    pub permissions: crate::fingerprint_environment::PermissionsFingerprint,
    #[serde(default)]
    pub battery: crate::fingerprint_environment::BatteryFingerprint,
    #[serde(default)]
    pub geolocation: crate::fingerprint_environment::GeolocationFingerprint,
    #[serde(default)]
    pub media_preferences: crate::fingerprint_environment::MediaPreferencesFingerprint,
    #[serde(default)]
    pub plugins: crate::fingerprint_environment::PluginListFingerprint,
    #[serde(default)]
    pub hardware_devices: crate::fingerprint_environment::HardwareDevicesFingerprint,
    #[serde(default)]
    pub sensors: crate::fingerprint_environment::SensorsFingerprint,
    #[serde(default)]
    pub timing: crate::fingerprint_environment::TimingFingerprint,
    #[serde(default)]
    pub xr: crate::fingerprint_environment::XrFingerprint,
    #[serde(default)]
    pub memory: crate::fingerprint_environment::MemoryFingerprint,
    #[serde(default)]
    pub performance: crate::fingerprint_performance::PerformanceFingerprint,
}

impl Default for UserAgentDataFingerprint {
    fn default() -> Self {
        Self {
            brands: vec![
                UserAgentBrandFingerprint {
                    brand: "Not_A Brand".to_owned(),
                    version: "99".to_owned(),
                    full_version: "99.0.0.0".to_owned(),
                },
                UserAgentBrandFingerprint {
                    brand: "Chromium".to_owned(),
                    version: "150".to_owned(),
                    full_version: "150.0.0.0".to_owned(),
                },
                UserAgentBrandFingerprint {
                    brand: "Google Chrome".to_owned(),
                    version: "150".to_owned(),
                    full_version: "150.0.0.0".to_owned(),
                },
            ],
            mobile: false,
            platform: "Windows".to_owned(),
            architecture: "x86".to_owned(),
            bitness: "64".to_owned(),
            model: String::new(),
            platform_version: "19.0.0".to_owned(),
            ua_full_version: "150.0.0.0".to_owned(),
            wow64: false,
            form_factors: vec!["Desktop".to_owned()],
        }
    }
}

impl Default for NetworkFingerprint {
    fn default() -> Self {
        Self {
            effective_type: "4g".to_owned(),
            rtt: 100,
            downlink: 1.75,
            save_data: false,
            connection_type: default_connection_type(),
            downlink_max: default_downlink_max(),
        }
    }
}

impl Default for SpeechFingerprint {
    fn default() -> Self {
        Self {
            voices: vec![
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft Huihui - Chinese (Simplified, PRC)".to_owned(),
                    name: "Microsoft Huihui - Chinese (Simplified, PRC)".to_owned(),
                    lang: "zh-CN".to_owned(),
                    local_service: true,
                    is_default: true,
                },
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft Kangkang - Chinese (Simplified, PRC)".to_owned(),
                    name: "Microsoft Kangkang - Chinese (Simplified, PRC)".to_owned(),
                    lang: "zh-CN".to_owned(),
                    local_service: true,
                    is_default: false,
                },
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft Yaoyao - Chinese (Simplified, PRC)".to_owned(),
                    name: "Microsoft Yaoyao - Chinese (Simplified, PRC)".to_owned(),
                    lang: "zh-CN".to_owned(),
                    local_service: true,
                    is_default: false,
                },
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft David - English (United States)".to_owned(),
                    name: "Microsoft David - English (United States)".to_owned(),
                    lang: "en-US".to_owned(),
                    local_service: true,
                    is_default: false,
                },
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft Mark - English (United States)".to_owned(),
                    name: "Microsoft Mark - English (United States)".to_owned(),
                    lang: "en-US".to_owned(),
                    local_service: true,
                    is_default: false,
                },
                SpeechVoiceFingerprint {
                    voice_uri: "Microsoft Zira - English (United States)".to_owned(),
                    name: "Microsoft Zira - English (United States)".to_owned(),
                    lang: "en-US".to_owned(),
                    local_service: true,
                    is_default: false,
                },
            ],
        }
    }
}

impl Default for NavigatorFingerprint {
    fn default() -> Self {
        let user_agent = DEFAULT_CHROME_USER_AGENT.to_owned();
        Self {
            app_version: user_agent
                .strip_prefix("Mozilla/")
                .unwrap_or(&user_agent)
                .to_owned(),
            user_agent,
            app_code_name: "Mozilla".to_owned(),
            app_name: "Netscape".to_owned(),
            platform: "Win32".to_owned(),
            product: "Gecko".to_owned(),
            product_sub: "20030107".to_owned(),
            vendor: "Google Inc.".to_owned(),
            vendor_sub: String::new(),
            language: "zh-CN".to_owned(),
            languages: vec![
                "zh-CN".to_owned(),
                "en".to_owned(),
                "en-GB".to_owned(),
                "en-US".to_owned(),
            ],
            hardware_concurrency: 28,
            device_memory_gb: 8.0,
            max_touch_points: 10,
            cookie_enabled: true,
            on_line: true,
            webdriver: true,
            pdf_viewer_enabled: true,
            do_not_track: None,
            user_activation_has_been_active: false,
            user_activation_is_active: false,
            user_agent_data: UserAgentDataFingerprint::default(),
            network: NetworkFingerprint::default(),
        }
    }
}

impl Default for EdgeFingerprint {
    fn default() -> Self {
        Self {
            id: "windows-10-chrome-150".to_owned(),
            locale: crate::fingerprint_surface::LocaleFingerprint::default(),
            navigator: NavigatorFingerprint::default(),
            screen: crate::fingerprint_surface::ScreenFingerprint::default(),
            rendering: crate::fingerprint_surface::RenderingFingerprint::default(),
            storage: crate::fingerprint_surface::StorageFingerprint::default(),
            speech: SpeechFingerprint::default(),
            fonts: crate::fingerprint_environment::FontFingerprint::default(),
            css: crate::fingerprint_environment::CssFingerprint::default(),
            document: crate::fingerprint_environment::DocumentFingerprint::default(),
            media: crate::fingerprint_environment::MediaFingerprint::default(),
            permissions: crate::fingerprint_environment::PermissionsFingerprint::default(),
            battery: crate::fingerprint_environment::BatteryFingerprint::default(),
            geolocation: crate::fingerprint_environment::GeolocationFingerprint::default(),
            media_preferences: crate::fingerprint_environment::MediaPreferencesFingerprint::default(
            ),
            plugins: crate::fingerprint_environment::PluginListFingerprint::default(),
            hardware_devices: crate::fingerprint_environment::HardwareDevicesFingerprint::default(),
            sensors: crate::fingerprint_environment::SensorsFingerprint::default(),
            timing: crate::fingerprint_environment::TimingFingerprint::default(),
            xr: crate::fingerprint_environment::XrFingerprint::default(),
            memory: crate::fingerprint_environment::MemoryFingerprint::default(),
            performance: crate::fingerprint_performance::PerformanceFingerprint::default(),
        }
    }
}

impl EdgeFingerprint {
    pub(crate) fn synchronize_default_browser_version(&mut self) {
        let user_agent = self.navigator.user_agent.clone();
        self.synchronize_default_android_platform(&user_agent);
        self.navigator
            .user_agent_data
            .synchronize_default_browser_version(&user_agent);
    }

    /// Convert only untouched desktop defaults when an Android UA selects the
    /// mobile Chromium surface. Explicit caller values always win.
    fn synchronize_default_android_platform(&mut self, user_agent: &str) {
        let Ok(browser) = crate::browser_version::BrowserVersion::from_user_agent(user_agent)
        else {
            return;
        };
        if !browser.is_android() {
            return;
        }

        let navigator_defaults = NavigatorFingerprint::default();
        let navigator_is_untouched_desktop = self.navigator.platform == navigator_defaults.platform
            && self.navigator.hardware_concurrency == navigator_defaults.hardware_concurrency
            && self.navigator.device_memory_gb == navigator_defaults.device_memory_gb
            && self.navigator.max_touch_points == navigator_defaults.max_touch_points
            && self.navigator.pdf_viewer_enabled == navigator_defaults.pdf_viewer_enabled;
        if navigator_is_untouched_desktop {
            self.navigator.platform = "Linux armv81".to_owned();
            self.navigator.hardware_concurrency = 8;
            self.navigator.device_memory_gb = 4.0;
            self.navigator.max_touch_points = 5;
            self.navigator.pdf_viewer_enabled = false;
        }

        let data_defaults = UserAgentDataFingerprint::default();
        let data = &mut self.navigator.user_agent_data;
        let data_is_untouched_desktop = data == &data_defaults;
        if data_is_untouched_desktop {
            data.mobile = true;
            data.platform = "Android".to_owned();
            data.architecture.clear();
            data.bitness.clear();
            data.model = android_model_from_user_agent(user_agent).unwrap_or_default();
            data.platform_version = android_version_from_user_agent(user_agent)
                .map(normalize_three_component_version)
                .unwrap_or_default();
            data.form_factors = vec!["Mobile".to_owned()];
            let major = browser.major().to_string();
            let full_version =
                crate::browser_version::BrowserVersion::full_version_from_user_agent(user_agent)
                    .unwrap_or_else(|| format!("{major}.0.0.0"));
            data.brands = vec![
                UserAgentBrandFingerprint {
                    brand: "Chromium".to_owned(),
                    version: major,
                    full_version,
                },
                UserAgentBrandFingerprint {
                    brand: "Not=A?Brand".to_owned(),
                    version: "99".to_owned(),
                    full_version: "99.0.0.0".to_owned(),
                },
            ];
        }

        let preference_defaults =
            crate::fingerprint_environment::MediaPreferencesFingerprint::default();
        if self.media_preferences.pointer == preference_defaults.pointer {
            self.media_preferences.pointer = "coarse".to_owned();
        }
        if self.media_preferences.any_pointer == preference_defaults.any_pointer {
            self.media_preferences.any_pointer = "coarse".to_owned();
        }
        if self.media_preferences.hover == preference_defaults.hover {
            self.media_preferences.hover = "none".to_owned();
        }
        if self.media_preferences.any_hover == preference_defaults.any_hover {
            self.media_preferences.any_hover = "none".to_owned();
        }

        let plugin_defaults = crate::fingerprint_environment::PluginListFingerprint::default();
        if self.plugins == plugin_defaults {
            self.plugins.plugins.clear();
        }

        let audio_defaults = crate::fingerprint_surface::AudioFingerprint::default();
        if self.rendering.audio.base_latency == audio_defaults.base_latency {
            self.rendering.audio.base_latency = 0.002_666_666_666_666_666_6;
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 128
            || !self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._-,".contains(&byte))
        {
            return Err(
                "fingerprint id must contain at most 128 ASCII identifier characters".to_owned(),
            );
        }
        crate::browser_version::BrowserVersion::from_user_agent(&self.navigator.user_agent)?;
        self.locale.validate()?;
        self.navigator.validate()?;
        self.screen.validate()?;
        self.rendering.validate()?;
        self.storage.validate()?;
        self.speech.validate()?;
        self.fonts.validate()?;
        self.css.validate()?;
        self.document.validate()?;
        self.media.validate()?;
        self.permissions.validate()?;
        self.battery.validate()?;
        self.geolocation.validate()?;
        self.media_preferences.validate()?;
        self.plugins.validate()?;
        self.hardware_devices.validate()?;
        self.sensors.validate()?;
        self.timing.validate()?;
        self.xr.validate()?;
        self.memory.validate()?;
        self.performance.validate()
    }
}

fn android_version_from_user_agent(user_agent: &str) -> Option<&str> {
    let value = user_agent.split_once("Android ")?.1;
    value.split([';', ')']).next().map(str::trim)
}

fn android_model_from_user_agent(user_agent: &str) -> Option<String> {
    let value = user_agent.split_once("Android ")?.1;
    let model = value.split_once(';')?.1.split(')').next()?.trim();
    let model = model.split(" Build/").next().unwrap_or(model).trim();
    (!model.is_empty() && model != "K").then(|| model.to_owned())
}

fn normalize_three_component_version(value: &str) -> String {
    let mut parts = value.split('.').take(3).collect::<Vec<_>>();
    while parts.len() < 3 {
        parts.push("0");
    }
    parts.join(".")
}

impl NavigatorFingerprint {
    pub fn validate(&self) -> Result<(), String> {
        let bounded_strings = [
            &self.user_agent,
            &self.app_version,
            &self.app_code_name,
            &self.app_name,
            &self.platform,
            &self.product,
            &self.product_sub,
            &self.vendor,
            &self.vendor_sub,
            &self.language,
        ];
        if bounded_strings.iter().any(|value| value.len() > 1024) {
            return Err("navigator fingerprint contains an oversized string".to_owned());
        }
        if self.user_agent.is_empty()
            || self.app_version.is_empty()
            || self.language.is_empty()
            || self.languages.is_empty()
        {
            return Err("navigator fingerprint contains an empty required field".to_owned());
        }
        if self.languages.len() > 16
            || self.languages[0] != self.language
            || self
                .languages
                .iter()
                .any(|language| language.is_empty() || language.len() > 64)
        {
            return Err("navigator.languages must begin with navigator.language".to_owned());
        }
        if self.app_version
            != self
                .user_agent
                .strip_prefix("Mozilla/")
                .unwrap_or(&self.user_agent)
        {
            return Err(
                "navigator.app_version must equal user_agent without the Mozilla/ prefix"
                    .to_owned(),
            );
        }
        if self.max_touch_points > 256 {
            return Err("max_touch_points must not exceed 256".to_owned());
        }
        if self.user_activation_is_active && !self.user_activation_has_been_active {
            return Err("navigator.userActivation.isActive requires hasBeenActive".to_owned());
        }
        if let Some(value) = &self.do_not_track
            && value != "0"
            && value != "1"
            && value != "unspecified"
        {
            return Err("do_not_track must be 0, 1, unspecified, or absent".to_owned());
        }
        self.user_agent_data.validate()?;
        self.network.validate()
    }
}

impl UserAgentDataFingerprint {
    pub(crate) fn synchronize_default_browser_version(&mut self, user_agent: &str) {
        let defaults = Self::default();
        let Ok(version) = crate::browser_version::BrowserVersion::from_user_agent(user_agent)
        else {
            return;
        };
        let major = version.major().to_string();
        let full_version =
            crate::browser_version::BrowserVersion::full_version_from_user_agent(user_agent)
                .unwrap_or_else(|| format!("{major}.0.0.0"));
        for brand in &mut self.brands {
            if matches!(
                brand.brand.as_str(),
                "Chromium" | "Google Chrome" | "Microsoft Edge"
            ) && defaults.brands.iter().any(|default| {
                default.brand == brand.brand
                    && default.version == brand.version
                    && default.full_version == brand.full_version
            }) {
                brand.version.clone_from(&major);
                brand.full_version.clone_from(&full_version);
            }
        }
        if self.ua_full_version == defaults.ua_full_version {
            self.ua_full_version = full_version;
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.brands.is_empty() || self.brands.len() > 16 {
            return Err("userAgentData.brands must contain 1 to 16 entries".to_owned());
        }
        if self.brands.iter().any(|brand| {
            brand.brand.is_empty()
                || brand.brand.len() > 128
                || brand.version.is_empty()
                || brand.version.len() > 32
                || brand.full_version.is_empty()
                || brand.full_version.len() > 64
        }) {
            return Err("userAgentData contains an invalid brand entry".to_owned());
        }
        let strings = [
            &self.platform,
            &self.architecture,
            &self.bitness,
            &self.model,
            &self.platform_version,
            &self.ua_full_version,
        ];
        if strings.iter().any(|value| value.len() > 128)
            || self.form_factors.len() > 16
            || self
                .form_factors
                .iter()
                .any(|value| value.is_empty() || value.len() > 64)
        {
            return Err("userAgentData contains an oversized field".to_owned());
        }
        Ok(())
    }
}

impl NetworkFingerprint {
    fn validate(&self) -> Result<(), String> {
        if !matches!(self.effective_type.as_str(), "slow-2g" | "2g" | "3g" | "4g")
            || !self.downlink.is_finite()
            || self.downlink < 0.0
            || self.downlink > 10_000.0
            || !matches!(
                self.connection_type.as_str(),
                "bluetooth"
                    | "cellular"
                    | "ethernet"
                    | "none"
                    | "wifi"
                    | "wimax"
                    | "other"
                    | "unknown"
            )
            || self.downlink_max.is_nan()
            || self.downlink_max < 0.0
        {
            return Err("network fingerprint is outside Chromium bounds".to_owned());
        }
        Ok(())
    }
}

impl SpeechFingerprint {
    fn validate(&self) -> Result<(), String> {
        if self.voices.len() > 256
            || self.voices.iter().any(|voice| {
                voice.voice_uri.is_empty()
                    || voice.voice_uri.len() > 1024
                    || voice.name.is_empty()
                    || voice.name.len() > 1024
                    || voice.lang.is_empty()
                    || voice.lang.len() > 64
            })
        {
            return Err("speech fingerprint contains an invalid voice".to_owned());
        }
        Ok(())
    }
}

#[derive(Clone)]
struct FingerprintState(EdgeFingerprint);

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate, fingerprint: EdgeFingerprint) {
    isolate.set_slot(FingerprintState(fingerprint));
}

pub(crate) fn navigator<'a>(scope: &'a v8::PinScope<'_, '_>) -> &'a NavigatorFingerprint {
    scope
        .get_slot::<FingerprintState>()
        .map(|state| &state.0.navigator)
        .expect("Edge fingerprint state was not prepared")
}

pub(crate) fn browser_version(
    scope: &v8::PinScope<'_, '_>,
) -> crate::browser_version::BrowserVersion {
    crate::browser_version::BrowserVersion::from_user_agent(&navigator(scope).user_agent)
        .expect("validated browser version was not available")
}

pub(crate) fn edge<'a>(scope: &'a v8::PinScope<'_, '_>) -> &'a EdgeFingerprint {
    scope
        .get_slot::<FingerprintState>()
        .map(|state| &state.0)
        .expect("Edge fingerprint state was not prepared")
}

pub(crate) fn screen_for_isolate(
    isolate: &v8::OwnedIsolate,
) -> crate::fingerprint_surface::ScreenFingerprint {
    isolate
        .get_slot::<FingerprintState>()
        .map(|state| state.0.screen.clone())
        .expect("Edge fingerprint state was not prepared")
}

#[cfg(test)]
mod tests {
    use super::NavigatorFingerprint;

    #[test]
    fn accepts_user_configured_hardware_values_without_browser_bucket_limits() {
        for (hardware_concurrency, device_memory_gb) in
            [(0, 0.0), (10, 3.75), (4096, 32.0), (u32::MAX, 1024.5)]
        {
            let mut navigator = NavigatorFingerprint::default();
            navigator.hardware_concurrency = hardware_concurrency;
            navigator.device_memory_gb = device_memory_gb;
            assert_eq!(navigator.validate(), Ok(()));
        }
    }
}
