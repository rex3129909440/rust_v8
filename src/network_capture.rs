#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum NetworkRequestSource {
    XmlHttpRequest = 1,
    Fetch = 2,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct CapturedNetworkRequest {
    pub sequence: u64,
    pub source: NetworkRequestSource,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Default)]
struct NetworkCaptureState {
    next_sequence: u64,
    entries: Vec<CapturedNetworkRequest>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NetworkCaptureState::default());
}

pub(crate) fn record(
    scope: &mut v8::PinScope<'_, '_>,
    source: NetworkRequestSource,
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) {
    let Some(state) = scope.get_slot_mut::<NetworkCaptureState>() else {
        return;
    };
    state.next_sequence = state.next_sequence.saturating_add(1);
    state.entries.push(CapturedNetworkRequest {
        sequence: state.next_sequence,
        source,
        method,
        url,
        headers,
        body,
    });
}

pub(crate) fn entries(isolate: &v8::OwnedIsolate) -> Vec<CapturedNetworkRequest> {
    isolate
        .get_slot::<NetworkCaptureState>()
        .map(|state| state.entries.clone())
        .unwrap_or_default()
}

pub(crate) fn clear(isolate: &mut v8::OwnedIsolate) {
    if let Some(state) = isolate.get_slot_mut::<NetworkCaptureState>() {
        state.entries.clear();
        state.next_sequence = 0;
    }
}

pub(crate) fn body_bytes(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> (Vec<u8>, Option<String>) {
    if value.is_null_or_undefined() {
        return (Vec::new(), None);
    }
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let backing = buffer.get_backing_store();
        let bytes = backing
            .data()
            .map(|data| {
                // SAFETY: the backing store remains alive for the duration of
                // this copy and exposes exactly `byte_length` readable bytes.
                unsafe {
                    std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), backing.byte_length())
                }
                .to_vec()
            })
            .unwrap_or_default();
        return (bytes, None);
    }
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut bytes = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut bytes);
        bytes.truncate(copied);
        return (bytes, None);
    }
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if let Some((bytes, media_type)) = crate::web::blob::byte_snapshot(scope, object) {
            let media_type = (!media_type.is_empty()).then_some(media_type);
            return (bytes, media_type);
        }
        if let Some(value) = crate::web::url_search_params::serialized_snapshot(scope, object) {
            return (
                value.into_bytes(),
                Some("application/x-www-form-urlencoded;charset=UTF-8".to_owned()),
            );
        }
    }
    (
        crate::webidl::value_to_string(scope, value).into_bytes(),
        Some("text/plain;charset=UTF-8".to_owned()),
    )
}

pub(crate) fn append_content_type_if_missing(
    headers: &mut Vec<(String, String)>,
    content_type: Option<String>,
) {
    let Some(content_type) = content_type else {
        return;
    };
    if !headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
    {
        headers.push(("Content-Type".to_owned(), content_type));
    }
}

pub(crate) fn encode_binary(entries: &[CapturedNetworkRequest]) -> Result<Vec<u8>, String> {
    const MAGIC: &[u8; 4] = b"ESNR";
    const VERSION: u16 = 1;

    let count = u32::try_from(entries.len())
        .map_err(|_| "too many captured network requests to export".to_owned())?;
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&VERSION.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        let method_len = u32::try_from(entry.method.len())
            .map_err(|_| "captured request method is too large".to_owned())?;
        let url_len = u32::try_from(entry.url.len())
            .map_err(|_| "captured request URL is too large".to_owned())?;
        let header_count = u32::try_from(entry.headers.len())
            .map_err(|_| "captured request has too many headers".to_owned())?;
        let body_len = u64::try_from(entry.body.len())
            .map_err(|_| "captured request body is too large".to_owned())?;

        output.extend_from_slice(&entry.sequence.to_le_bytes());
        output.push(entry.source as u8);
        output.extend_from_slice(&[0_u8; 3]);
        output.extend_from_slice(&method_len.to_le_bytes());
        output.extend_from_slice(&url_len.to_le_bytes());
        output.extend_from_slice(&header_count.to_le_bytes());
        output.extend_from_slice(&body_len.to_le_bytes());
        output.extend_from_slice(entry.method.as_bytes());
        output.extend_from_slice(entry.url.as_bytes());
        for (name, value) in &entry.headers {
            let name_len = u32::try_from(name.len())
                .map_err(|_| "captured request header name is too large".to_owned())?;
            let value_len = u32::try_from(value.len())
                .map_err(|_| "captured request header value is too large".to_owned())?;
            output.extend_from_slice(&name_len.to_le_bytes());
            output.extend_from_slice(&value_len.to_le_bytes());
            output.extend_from_slice(name.as_bytes());
            output.extend_from_slice(value.as_bytes());
        }
        output.extend_from_slice(&entry.body);
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{NetworkRequestSource, encode_binary};

    #[test]
    fn xhr_and_fetch_are_captured_without_native_trace() {
        let mut runtime = crate::EdgeRuntime::new().expect("runtime");
        runtime
            .evaluate(
                r#"
                const xhr = new XMLHttpRequest();
                xhr.open("POST", "https://capture.example/xhr");
                xhr.setRequestHeader("X-Test", "one");
                xhr.setRequestHeader("x-test", "two");
                xhr.send(new Uint8Array([0, 1, 2, 255]));

                fetch("https://capture.example/fetch", {
                  method: "PUT",
                  headers: { "X-Fetch": "yes" },
                  body: "fetch-body"
                }).catch(() => {});
                "#,
            )
            .expect("evaluation");

        assert!(
            runtime.native_trace().is_empty(),
            "request capture must not enable API tracing"
        );
        let requests = runtime.network_requests();
        assert_eq!(requests.len(), 2);

        let xhr = &requests[0];
        assert_eq!(xhr.source, NetworkRequestSource::XmlHttpRequest);
        assert_eq!(xhr.method, "POST");
        assert_eq!(xhr.url, "https://capture.example/xhr");
        assert_eq!(xhr.body, [0, 1, 2, 255]);
        assert!(
            xhr.headers.iter().any(|(name, value)| {
                name.eq_ignore_ascii_case("x-test") && value == "one, two"
            })
        );

        let fetch = &requests[1];
        assert_eq!(fetch.source, NetworkRequestSource::Fetch);
        assert_eq!(fetch.method, "PUT");
        assert_eq!(fetch.url, "https://capture.example/fetch");
        assert_eq!(fetch.body, b"fetch-body");
        assert!(
            fetch
                .headers
                .iter()
                .any(|(name, value)| { name.eq_ignore_ascii_case("x-fetch") && value == "yes" })
        );
        assert!(fetch.headers.iter().any(|(name, value)| {
            name.eq_ignore_ascii_case("content-type")
                && value.eq_ignore_ascii_case("text/plain;charset=UTF-8")
        }));

        let binary = encode_binary(&requests).expect("binary export");
        assert_eq!(&binary[..4], b"ESNR");
        assert_eq!(u16::from_le_bytes([binary[4], binary[5]]), 1);
        assert_eq!(
            u32::from_le_bytes([binary[8], binary[9], binary[10], binary[11]]),
            2
        );

        runtime.clear_network_requests();
        assert!(runtime.network_requests().is_empty());
    }
}
