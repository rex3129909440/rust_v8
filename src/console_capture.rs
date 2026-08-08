const MAX_ENTRIES: usize = 16_384;
const MAX_ARGUMENTS: usize = 64;
const MAX_SEQUENCE_VALUES: usize = 256;
const MAX_OBJECT_PROPERTIES: usize = 128;
const MAX_DEPTH: usize = 4;
const MAX_STRING_BYTES: usize = 1024 * 1024;
const MAX_BINARY_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[repr(u8)]
pub enum ConsoleLevel {
    Debug = 1,
    Info = 2,
    Log = 3,
    Warn = 4,
    Error = 5,
    Dir = 6,
    DirXml = 7,
    Table = 8,
    Trace = 9,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ConsoleValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String {
        value: String,
        truncated: bool,
    },
    BigInt {
        value: String,
        truncated: bool,
    },
    Bytes {
        type_name: String,
        value: Vec<u8>,
        truncated: bool,
    },
    Sequence {
        type_name: String,
        values: Vec<ConsoleValue>,
        truncated: bool,
    },
    Object {
        type_name: String,
        entries: Vec<(String, ConsoleValue)>,
        truncated: bool,
    },
    Other {
        type_name: String,
        display: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CapturedConsoleOutput {
    pub sequence: u64,
    pub level: ConsoleLevel,
    pub frame_url: String,
    pub text: String,
    pub arguments: Vec<ConsoleValue>,
}

#[derive(Default)]
struct ConsoleCaptureState {
    next_sequence: u64,
    entries: Vec<CapturedConsoleOutput>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ConsoleCaptureState::default());
}

pub(crate) fn record<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    level: ConsoleLevel,
    callback_arguments: &v8::FunctionCallbackArguments<'s>,
) {
    let count = callback_arguments.length().max(0) as usize;
    let retained = count.min(MAX_ARGUMENTS);
    let mut arguments = Vec::with_capacity(retained);
    for index in 0..retained {
        arguments.push(capture_value(
            scope,
            callback_arguments.get(index as i32),
            0,
            true,
        ));
    }
    if count > retained {
        arguments.push(ConsoleValue::Other {
            type_name: "TruncatedArguments".to_owned(),
            display: format!("{} additional arguments", count - retained),
        });
    }
    let text = arguments
        .iter()
        .map(display_value)
        .collect::<Vec<_>>()
        .join(" ");
    let frame_url = current_realm_url(scope);

    let Some(state) = scope.get_slot_mut::<ConsoleCaptureState>() else {
        return;
    };
    state.next_sequence = state.next_sequence.saturating_add(1);
    if state.entries.len() >= MAX_ENTRIES {
        return;
    }
    state.entries.push(CapturedConsoleOutput {
        sequence: state.next_sequence,
        level,
        frame_url,
        text,
        arguments,
    });
}

pub(crate) fn entries(isolate: &v8::OwnedIsolate) -> Vec<CapturedConsoleOutput> {
    isolate
        .get_slot::<ConsoleCaptureState>()
        .map(|state| state.entries.clone())
        .unwrap_or_default()
}

pub(crate) fn clear(isolate: &mut v8::OwnedIsolate) {
    if let Some(state) = isolate.get_slot_mut::<ConsoleCaptureState>() {
        state.entries.clear();
        state.next_sequence = 0;
    }
}

pub(crate) fn observe_arguments<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    callback_arguments: &v8::FunctionCallbackArguments<'s>,
    start: usize,
) {
    let count = callback_arguments.length().max(0) as usize;
    for index in start..count.min(MAX_ARGUMENTS) {
        observe_console_value(scope, callback_arguments.get(index as i32), 0);
    }
}

fn capture_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    depth: usize,
    format_nested_errors: bool,
) -> ConsoleValue {
    if value.is_undefined() {
        return ConsoleValue::Undefined;
    }
    if value.is_null() {
        return ConsoleValue::Null;
    }
    if value.is_boolean() {
        return ConsoleValue::Boolean(value.boolean_value(scope));
    }
    if value.is_number() {
        return ConsoleValue::Number(value.number_value(scope).unwrap_or(f64::NAN));
    }
    if value.is_string() {
        let text = crate::webidl::value_to_string(scope, value);
        let (value, truncated) = bounded_string(text, MAX_STRING_BYTES);
        return ConsoleValue::String { value, truncated };
    }
    if value.is_big_int() {
        let text = crate::webidl::value_to_string(scope, value);
        let (value, truncated) = bounded_string(text, MAX_STRING_BYTES);
        return ConsoleValue::BigInt { value, truncated };
    }
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let backing = buffer.get_backing_store();
        let available = backing.byte_length();
        let retained = available.min(MAX_BINARY_BYTES);
        let bytes = backing
            .data()
            .map(|data| {
                // SAFETY: the backing store remains alive while its bytes are copied.
                unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), retained) }.to_vec()
            })
            .unwrap_or_default();
        return ConsoleValue::Bytes {
            type_name: "ArrayBuffer".to_owned(),
            value: bytes,
            truncated: available > retained,
        };
    }
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let available = view.byte_length();
        let retained = available.min(MAX_BINARY_BYTES);
        let mut bytes = vec![0_u8; retained];
        let copied = view.copy_contents(&mut bytes);
        bytes.truncate(copied);
        return ConsoleValue::Bytes {
            type_name: value.type_repr().to_owned(),
            value: bytes,
            truncated: available > retained,
        };
    }
    if depth < MAX_DEPTH && (value.is_array() || value.is_arguments_object()) {
        if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
            let length = sequence_length(scope, object).min(MAX_SEQUENCE_VALUES);
            let mut values = Vec::with_capacity(length);
            for index in 0..length {
                values.push(
                    own_data_value(scope, object, &index.to_string())
                        .map(|item| capture_value(scope, item, depth + 1, format_nested_errors))
                        .unwrap_or(ConsoleValue::Undefined),
                );
            }
            let actual_length = sequence_length(scope, object);
            return ConsoleValue::Sequence {
                type_name: if value.is_arguments_object() {
                    "Arguments".to_owned()
                } else {
                    "Array".to_owned()
                },
                values,
                truncated: actual_length > length,
            };
        }
    }
    if value.is_symbol() {
        let description = v8::Local::<v8::Symbol>::try_from(value)
            .ok()
            .map(|symbol| symbol.description(scope))
            .filter(|description| !description.is_undefined())
            .map(|description| crate::webidl::value_to_string(scope, description))
            .unwrap_or_default();
        return ConsoleValue::Other {
            type_name: "Symbol".to_owned(),
            display: format!("Symbol({description})"),
        };
    }
    if value.is_function() {
        let name = v8::Local::<v8::Function>::try_from(value)
            .ok()
            .map(|function| function.get_name(scope).to_rust_string_lossy(scope))
            .unwrap_or_default();
        return ConsoleValue::Other {
            type_name: "Function".to_owned(),
            display: if name.is_empty() {
                "[function]".to_owned()
            } else {
                format!("[function {name}]")
            },
        };
    }
    if value.is_native_error() && format_nested_errors {
        observe_error_name_and_message(scope, value);
    }
    let object = v8::Local::<v8::Object>::try_from(value).ok();
    let type_name = object
        .map(|object| object.get_constructor_name().to_rust_string_lossy(scope))
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| value.type_repr().to_owned());
    if depth < MAX_DEPTH && !value.is_proxy() {
        if let Some(object) = object {
            let property_arguments = v8::GetPropertyNamesArgs {
                mode: v8::KeyCollectionMode::OwnOnly,
                property_filter: v8::PropertyFilter::SKIP_SYMBOLS,
                index_filter: v8::IndexFilter::IncludeIndices,
                key_conversion: v8::KeyConversionMode::ConvertToString,
            };
            if let Some(keys) = object.get_own_property_names(scope, property_arguments) {
                let actual_count = keys.length() as usize;
                if actual_count > 0 {
                    let retained = actual_count.min(MAX_OBJECT_PROPERTIES);
                    let mut entries = Vec::with_capacity(retained);
                    for index in 0..retained {
                        let Some(key_value) = keys.get_index(scope, index as u32) else {
                            continue;
                        };
                        let Some(key) = key_value.to_string(scope) else {
                            continue;
                        };
                        let captured = own_data_value_by_key(scope, object, key.into())
                            .map(|item| capture_value(scope, item, depth + 1, false))
                            .unwrap_or(ConsoleValue::Other {
                                type_name: "Accessor".to_owned(),
                                display: "[accessor]".to_owned(),
                            });
                        let name = key.to_rust_string_lossy(scope);
                        entries.push((name, captured));
                    }
                    return ConsoleValue::Object {
                        type_name,
                        entries,
                        truncated: actual_count > retained,
                    };
                }
            }
        }
    }
    ConsoleValue::Other {
        display: format!("[object {type_name}]"),
        type_name,
    }
}

fn observe_error_name_and_message(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) {
    let Ok(error) = v8::Local::<v8::Object>::try_from(value) else {
        return;
    };
    v8::tc_scope!(let try_catch, scope);
    for name in ["name", "message"] {
        let Some(key) = v8::String::new(try_catch, name) else {
            return;
        };
        let Some(property) = error.get(try_catch, key.into()) else {
            try_catch.reset();
            return;
        };
        if !property.is_undefined() && property.to_string(try_catch).is_none() {
            try_catch.reset();
            return;
        }
    }
}

fn observe_console_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    value: v8::Local<'s, v8::Value>,
    depth: usize,
) {
    if value.is_native_error() {
        observe_error_name_and_message(scope, value);
        return;
    }
    if depth >= MAX_DEPTH || (!value.is_array() && !value.is_arguments_object()) {
        return;
    }
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return;
    };
    let length = sequence_length(scope, object).min(MAX_SEQUENCE_VALUES);
    for index in 0..length {
        if let Some(item) = own_data_value(scope, object, &index.to_string()) {
            observe_console_value(scope, item, depth + 1);
        }
    }
}

fn sequence_length<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
) -> usize {
    own_data_value(scope, object, "length")
        .and_then(|value| value.uint32_value(scope))
        .map(|value| value as usize)
        .unwrap_or_default()
}

fn own_data_value<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    own_data_value_by_key(scope, object, key.into())
}

fn own_data_value_by_key<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    key: v8::Local<'s, v8::Name>,
) -> Option<v8::Local<'s, v8::Value>> {
    let descriptor = object.get_own_property_descriptor(scope, key)?;
    let descriptor = v8::Local::<v8::Object>::try_from(descriptor).ok()?;
    let value_key = v8::String::new(scope, "value")?;
    if descriptor.has_own_property(scope, value_key.into()) != Some(true) {
        return None;
    }
    descriptor.get(scope, value_key.into())
}

fn current_realm_url(scope: &mut v8::PinScope<'_, '_>) -> String {
    let global = scope.get_current_context().global(scope);
    let location_key = match v8::String::new(scope, "location") {
        Some(value) => value,
        None => return String::new(),
    };
    let href_key = match v8::String::new(scope, "href") {
        Some(value) => value,
        None => return String::new(),
    };
    global
        .get(scope, location_key.into())
        .and_then(|value| v8::Local::<v8::Object>::try_from(value).ok())
        .and_then(|location| location.get(scope, href_key.into()))
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}

fn bounded_string(value: String, maximum_bytes: usize) -> (String, bool) {
    if value.len() <= maximum_bytes {
        return (value, false);
    }
    let mut boundary = maximum_bytes;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    (value[..boundary].to_owned(), true)
}

fn display_value(value: &ConsoleValue) -> String {
    match value {
        ConsoleValue::Undefined => "undefined".to_owned(),
        ConsoleValue::Null => "null".to_owned(),
        ConsoleValue::Boolean(value) => value.to_string(),
        ConsoleValue::Number(value) => value.to_string(),
        ConsoleValue::String { value, truncated } | ConsoleValue::BigInt { value, truncated } => {
            if *truncated {
                format!("{value}…")
            } else {
                value.clone()
            }
        }
        ConsoleValue::Bytes {
            type_name,
            value,
            truncated,
        } => {
            let suffix = if *truncated { ", …" } else { "" };
            format!(
                "{type_name}({}) [{}{suffix}]",
                value.len(),
                value
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ConsoleValue::Sequence {
            type_name,
            values,
            truncated,
        } => {
            let suffix = if *truncated { ", …" } else { "" };
            format!(
                "{type_name}({}) [{}{suffix}]",
                values.len(),
                values
                    .iter()
                    .map(display_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ConsoleValue::Object {
            type_name,
            entries,
            truncated,
        } => {
            let suffix = if *truncated { ", …" } else { "" };
            format!(
                "{type_name} {{{}{suffix}}}",
                entries
                    .iter()
                    .map(|(name, value)| format!("{name}: {}", display_value(value)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        ConsoleValue::Other { display, .. } => display.clone(),
    }
}

pub(crate) fn encode_binary(entries: &[CapturedConsoleOutput]) -> Result<Vec<u8>, String> {
    let count = u32::try_from(entries.len())
        .map_err(|_| "too many console stdout entries to export".to_owned())?;
    let mut output = Vec::new();
    output.extend_from_slice(b"ESSO");
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&0_u16.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    for entry in entries {
        output.extend_from_slice(&entry.sequence.to_le_bytes());
        output.push(entry.level as u8);
        output.extend_from_slice(&[0_u8; 3]);
        write_text(&mut output, &entry.frame_url)?;
        write_text(&mut output, &entry.text)?;
        let argument_count = u32::try_from(entry.arguments.len())
            .map_err(|_| "console stdout entry has too many arguments".to_owned())?;
        output.extend_from_slice(&argument_count.to_le_bytes());
        for value in &entry.arguments {
            encode_value(&mut output, value)?;
        }
    }
    Ok(output)
}

fn write_text(output: &mut Vec<u8>, value: &str) -> Result<(), String> {
    let length =
        u32::try_from(value.len()).map_err(|_| "console stdout text is too large".to_owned())?;
    output.extend_from_slice(&length.to_le_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

fn encode_value(output: &mut Vec<u8>, value: &ConsoleValue) -> Result<(), String> {
    let (tag, truncated) = match value {
        ConsoleValue::Undefined => (0, false),
        ConsoleValue::Null => (1, false),
        ConsoleValue::Boolean(_) => (2, false),
        ConsoleValue::Number(_) => (3, false),
        ConsoleValue::String { truncated, .. } => (4, *truncated),
        ConsoleValue::BigInt { truncated, .. } => (5, *truncated),
        ConsoleValue::Bytes { truncated, .. } => (6, *truncated),
        ConsoleValue::Sequence { truncated, .. } => (7, *truncated),
        ConsoleValue::Other { .. } => (8, false),
        ConsoleValue::Object { truncated, .. } => (9, *truncated),
    };
    output.push(tag);
    output.push(u8::from(truncated));
    output.extend_from_slice(&0_u16.to_le_bytes());
    match value {
        ConsoleValue::Undefined | ConsoleValue::Null => {}
        ConsoleValue::Boolean(value) => output.push(u8::from(*value)),
        ConsoleValue::Number(value) => output.extend_from_slice(&value.to_le_bytes()),
        ConsoleValue::String { value, .. } | ConsoleValue::BigInt { value, .. } => {
            write_text(output, value)?;
        }
        ConsoleValue::Bytes {
            type_name, value, ..
        } => {
            write_text(output, type_name)?;
            let length = u64::try_from(value.len())
                .map_err(|_| "console stdout byte value is too large".to_owned())?;
            output.extend_from_slice(&length.to_le_bytes());
            output.extend_from_slice(value);
        }
        ConsoleValue::Sequence {
            type_name, values, ..
        } => {
            write_text(output, type_name)?;
            let count = u32::try_from(values.len())
                .map_err(|_| "console stdout sequence is too large".to_owned())?;
            output.extend_from_slice(&count.to_le_bytes());
            for value in values {
                encode_value(output, value)?;
            }
        }
        ConsoleValue::Object {
            type_name, entries, ..
        } => {
            write_text(output, type_name)?;
            let count = u32::try_from(entries.len())
                .map_err(|_| "console stdout object is too large".to_owned())?;
            output.extend_from_slice(&count.to_le_bytes());
            for (name, value) in entries {
                write_text(output, name)?;
                encode_value(output, value)?;
            }
        }
        ConsoleValue::Other { type_name, display } => {
            write_text(output, type_name)?;
            write_text(output, display)?;
        }
    }
    Ok(())
}
