use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TextDecoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DecoderRecord>,
}

#[derive(Clone)]
struct DecoderRecord {
    encoding: String,
    fatal: bool,
    ignore_bom: bool,
    pending: Vec<u8>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TextDecoderStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "TextDecoder", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<TextDecoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "TextDecoder",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "encoding", get_encoding)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "fatal", get_fatal)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ignoreBOM", get_ignore_bom)?;
    crate::webidl::define_method(scope, prototype, "decode", 0, decode)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TextDecoderStore>()
        .ok_or_else(|| "TextDecoder state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'TextDecoder': use new");
        return;
    }
    let label = if arguments.length() == 0 || arguments.get(0).is_undefined() {
        "utf-8".to_owned()
    } else {
        crate::webidl::value_to_string(scope, arguments.get(0))
    };
    let Some(encoding) = canonical_encoding(&label) else {
        crate::webidl::throw_type_error(scope, "The encoding label is not supported");
        return;
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let fatal = options.is_some_and(|value| super::event::boolean_property(scope, value, "fatal"));
    let ignore_bom =
        options.is_some_and(|value| super::event::boolean_property(scope, value, "ignoreBOM"));
    let object = arguments.this();
    scope
        .get_slot_mut::<TextDecoderStore>()
        .expect("TextDecoder state")
        .records
        .insert(
            object.get_identity_hash().get(),
            DecoderRecord {
                encoding: encoding.to_owned(),
                fatal,
                ignore_bom,
                pending: Vec::new(),
            },
        );
    result.set(object.into());
}

fn canonical_encoding(label: &str) -> Option<&'static str> {
    match label.trim().to_ascii_lowercase().as_str() {
        "utf-8" | "utf8" | "unicode-1-1-utf-8" => Some("utf-8"),
        "utf-16" | "utf-16le" | "unicodefffe" => Some("utf-16le"),
        "utf-16be" | "unicodefeff" => Some("utf-16be"),
        "windows-1252" | "cp1252" | "ascii" | "us-ascii" | "iso-8859-1" | "latin1" => {
            Some("windows-1252")
        }
        _ => None,
    }
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DecoderRecord> {
    scope
        .get_slot::<TextDecoderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_encoding(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, &record.encoding) {
        result.set(value.into());
    }
}

fn get_fatal(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.fatal).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_ignore_bom(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.ignore_bom).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn bytes_from_value(
    _scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Result<Vec<u8>, String> {
    if value.is_undefined() {
        return Ok(Vec::new());
    }
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut bytes = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut bytes);
        bytes.truncate(copied);
        return Ok(bytes);
    }
    if let Ok(buffer) = v8::Local::<v8::ArrayBuffer>::try_from(value) {
        let backing = buffer.get_backing_store();
        return Ok(backing.iter().map(|cell| cell.get()).collect());
    }
    Err("input must be an ArrayBuffer or ArrayBufferView".to_owned())
}

fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let mut bytes = match bytes_from_value(scope, arguments.get(0)) {
        Ok(bytes) => bytes,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let streaming =
        options.is_some_and(|value| super::event::boolean_property(scope, value, "stream"));
    let identity = arguments.this().get_identity_hash().get();
    let mut combined = record.pending.clone();
    combined.append(&mut bytes);
    let decoded = match decode_bytes(&record.encoding, &combined, record.fatal, record.ignore_bom) {
        Ok(value) => value,
        Err(message) => {
            crate::webidl::throw_type_error(
                scope,
                &format!("Failed to execute 'decode' on 'TextDecoder': {message}"),
            );
            return;
        }
    };
    if let Some(current) = scope
        .get_slot_mut::<TextDecoderStore>()
        .and_then(|store| store.records.get_mut(&identity))
    {
        current.pending = if streaming {
            incomplete_suffix(&record.encoding, &combined)
        } else {
            Vec::new()
        };
    }
    if let Some(value) = v8::String::new(scope, &decoded) {
        result.set(value.into());
    }
}

pub(crate) fn decode_bytes(
    encoding: &str,
    bytes: &[u8],
    fatal: bool,
    ignore_bom: bool,
) -> Result<String, String> {
    let mut output = match encoding {
        "utf-8" => {
            if fatal {
                std::str::from_utf8(bytes)
                    .map_err(|_| "The encoded data was not valid.".to_owned())?
                    .to_owned()
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }
        "utf-16le" => decode_utf16(bytes, true, fatal)?,
        "utf-16be" => decode_utf16(bytes, false, fatal)?,
        "windows-1252" => bytes.iter().map(|byte| windows_1252(*byte)).collect(),
        _ => return Err("Unsupported encoding".to_owned()),
    };
    if !ignore_bom && output.starts_with('\u{feff}') {
        output.remove(0);
    }
    Ok(output)
}

fn decode_utf16(bytes: &[u8], little_endian: bool, fatal: bool) -> Result<String, String> {
    if fatal && bytes.len() % 2 != 0 {
        return Err("The encoded data has an incomplete UTF-16 code unit".to_owned());
    }
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for pair in bytes.chunks_exact(2) {
        units.push(if little_endian {
            u16::from_le_bytes([pair[0], pair[1]])
        } else {
            u16::from_be_bytes([pair[0], pair[1]])
        });
    }
    if fatal {
        char::decode_utf16(units)
            .map(|value| value.map_err(|_| "The encoded data was not valid.".to_owned()))
            .collect()
    } else {
        Ok(char::decode_utf16(units)
            .map(|value| value.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect())
    }
}

fn windows_1252(byte: u8) -> char {
    match byte {
        0x80 => '\u{20ac}',
        0x82 => '\u{201a}',
        0x83 => '\u{0192}',
        0x84 => '\u{201e}',
        0x85 => '\u{2026}',
        0x86 => '\u{2020}',
        0x87 => '\u{2021}',
        0x88 => '\u{02c6}',
        0x89 => '\u{2030}',
        0x8a => '\u{0160}',
        0x8b => '\u{2039}',
        0x8c => '\u{0152}',
        0x8e => '\u{017d}',
        0x91 => '\u{2018}',
        0x92 => '\u{2019}',
        0x93 => '\u{201c}',
        0x94 => '\u{201d}',
        0x95 => '\u{2022}',
        0x96 => '\u{2013}',
        0x97 => '\u{2014}',
        0x98 => '\u{02dc}',
        0x99 => '\u{2122}',
        0x9a => '\u{0161}',
        0x9b => '\u{203a}',
        0x9c => '\u{0153}',
        0x9e => '\u{017e}',
        0x9f => '\u{0178}',
        value => char::from_u32(value as u32).unwrap_or(char::REPLACEMENT_CHARACTER),
    }
}

fn incomplete_suffix(encoding: &str, bytes: &[u8]) -> Vec<u8> {
    if matches!(encoding, "utf-16le" | "utf-16be") && bytes.len() % 2 == 1 {
        return vec![*bytes.last().unwrap_or(&0)];
    }
    if encoding != "utf-8" {
        return Vec::new();
    }
    let start = bytes.len().saturating_sub(3);
    for index in start..bytes.len() {
        if std::str::from_utf8(&bytes[index..]).is_ok() {
            return Vec::new();
        }
        let first = bytes[index];
        let expected = if first & 0b1111_1000 == 0b1111_0000 {
            4
        } else if first & 0b1111_0000 == 0b1110_0000 {
            3
        } else if first & 0b1110_0000 == 0b1100_0000 {
            2
        } else {
            continue;
        };
        if bytes.len() - index < expected {
            return bytes[index..].to_vec();
        }
    }
    Vec::new()
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<TextDecoderStore>() {
        store.constructor.remove(realm_id);
    }
}
