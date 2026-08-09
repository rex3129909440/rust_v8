/// Typed profile data used to seed the root realm's Performance Timeline.
///
/// An absent `entries` value preserves live page/network-derived entries.  A
/// present value, including an empty vector, replaces the automatically
/// generated initial timeline with the supplied ordered records.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PerformanceFingerprint {
    #[serde(default)]
    pub entries: Option<Vec<PerformanceEntryFingerprint>>,
    /// Content-Encoding used to model source passed through
    /// `evaluate(..., source_url=...)`. The source itself is decoded UTF-8;
    /// Rust encodes those bytes to derive Resource Timing byte counts.
    #[serde(default = "default_evaluated_script_content_encoding")]
    pub evaluated_script_content_encoding: String,
}

fn default_evaluated_script_content_encoding() -> String {
    "zstd".to_owned()
}

impl Default for PerformanceFingerprint {
    fn default() -> Self {
        Self {
            entries: None,
            evaluated_script_content_encoding: default_evaluated_script_content_encoding(),
        }
    }
}

/// One typed Performance Timeline record.
///
/// Resource fields are used by `resource` and `navigation` records;
/// navigation and paint fields are used only by their corresponding subtype.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PerformanceEntryFingerprint {
    pub name: String,
    pub entry_type: String,
    pub start_time: f64,
    pub duration: f64,

    pub initiator_type: String,
    pub delivery_type: String,
    pub next_hop_protocol: String,
    pub render_blocking_status: String,
    pub content_type: String,
    pub content_encoding: String,
    pub worker_start: f64,
    pub worker_router_evaluation_start: f64,
    pub worker_cache_lookup_start: f64,
    pub worker_matched_source_type: String,
    pub worker_final_source_type: String,
    pub redirect_start: f64,
    pub redirect_end: f64,
    pub fetch_start: f64,
    pub domain_lookup_start: f64,
    pub domain_lookup_end: f64,
    pub connect_start: f64,
    pub secure_connection_start: f64,
    pub connect_end: f64,
    pub request_start: f64,
    pub response_start: f64,
    pub first_interim_response_start: f64,
    pub final_response_headers_start: f64,
    pub response_end: f64,
    pub transfer_size: Option<u64>,
    pub encoded_body_size: Option<u64>,
    pub decoded_body_size: Option<u64>,
    pub response_status: Option<u16>,

    pub unload_event_start: f64,
    pub unload_event_end: f64,
    pub dom_interactive: f64,
    pub dom_content_loaded_event_start: f64,
    pub dom_content_loaded_event_end: f64,
    pub dom_complete: f64,
    pub load_event_start: f64,
    pub load_event_end: f64,
    pub navigation_type: String,
    pub redirect_count: u32,
    pub critical_ch_restart: f64,
    pub activation_start: f64,

    pub paint_time: f64,
    pub presentation_time: f64,
}

impl Default for PerformanceEntryFingerprint {
    fn default() -> Self {
        Self {
            name: String::new(),
            entry_type: "resource".to_owned(),
            start_time: 0.0,
            duration: 0.0,
            initiator_type: String::new(),
            delivery_type: String::new(),
            next_hop_protocol: String::new(),
            render_blocking_status: "non-blocking".to_owned(),
            content_type: String::new(),
            content_encoding: String::new(),
            worker_start: 0.0,
            worker_router_evaluation_start: 0.0,
            worker_cache_lookup_start: 0.0,
            worker_matched_source_type: String::new(),
            worker_final_source_type: String::new(),
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: 0.0,
            domain_lookup_start: 0.0,
            domain_lookup_end: 0.0,
            connect_start: 0.0,
            secure_connection_start: 0.0,
            connect_end: 0.0,
            request_start: 0.0,
            response_start: 0.0,
            first_interim_response_start: 0.0,
            final_response_headers_start: 0.0,
            response_end: 0.0,
            transfer_size: None,
            encoded_body_size: None,
            decoded_body_size: None,
            response_status: None,
            unload_event_start: 0.0,
            unload_event_end: 0.0,
            dom_interactive: 0.0,
            dom_content_loaded_event_start: 0.0,
            dom_content_loaded_event_end: 0.0,
            dom_complete: 0.0,
            load_event_start: 0.0,
            load_event_end: 0.0,
            navigation_type: "navigate".to_owned(),
            redirect_count: 0,
            critical_ch_restart: 0.0,
            activation_start: 0.0,
            paint_time: 0.0,
            presentation_time: 0.0,
        }
    }
}

impl PerformanceFingerprint {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !matches!(
            self.evaluated_script_content_encoding.as_str(),
            "" | "gzip" | "deflate" | "br" | "zstd"
        ) {
            return Err(
                "evaluated script content encoding must be gzip, deflate, br, zstd, or empty"
                    .to_owned(),
            );
        }
        let Some(entries) = &self.entries else {
            return Ok(());
        };
        if entries.len() > 1_024 {
            return Err("performance profile contains more than 1024 entries".to_owned());
        }
        for entry in entries {
            entry.validate()?;
        }
        Ok(())
    }
}

impl PerformanceEntryFingerprint {
    fn validate(&self) -> Result<(), String> {
        if self.name.len() > 16 * 1024
            || !matches!(
                self.entry_type.as_str(),
                "navigation" | "resource" | "visibility-state" | "paint"
            )
        {
            return Err("performance entry contains an invalid name or type".to_owned());
        }
        let strings = [
            &self.initiator_type,
            &self.delivery_type,
            &self.next_hop_protocol,
            &self.render_blocking_status,
            &self.content_type,
            &self.content_encoding,
            &self.worker_matched_source_type,
            &self.worker_final_source_type,
            &self.navigation_type,
        ];
        if strings.iter().any(|value| value.len() > 1024) {
            return Err("performance entry contains an oversized string".to_owned());
        }
        let numbers = [
            self.start_time,
            self.duration,
            self.worker_start,
            self.worker_router_evaluation_start,
            self.worker_cache_lookup_start,
            self.redirect_start,
            self.redirect_end,
            self.fetch_start,
            self.domain_lookup_start,
            self.domain_lookup_end,
            self.connect_start,
            self.secure_connection_start,
            self.connect_end,
            self.request_start,
            self.response_start,
            self.first_interim_response_start,
            self.final_response_headers_start,
            self.response_end,
            self.unload_event_start,
            self.unload_event_end,
            self.dom_interactive,
            self.dom_content_loaded_event_start,
            self.dom_content_loaded_event_end,
            self.dom_complete,
            self.load_event_start,
            self.load_event_end,
            self.critical_ch_restart,
            self.activation_start,
            self.paint_time,
            self.presentation_time,
        ];
        if numbers
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(
                "performance entry timing values must be finite and non-negative".to_owned(),
            );
        }
        if self.entry_type == "visibility-state"
            && !matches!(self.name.as_str(), "visible" | "hidden")
        {
            return Err("visibility-state entry name must be visible or hidden".to_owned());
        }
        if !self.content_encoding.is_empty()
            && !matches!(
                self.content_encoding.as_str(),
                "gzip" | "deflate" | "br" | "zstd"
            )
        {
            return Err(
                "performance content_encoding must be gzip, deflate, br, zstd, or empty".to_owned(),
            );
        }
        if matches!(self.entry_type.as_str(), "navigation" | "resource")
            && !self.content_encoding.is_empty()
            && (self.encoded_body_size.is_none() || self.decoded_body_size.is_none())
        {
            return Err(
                "compressed performance entries require encoded_body_size and decoded_body_size"
                    .to_owned(),
            );
        }
        Ok(())
    }

    pub(crate) fn resolved_body_sizes(&self) -> (u64, u64, u64) {
        let encoded = self
            .encoded_body_size
            .or_else(|| {
                self.content_encoding
                    .is_empty()
                    .then_some(self.decoded_body_size)
                    .flatten()
            })
            .unwrap_or_default();
        let decoded = self
            .decoded_body_size
            .or_else(|| self.content_encoding.is_empty().then_some(encoded))
            .unwrap_or_default();
        let transfer = self
            .transfer_size
            .unwrap_or_else(|| encoded.saturating_add(300));
        (transfer, encoded, decoded)
    }
}
