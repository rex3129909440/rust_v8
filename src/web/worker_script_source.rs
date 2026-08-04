pub(crate) struct WorkerScript {
    pub(crate) url: String,
    pub(crate) source: String,
}

pub(crate) fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    input: &str,
    base: Option<&str>,
) -> Result<String, String> {
    if input.starts_with("data:") || input.starts_with("blob:") {
        return Ok(input.to_owned());
    }
    if let Ok(url) = url::Url::parse(input) {
        return Ok(url.to_string());
    }
    let base = base
        .map(str::to_owned)
        .unwrap_or_else(|| parent_location(scope));
    url::Url::parse(&base)
        .and_then(|base| base.join(input))
        .map(|url| url.to_string())
        .map_err(|_| format!("Failed to resolve worker script URL '{input}'"))
}

pub(crate) fn load(
    scope: &mut v8::PinScope<'_, '_>,
    input: &str,
    base: Option<&str>,
) -> Result<WorkerScript, String> {
    load_with_initiator(scope, input, base, "other")
}

pub(crate) fn load_with_initiator(
    scope: &mut v8::PinScope<'_, '_>,
    input: &str,
    base: Option<&str>,
    initiator_type: &str,
) -> Result<WorkerScript, String> {
    let resolved = resolve(scope, input, base)?;
    if resolved.starts_with("data:") {
        return Ok(WorkerScript {
            url: resolved.clone(),
            source: decode_data_url(&resolved)?,
        });
    }
    if resolved.starts_with("blob:") {
        let Some((bytes, _)) = super::url::object_url_snapshot(scope, &resolved) else {
            return Err(format!("Worker script URL '{resolved}' has been revoked"));
        };
        return Ok(WorkerScript {
            url: resolved,
            source: String::from_utf8(bytes)
                .map_err(|_| "Worker script is not valid UTF-8".to_owned())?,
        });
    }
    let start_time = super::performance::now_for_current_realm(scope).unwrap_or(0.0);
    if let Some(entry) = crate::network_replay::lookup(scope, "GET", &resolved) {
        super::performance_resource_timing::record_network_replay(
            scope,
            &entry,
            initiator_type,
            start_time,
        );
        if !(200..=299).contains(&entry.status) {
            return Err(format!(
                "Worker script URL '{}' returned HTTP status {}",
                entry.url, entry.status
            ));
        }
        return Ok(WorkerScript {
            url: entry.url,
            source: String::from_utf8(entry.body)
                .map_err(|_| "Worker script is not valid UTF-8".to_owned())?,
        });
    }
    Err(format!(
        "The offline Worker cannot load network script URL '{resolved}'"
    ))
}

fn parent_location(scope: &mut v8::PinScope<'_, '_>) -> String {
    let global = scope.get_current_context().global(scope);
    let location = v8::String::new(scope, "location")
        .and_then(|key| global.get(scope, key.into()))
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok());
    location
        .and_then(|location| {
            v8::String::new(scope, "href")
                .and_then(|key| location.get(scope, key.into()))
                .map(|value| crate::webidl::value_to_string(scope, value))
        })
        .unwrap_or_else(|| "https://sandbox.test/".to_owned())
}

fn decode_data_url(url: &str) -> Result<String, String> {
    let rest = url
        .strip_prefix("data:")
        .ok_or_else(|| "Worker script is not a data URL".to_owned())?;
    let (metadata, body) = rest
        .split_once(',')
        .ok_or_else(|| "Malformed worker data URL".to_owned())?;
    let bytes = if metadata
        .split(';')
        .any(|part| part.eq_ignore_ascii_case("base64"))
    {
        decode_base64(body)?
    } else {
        percent_decode(body)?
    };
    String::from_utf8(bytes).map_err(|_| "Worker script is not valid UTF-8".to_owned())
}

fn percent_decode(value: &str) -> Result<Vec<u8>, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = hex(*bytes
                .get(index + 1)
                .ok_or_else(|| "Malformed percent escape".to_owned())?)
            .ok_or_else(|| "Malformed percent escape".to_owned())?;
            let low = hex(*bytes
                .get(index + 2)
                .ok_or_else(|| "Malformed percent escape".to_owned())?)
            .ok_or_else(|| "Malformed percent escape".to_owned())?;
            output.push((high << 4) | low);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    Ok(output)
}

fn decode_base64(value: &str) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(value.len() * 3 / 4);
    let mut block = [0_u8; 4];
    let mut filled = 0;
    for byte in value.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        block[filled] = if byte == b'=' {
            64
        } else {
            base64_value(byte).ok_or_else(|| "Malformed base64 worker URL".to_owned())?
        };
        filled += 1;
        if filled != 4 {
            continue;
        }
        output.push((block[0] << 2) | (block[1] >> 4));
        if block[2] != 64 {
            output.push((block[1] << 4) | (block[2] >> 2));
        }
        if block[3] != 64 {
            output.push((block[2] << 6) | block[3]);
        }
        filled = 0;
    }
    if filled != 0 {
        return Err("Malformed base64 worker URL".to_owned());
    }
    Ok(output)
}

fn hex(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn base64_value(value: u8) -> Option<u8> {
    match value {
        b'A'..=b'Z' => Some(value - b'A'),
        b'a'..=b'z' => Some(value - b'a' + 26),
        b'0'..=b'9' => Some(value - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
