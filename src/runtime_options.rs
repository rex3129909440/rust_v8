use crate::{
    DeterministicExecution, EdgeFingerprint, IframeHook, NetworkReplayEntry, PageInit,
    SandboxLimits,
};

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct EdgeRuntimeOptions {
    pub fingerprint: EdgeFingerprint,
    pub deterministic: DeterministicExecution,
    pub network_replay: Vec<NetworkReplayEntry>,
    pub limits: SandboxLimits,
    pub page: Option<PageInit>,
    pub cross_origin_isolated: bool,
    pub iframe_hooks: Vec<IframeHook>,
}

impl EdgeRuntimeOptions {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.fingerprint.validate()?;
        self.deterministic.validate()?;
        self.limits.validate()?;
        if let Some(page) = &self.page {
            page.validate()?;
        }
        crate::iframe_hook::validate_hooks(&self.iframe_hooks)?;
        if self.network_replay.len() > 1_024 {
            return Err("network replay contains more than 1024 entries".to_owned());
        }
        for entry in &self.network_replay {
            entry.validate()?;
        }
        Ok(())
    }
}
