#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct NetworkReplayEntry {
    pub url: String,
    pub method: String,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NetworkReplayState {
    entries: Vec<NetworkReplayEntry>,
}

impl NetworkReplayEntry {
    pub fn get(url: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_owned(),
            status: 200,
            status_text: "OK".to_owned(),
            headers: Vec::new(),
            body: body.into(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let parsed =
            url::Url::parse(&self.url).map_err(|_| "network replay URL is invalid".to_owned())?;
        if !matches!(parsed.scheme(), "http" | "https")
            || self.url.len() > 4096
            || self.method.is_empty()
            || self.method.len() > 32
            || !self
                .method
                .bytes()
                .all(|byte| byte.is_ascii_alphabetic() || byte == b'-')
            || !(100..=599).contains(&self.status)
            || self.status_text.len() > 256
            || self.headers.len() > 256
            || self.body.len() > 16 * 1024 * 1024
            || self.headers.iter().any(|(name, value)| {
                name.is_empty()
                    || name.len() > 256
                    || value.len() > 16 * 1024
                    || name.contains(['\r', '\n', ':'])
                    || value.contains(['\r', '\n'])
            })
        {
            return Err("network replay entry is outside supported bounds".to_owned());
        }
        Ok(())
    }
}

impl NetworkReplayState {
    pub(crate) fn new(entries: Vec<NetworkReplayEntry>) -> Self {
        Self { entries }
    }

    pub(crate) fn lookup(&self, method: &str, url: &str) -> Option<NetworkReplayEntry> {
        self.entries
            .iter()
            .find(|entry| entry.method.eq_ignore_ascii_case(method) && entry.url == url)
            .cloned()
    }
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate, entries: Vec<NetworkReplayEntry>) {
    isolate.set_slot(NetworkReplayState::new(entries));
}

pub(crate) fn lookup(
    scope: &v8::PinScope<'_, '_>,
    method: &str,
    url: &str,
) -> Option<NetworkReplayEntry> {
    scope.get_slot::<NetworkReplayState>()?.lookup(method, url)
}
