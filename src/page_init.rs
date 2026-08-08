pub const DEFAULT_PAGE_URL: &str = "https://sandbox.test/";
const MAX_HTML_BYTES: usize = 4 * 1024 * 1024;
const MAX_URL_BYTES: usize = 16 * 1024;
const MAX_REFERRER_BYTES: usize = 16 * 1024;
const MAX_CONTENT_TYPE_BYTES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PageInit {
    pub url: String,
    pub html: String,
    pub referrer: String,
    pub content_type: String,
}

impl Default for PageInit {
    fn default() -> Self {
        Self {
            url: DEFAULT_PAGE_URL.to_owned(),
            html: String::new(),
            referrer: String::new(),
            content_type: "text/html".to_owned(),
        }
    }
}

impl PageInit {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.url.len() > MAX_URL_BYTES {
            return Err("page URL exceeds 16384 bytes".to_owned());
        }
        let url = parse_https_url(&self.url, "page URL")?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err("page URL must not contain credentials".to_owned());
        }
        if self.html.len() > MAX_HTML_BYTES {
            return Err("page HTML exceeds 4194304 bytes".to_owned());
        }
        if self.referrer.len() > MAX_REFERRER_BYTES {
            return Err("page referrer exceeds 16384 bytes".to_owned());
        }
        if !self.referrer.is_empty() {
            parse_http_url(&self.referrer, "page referrer")?;
        }
        if self.content_type.is_empty()
            || self.content_type.len() > MAX_CONTENT_TYPE_BYTES
            || self.content_type.contains(['\r', '\n', '\0'])
        {
            return Err("page content type is invalid".to_owned());
        }
        let essence = content_type_essence(&self.content_type);
        if !matches!(
            essence,
            "text/html"
                | "application/xhtml+xml"
                | "application/xml"
                | "text/xml"
                | "image/svg+xml"
        ) {
            return Err("page content type is not supported".to_owned());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PageEnvironment {
    init: PageInit,
    url: url::Url,
    base_url: url::Url,
}

pub(crate) fn prepare(
    isolate: &mut v8::OwnedIsolate,
    init: Option<PageInit>,
) -> Result<(), String> {
    let init = init.unwrap_or_default();
    init.validate()?;
    let url = parse_https_url(&init.url, "page URL")?;
    isolate.set_slot(PageEnvironment {
        init,
        base_url: url.clone(),
        url,
    });
    Ok(())
}

pub(crate) fn url(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.url.as_str().to_owned())
        .unwrap_or_else(|| DEFAULT_PAGE_URL.to_owned())
}

pub(crate) fn html(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.init.html.clone())
        .unwrap_or_default()
}

pub(crate) fn referrer(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.init.referrer.clone())
        .unwrap_or_default()
}

pub(crate) fn content_type(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| content_type_essence(&environment.init.content_type).to_owned())
        .unwrap_or_else(|| "text/html".to_owned())
}

pub(crate) fn origin(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.url.origin().ascii_serialization())
        .unwrap_or_else(|| "https://sandbox.test".to_owned())
}

pub(crate) fn host(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .and_then(|environment| environment.url.host_str())
        .unwrap_or("sandbox.test")
        .to_owned()
}

pub(crate) fn path(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.url.path().to_owned())
        .unwrap_or_else(|| "/".to_owned())
}

pub(crate) fn base_url(scope: &v8::PinScope<'_, '_>) -> String {
    environment(scope)
        .map(|environment| environment.base_url.as_str().to_owned())
        .unwrap_or_else(|| DEFAULT_PAGE_URL.to_owned())
}

pub(crate) fn update_base_url(scope: &mut v8::PinScope<'_, '_>, candidate: Option<&str>) {
    let Some(environment) = scope.get_slot_mut::<PageEnvironment>() else {
        return;
    };
    environment.base_url = candidate
        .and_then(|candidate| environment.url.join(candidate).ok())
        .unwrap_or_else(|| environment.url.clone());
}

pub(crate) fn navigate(scope: &mut v8::PinScope<'_, '_>, value: &str) {
    let Ok(url) = parse_https_url(value, "page URL") else {
        return;
    };
    let Some(environment) = scope.get_slot_mut::<PageEnvironment>() else {
        return;
    };
    environment.url = url.clone();
    environment.base_url = url;
}

fn environment<'a>(scope: &'a v8::PinScope<'_, '_>) -> Option<&'a PageEnvironment> {
    scope.get_slot::<PageEnvironment>()
}

fn parse_https_url(value: &str, name: &str) -> Result<url::Url, String> {
    let url = parse_http_url(value, name)?;
    if url.scheme() != "https" {
        return Err(format!("{name} must use HTTPS"));
    }
    Ok(url)
}

fn parse_http_url(value: &str, name: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(value).map_err(|_| format!("{name} is invalid"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(format!("{name} must be an absolute HTTP(S) URL"));
    }
    Ok(url)
}

fn content_type_essence(value: &str) -> &str {
    value
        .split_once(';')
        .map_or(value, |(essence, _)| essence)
        .trim()
}
