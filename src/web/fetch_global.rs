pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let function =
        crate::webidl::create_function(scope, "fetch", 1, v8::ConstructorBehavior::Throw, execute)?;
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "fetch")?;
    if global.define_own_property(
        scope,
        key.into(),
        function.into(),
        v8::PropertyAttribute::NONE,
    ) == Some(true)
    {
        Ok(())
    } else {
        Err("cannot define window.fetch".to_owned())
    }
}

pub(crate) fn execute<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'fetch' on 'Window': 1 argument required, but only 0 present.",
        );
        return;
    }
    let input = arguments.get(0);
    let request = v8::Local::<v8::Object>::try_from(input)
        .ok()
        .filter(|object| super::request::url(scope, *object).is_some());
    let request_snapshot = request.and_then(|object| super::request::fetch_snapshot(scope, object));
    let input_url = request_snapshot
        .as_ref()
        .map(|request| request.url.clone())
        .unwrap_or_else(|| crate::webidl::value_to_string(scope, input));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let url = match resolve_request_url(scope, &input_url) {
        Ok(url) => url,
        Err(_) => {
            // RequestInit is a Web IDL dictionary. Chromium converts it before
            // rejecting an invalid URL, so its observable getters must still
            // run even though no network request can be created.
            super::request::observe_request_init(scope, init);
            reject_type_error(
                scope,
                &format!("Failed to parse URL from {input_url}"),
                result,
            );
            return;
        }
    };
    let method = init
        .and_then(|init| {
            v8::String::new(scope, "method")
                .and_then(|key| init.get(scope, key.into()))
                .filter(|value| !value.is_undefined())
                .map(|value| crate::webidl::value_to_string(scope, value))
        })
        .or_else(|| {
            request_snapshot
                .as_ref()
                .map(|request| request.method.clone())
        })
        .unwrap_or_else(|| "GET".to_owned())
        .to_ascii_uppercase();
    let mut headers = init
        .and_then(|init| super::request::init_headers(scope, init))
        .or_else(|| {
            request_snapshot
                .as_ref()
                .map(|request| request.headers.clone())
        })
        .unwrap_or_default();
    let body_value = init.and_then(|init| {
        v8::String::new(scope, "body")
            .and_then(|key| init.get(scope, key.into()))
            .filter(|value| !value.is_undefined())
    });
    let (body, content_type) = if matches!(method.as_str(), "GET" | "HEAD") {
        (Vec::new(), None)
    } else if let Some(body) = body_value {
        crate::network_capture::body_bytes(scope, body)
    } else {
        (
            request_snapshot
                .as_ref()
                .map(|request| request.bytes.clone())
                .unwrap_or_default(),
            None,
        )
    };
    crate::network_capture::append_content_type_if_missing(&mut headers, content_type);
    crate::network_capture::record(
        scope,
        crate::NetworkRequestSource::Fetch,
        method.clone(),
        url.clone(),
        headers,
        body,
    );
    if !super::worker_global_scope::is_entered_service_realm(scope)
        && let Some(realm_id) = super::service_worker_container::active_realm_id(scope)
        && let Some(response) =
            super::worker_global_scope::dispatch_service_fetch(scope, realm_id, &url)
    {
        match response {
            Ok(response) => {
                let response = v8::Local::new(scope, &response);
                if let Ok(promise) = super::writable_stream::resolved_promise(scope, response) {
                    result.set(promise.into());
                }
            }
            Err(message) => reject_type_error(scope, &message, result),
        }
        return;
    }
    if url.starts_with("data:") {
        match decode_data_url(&url) {
            Ok((content_type, bytes)) => {
                match super::response::create_fetch_response(
                    scope,
                    url,
                    200,
                    "OK".to_owned(),
                    vec![("content-type".to_owned(), content_type)],
                    bytes,
                ) {
                    Ok(response) => {
                        if let Ok(promise) =
                            super::writable_stream::resolved_promise(scope, response.into())
                        {
                            result.set(promise.into());
                        }
                    }
                    Err(message) => reject_type_error(scope, &message, result),
                }
            }
            Err(message) => reject_type_error(scope, &message, result),
        }
        return;
    }
    let start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    if let Some(entry) = crate::network_replay::lookup(scope, &method, &url) {
        super::performance_resource_timing::record_network_replay(
            scope, &entry, "fetch", start_time,
        );
        match super::response::create_fetch_response(
            scope,
            entry.url,
            entry.status,
            entry.status_text,
            entry.headers,
            entry.body,
        ) {
            Ok(response) => {
                if let Ok(promise) =
                    super::writable_stream::resolved_promise(scope, response.into())
                {
                    result.set(promise.into());
                }
            }
            Err(message) => reject_type_error(scope, &message, result),
        }
        return;
    }
    let message =
        format!("Fetch for the '{url}' URL scheme is unavailable in this offline runtime");
    reject_type_error(scope, &message, result);
}

pub(crate) fn resolve_request_url(
    scope: &mut v8::PinScope<'_, '_>,
    input: &str,
) -> Result<String, String> {
    if let Ok(url) = url::Url::parse(input) {
        return Ok(url.to_string());
    }
    let document_base = super::document_global::value(scope)
        .map(|document| super::document::base_url(scope, document));
    let global = scope.get_current_context().global(scope);
    let base = document_base
        .or_else(|| {
            v8::String::new(scope, "location")
                .and_then(|key| global.get(scope, key.into()))
                .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
                .and_then(|location| {
                    v8::String::new(scope, "href")
                        .and_then(|key| location.get(scope, key.into()))
                        .map(|value| crate::webidl::value_to_string(scope, value))
                })
        })
        .unwrap_or_else(|| crate::page_init::base_url(scope));
    let base = url::Url::parse(&base)
        .ok()
        .filter(|base| matches!(base.scheme(), "http" | "https"))
        .or_else(|| url::Url::parse(&crate::page_init::base_url(scope)).ok())
        .ok_or_else(|| format!("Failed to resolve a base URL for {input}"))?;
    base.join(input)
        .map(|url| url.to_string())
        .map_err(|_| format!("Failed to parse URL from {input}"))
}

pub(crate) fn decode_data_url(url: &str) -> Result<(String, Vec<u8>), String> {
    let payload = url
        .strip_prefix("data:")
        .ok_or_else(|| "Invalid data URL".to_owned())?;
    let (metadata, encoded) = payload
        .split_once(',')
        .ok_or_else(|| "Invalid data URL".to_owned())?;
    let base64 = metadata
        .rsplit(';')
        .next()
        .is_some_and(|value| value.eq_ignore_ascii_case("base64"));
    let media_type = if metadata.is_empty() || metadata.starts_with(';') {
        "text/plain;charset=US-ASCII".to_owned()
    } else if base64 {
        metadata
            .strip_suffix(";base64")
            .or_else(|| metadata.strip_suffix(";BASE64"))
            .unwrap_or(metadata)
            .to_owned()
    } else {
        metadata.to_owned()
    };
    let decoded = percent_decode(encoded)?;
    let bytes = if base64 {
        decode_base64(&decoded)?
    } else {
        decoded.into_bytes()
    };
    Ok((media_type, bytes))
}

fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("Invalid percent escape in data URL".to_owned());
            }
            let high = hex(bytes[index + 1])
                .ok_or_else(|| "Invalid percent escape in data URL".to_owned())?;
            let low = hex(bytes[index + 2])
                .ok_or_else(|| "Invalid percent escape in data URL".to_owned())?;
            decoded.push(high << 4 | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    Ok(String::from_utf8_lossy(&decoded).into_owned())
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    let mut sextets = Vec::new();
    let mut padding = 0;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            padding += 1;
            sextets.push(0);
        } else {
            let sextet = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => return Err("Invalid base64 data URL".to_owned()),
            };
            sextets.push(sextet);
        }
    }
    if sextets.is_empty() {
        return Ok(Vec::new());
    }
    if sextets.len() % 4 != 0 || padding > 2 {
        return Err("Invalid base64 data URL".to_owned());
    }
    let mut output = Vec::with_capacity(sextets.len() / 4 * 3);
    for chunk in sextets.chunks_exact(4) {
        output.push(chunk[0] << 2 | chunk[1] >> 4);
        output.push(chunk[1] << 4 | chunk[2] >> 2);
        output.push(chunk[2] << 6 | chunk[3]);
    }
    output.truncate(output.len().saturating_sub(padding));
    Ok(output)
}

fn reject_type_error(
    scope: &mut v8::PinScope<'_, '_>,
    message: &str,
    mut result: v8::ReturnValue<'_>,
) {
    let message = v8::String::new(scope, message).expect("short fetch error");
    let exception = v8::Exception::type_error(scope, message);
    if let Ok(promise) = super::writable_stream::rejected_promise(scope, exception) {
        result.set(promise.into());
    }
}
