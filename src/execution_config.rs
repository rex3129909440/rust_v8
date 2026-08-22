use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct DeterministicExecution {
    pub clock_epoch_ms: Option<i64>,
    pub clock_step_ms: u64,
    pub random_seed: Option<u64>,
    pub max_task_turns: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SandboxLimits {
    pub timeout: Option<Duration>,
    pub max_heap_bytes: Option<usize>,
    pub max_young_generation_bytes: Option<usize>,
    pub max_code_range_bytes: Option<usize>,
    pub max_resident_bytes: Option<usize>,
    pub max_source_bytes: Option<usize>,
    pub max_output_bytes: Option<usize>,
}

impl Default for DeterministicExecution {
    fn default() -> Self {
        Self {
            clock_epoch_ms: None,
            clock_step_ms: 1,
            random_seed: None,
            max_task_turns: 1_024,
        }
    }
}

impl SandboxLimits {
    pub fn isolated_default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            max_heap_bytes: Some(512 * 1024 * 1024),
            max_young_generation_bytes: None,
            max_code_range_bytes: None,
            max_resident_bytes: Some(768 * 1024 * 1024),
            max_source_bytes: Some(1024 * 1024),
            max_output_bytes: Some(1024 * 1024),
        }
    }

    pub(crate) fn apply_isolated_defaults(&mut self) {
        let defaults = Self::isolated_default();
        self.timeout = self.timeout.or(defaults.timeout);
        self.max_heap_bytes = self.max_heap_bytes.or(defaults.max_heap_bytes);
        // V8-specific constraints remain opt-in. V8 derives its normal
        // generation/code defaults when these fields are absent.
        self.max_resident_bytes = self.max_resident_bytes.or(defaults.max_resident_bytes);
        self.max_source_bytes = self.max_source_bytes.or(defaults.max_source_bytes);
        self.max_output_bytes = self.max_output_bytes.or(defaults.max_output_bytes);
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.timeout.is_some_and(|value| {
            value < Duration::from_millis(10) || value > Duration::from_secs(300)
        }) {
            return Err("timeout must be between 10 milliseconds and 300 seconds".to_owned());
        }
        if self
            .max_heap_bytes
            .is_some_and(|value| !(16 * 1024 * 1024..=8 * 1024 * 1024 * 1024).contains(&value))
        {
            return Err("max_heap_bytes must be between 16 MiB and 8 GiB".to_owned());
        }
        if self.max_young_generation_bytes.is_some_and(|value| {
            !(1024 * 1024..=512 * 1024 * 1024).contains(&value)
                || self
                    .max_heap_bytes
                    .is_some_and(|heap| value.saturating_add(2 * 1024 * 1024) >= heap)
        }) {
            return Err(
                "max_young_generation_bytes must be between 1 MiB and 512 MiB and leave at least 2 MiB below max_heap_bytes"
                    .to_owned(),
            );
        }
        if self
            .max_code_range_bytes
            .is_some_and(|value| !(8 * 1024 * 1024..=2 * 1024 * 1024 * 1024).contains(&value))
        {
            return Err("max_code_range_bytes must be between 8 MiB and 2 GiB".to_owned());
        }
        if self.max_resident_bytes.is_some_and(|value| {
            !(64 * 1024 * 1024..=16 * 1024 * 1024 * 1024usize).contains(&value)
        }) {
            return Err("max_resident_bytes must be between 64 MiB and 16 GiB".to_owned());
        }
        if self
            .max_output_bytes
            .is_some_and(|value| !(1024..=64 * 1024 * 1024).contains(&value))
        {
            return Err("max_output_bytes must be between 1 KiB and 64 MiB".to_owned());
        }
        if self
            .max_source_bytes
            .is_some_and(|value| !(1024..=64 * 1024 * 1024).contains(&value))
        {
            return Err("max_source_bytes must be between 1 KiB and 64 MiB".to_owned());
        }
        Ok(())
    }
}

impl DeterministicExecution {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.clock_step_ms > 86_400_000 {
            return Err("clock_step_ms must not exceed one day".to_owned());
        }
        if !(1..=65_536).contains(&self.max_task_turns) {
            return Err("max_task_turns must be between 1 and 65536".to_owned());
        }
        Ok(())
    }
}
