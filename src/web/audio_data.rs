use std::collections::HashMap;

#[derive(Clone)]
struct AudioDataRecord {
    format: String,
    sample_rate: f64,
    number_of_frames: u32,
    number_of_channels: u32,
    timestamp: i64,
    bytes: Vec<u8>,
    closed: bool,
}

#[derive(Clone)]
pub(crate) struct AudioDataEncodingSnapshot {
    pub(crate) format: String,
    pub(crate) sample_rate: f64,
    pub(crate) number_of_frames: u32,
    pub(crate) number_of_channels: u32,
    pub(crate) timestamp: i64,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct AudioDataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioDataRecord>,
}

#[derive(Clone)]
struct CopyOptions {
    format: String,
    plane_index: u32,
    frame_offset: u32,
    frame_count: u32,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioDataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioData", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<AudioDataStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioData",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "format", get_format)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sampleRate", get_sample_rate)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "numberOfFrames",
        get_number_of_frames,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "numberOfChannels",
        get_number_of_channels,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duration", get_duration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timestamp", get_timestamp)?;
    crate::webidl::define_method(scope, prototype, "allocationSize", 1, allocation_size)?;
    crate::webidl::define_method(scope, prototype, "clone", 0, clone_audio_data)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "copyTo", 2, copy_to)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioDataStore>()
        .ok_or_else(|| "AudioData state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    (!value.is_undefined()).then_some(value)
}

fn required_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    member(scope, object, name).map(|value| crate::webidl::value_to_string(scope, value))
}

fn required_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<f64> {
    member(scope, object, name)?.number_value(scope)
}

fn valid_format(format: &str) -> bool {
    matches!(
        format,
        "u8" | "s16" | "s32" | "f32" | "u8-planar" | "s16-planar" | "s32-planar" | "f32-planar"
    )
}

fn bytes_per_sample(format: &str) -> usize {
    if format.starts_with("u8") {
        1
    } else if format.starts_with("s16") {
        2
    } else {
        4
    }
}

fn is_planar(format: &str) -> bool {
    format.ends_with("-planar")
}

fn source_bytes(value: v8::Local<'_, v8::Value>) -> Option<Vec<u8>> {
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        let mut output = vec![0_u8; view.byte_length()];
        let copied = view.copy_contents(&mut output);
        output.truncate(copied);
        return Some(output);
    }
    let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value).ok()?;
    let backing = buffer.get_backing_store();
    let data = backing.data()?;
    Some(
        unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), backing.byte_length()) }
            .to_vec(),
    )
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioData': 1 argument required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "AudioData initializer must be an object");
        return;
    };
    let Some(format) = required_string(scope, init, "format") else {
        crate::webidl::throw_type_error(scope, "Required member format is undefined");
        return;
    };
    if !valid_format(&format) {
        crate::webidl::throw_type_error(scope, "The provided audio format is invalid");
        return;
    }
    let Some(sample_rate) = required_number(scope, init, "sampleRate") else {
        crate::webidl::throw_type_error(scope, "Required member sampleRate is undefined");
        return;
    };
    let Some(number_of_frames) =
        required_number(scope, init, "numberOfFrames").map(|value| value.max(0.0) as u32)
    else {
        crate::webidl::throw_type_error(scope, "Required member numberOfFrames is undefined");
        return;
    };
    let Some(number_of_channels) =
        required_number(scope, init, "numberOfChannels").map(|value| value.max(0.0) as u32)
    else {
        crate::webidl::throw_type_error(scope, "Required member numberOfChannels is undefined");
        return;
    };
    let Some(timestamp) = required_number(scope, init, "timestamp").map(|value| value as i64)
    else {
        crate::webidl::throw_type_error(scope, "Required member timestamp is undefined");
        return;
    };
    if !sample_rate.is_finite()
        || sample_rate <= 0.0
        || number_of_frames == 0
        || number_of_channels == 0
    {
        crate::webidl::throw_type_error(scope, "AudioData dimensions must be positive");
        return;
    }
    let Some(bytes) = member(scope, init, "data").and_then(source_bytes) else {
        crate::webidl::throw_type_error(scope, "Required member data is not a BufferSource");
        return;
    };
    let required_length =
        number_of_frames as usize * number_of_channels as usize * bytes_per_sample(&format);
    if bytes.len() < required_length {
        crate::webidl::throw_type_error(
            scope,
            "AudioData data is smaller than the described audio frames",
        );
        return;
    }
    let record = AudioDataRecord {
        format,
        sample_rate,
        number_of_frames,
        number_of_channels,
        timestamp,
        bytes: bytes[..required_length].to_vec(),
        closed: false,
    };
    scope
        .get_slot_mut::<AudioDataStore>()
        .expect("AudioData state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioDataRecord> {
    scope
        .get_slot::<AudioDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn is_audio_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<AudioDataStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn encoding_snapshot(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioDataEncodingSnapshot> {
    let record = record(scope, object)?;
    (!record.closed).then_some(AudioDataEncodingSnapshot {
        format: record.format,
        sample_rate: record.sample_rate,
        number_of_frames: record.number_of_frames,
        number_of_channels: record.number_of_channels,
        timestamp: record.timestamp,
        bytes: record.bytes,
    })
}

pub(crate) fn create_from_encoding_snapshot<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    snapshot: AudioDataEncodingSnapshot,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create decoded AudioData".to_owned());
    }
    scope
        .get_slot_mut::<AudioDataStore>()
        .ok_or_else(|| "AudioData state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioDataRecord {
                format: snapshot.format,
                sample_rate: snapshot.sample_rate,
                number_of_frames: snapshot.number_of_frames,
                number_of_channels: snapshot.number_of_channels,
                timestamp: snapshot.timestamp,
                bytes: snapshot.bytes,
                closed: false,
            },
        );
    Ok(object)
}

fn get_format(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        result.set(v8::null(scope).into());
    } else if let Some(value) = v8::String::new(scope, &record.format) {
        result.set(value.into());
    }
}

fn return_u32(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioDataRecord) -> u32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let value = if record.closed { 0 } else { select(&record) };
        result.set(v8::Integer::new_from_unsigned(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_number_of_frames(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.number_of_frames)
}
fn get_number_of_channels(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_u32(s, a, r, |v| v.number_of_channels)
}
fn get_sample_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(
            v8::Number::new(
                scope,
                if record.closed {
                    0.0
                } else {
                    record.sample_rate
                },
            )
            .into(),
        );
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        let duration = if record.closed {
            0.0
        } else {
            (record.number_of_frames as f64 * 1_000_000.0 / record.sample_rate).trunc()
        };
        result.set(v8::Number::new(scope, duration).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_timestamp(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.timestamp as f64).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn throw_range_error(scope: &mut v8::PinScope<'_, '_>, message: &str) {
    if let Some(message) = v8::String::new(scope, message) {
        let exception = v8::Exception::range_error(scope, message);
        scope.throw_exception(exception);
    }
}

fn throw_closed(scope: &mut v8::PinScope<'_, '_>, operation: &str) {
    let message = format!("Failed to execute '{operation}' on 'AudioData': AudioData is closed.");
    if let Ok(exception) =
        super::dom_exception::create(scope, message, "InvalidStateError".to_owned())
    {
        scope.throw_exception(exception.into());
    }
}

fn copy_options(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    record: &AudioDataRecord,
) -> Option<CopyOptions> {
    let Ok(options) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "AudioData copy options are required");
        return None;
    };
    let Some(plane_index) =
        required_number(scope, options, "planeIndex").map(|value| value.max(0.0) as u32)
    else {
        crate::webidl::throw_type_error(scope, "Required member planeIndex is undefined");
        return None;
    };
    let format = required_string(scope, options, "format").unwrap_or_else(|| record.format.clone());
    if !valid_format(&format) {
        crate::webidl::throw_type_error(scope, "The provided audio format is invalid");
        return None;
    }
    if (is_planar(&format) && plane_index >= record.number_of_channels)
        || (!is_planar(&format) && plane_index != 0)
    {
        throw_range_error(scope, "Invalid planeIndex.");
        return None;
    }
    let frame_offset = required_number(scope, options, "frameOffset")
        .map(|value| value.max(0.0) as u32)
        .unwrap_or(0);
    if frame_offset > record.number_of_frames {
        throw_range_error(scope, "frameOffset exceeds the available frames.");
        return None;
    }
    let available = record.number_of_frames - frame_offset;
    let frame_count = required_number(scope, options, "frameCount")
        .map(|value| value.max(0.0) as u32)
        .unwrap_or(available);
    if frame_count > available {
        throw_range_error(scope, "frameCount exceeds the available frames.");
        return None;
    }
    Some(CopyOptions {
        format,
        plane_index,
        frame_offset,
        frame_count,
    })
}

fn allocation_length(record: &AudioDataRecord, options: &CopyOptions) -> usize {
    let channel_factor = if is_planar(&options.format) {
        1
    } else {
        record.number_of_channels as usize
    };
    options.frame_count as usize * channel_factor * bytes_per_sample(&options.format)
}

fn allocation_size(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        throw_closed(scope, "allocationSize");
        return;
    }
    let Some(options) = copy_options(scope, arguments.get(0), &record) else {
        return;
    };
    result.set(v8::Number::new(scope, allocation_length(&record, &options) as f64).into());
}

fn read_sample(record: &AudioDataRecord, channel: u32, frame: u32) -> f32 {
    let sample_index = if is_planar(&record.format) {
        channel as usize * record.number_of_frames as usize + frame as usize
    } else {
        frame as usize * record.number_of_channels as usize + channel as usize
    };
    let offset = sample_index * bytes_per_sample(&record.format);
    if record.format.starts_with("u8") {
        (record.bytes[offset] as f32 - 128.0) / 128.0
    } else if record.format.starts_with("s16") {
        let bytes = [record.bytes[offset], record.bytes[offset + 1]];
        i16::from_le_bytes(bytes) as f32 / 32768.0
    } else if record.format.starts_with("s32") {
        let bytes = [
            record.bytes[offset],
            record.bytes[offset + 1],
            record.bytes[offset + 2],
            record.bytes[offset + 3],
        ];
        i32::from_le_bytes(bytes) as f32 / 2_147_483_648.0
    } else {
        let bytes = [
            record.bytes[offset],
            record.bytes[offset + 1],
            record.bytes[offset + 2],
            record.bytes[offset + 3],
        ];
        f32::from_le_bytes(bytes)
    }
}

fn write_sample(output: &mut Vec<u8>, format: &str, sample: f32) {
    if format.starts_with("u8") {
        let sample = sample.clamp(-1.0, 1.0);
        output.push((sample * 128.0 + 128.0).round().clamp(0.0, 255.0) as u8);
    } else if format.starts_with("s16") {
        let sample = sample.clamp(-1.0, 1.0);
        let value = (sample * 32768.0)
            .round()
            .clamp(i16::MIN as f32, i16::MAX as f32) as i16;
        output.extend_from_slice(&value.to_le_bytes());
    } else if format.starts_with("s32") {
        let sample = sample.clamp(-1.0, 1.0);
        let value = (sample as f64 * 2_147_483_648.0)
            .round()
            .clamp(i32::MIN as f64, i32::MAX as f64) as i32;
        output.extend_from_slice(&value.to_le_bytes());
    } else {
        output.extend_from_slice(&sample.to_le_bytes());
    }
}

fn converted_bytes(record: &AudioDataRecord, options: &CopyOptions) -> Vec<u8> {
    let mut output = Vec::with_capacity(allocation_length(record, options));
    if is_planar(&options.format) {
        for frame in options.frame_offset..options.frame_offset + options.frame_count {
            write_sample(
                &mut output,
                &options.format,
                read_sample(record, options.plane_index, frame),
            );
        }
    } else {
        for frame in options.frame_offset..options.frame_offset + options.frame_count {
            for channel in 0..record.number_of_channels {
                write_sample(
                    &mut output,
                    &options.format,
                    read_sample(record, channel, frame),
                );
            }
        }
    }
    output
}

fn destination(value: v8::Local<'_, v8::Value>) -> Option<(*mut u8, usize)> {
    if let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) {
        return Some((view.data().cast::<u8>(), view.byte_length()));
    }
    let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value).ok()?;
    let backing = buffer.get_backing_store();
    let pointer = backing.data()?.as_ptr().cast::<u8>();
    Some((pointer, backing.byte_length()))
}

fn copy_to(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        throw_closed(scope, "copyTo");
        return;
    }
    let Some((pointer, length)) = destination(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "destination is not a BufferSource");
        return;
    };
    let Some(options) = copy_options(scope, arguments.get(1), &record) else {
        return;
    };
    let bytes = converted_bytes(&record, &options);
    if length < bytes.len() {
        throw_range_error(scope, "destination is not large enough.");
        return;
    }
    if !bytes.is_empty() {
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer, bytes.len());
        }
    }
}

fn clone_audio_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        throw_closed(scope, "clone");
        return;
    }
    let Ok(constructor) = ensure_constructor(scope) else {
        return;
    };
    let Ok(prototype) = crate::webidl::prototype(scope, constructor) else {
        return;
    };
    let clone = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, clone, prototype.into()) != Some(true) {
        crate::webidl::throw_type_error(scope, "cannot clone AudioData");
        return;
    }
    scope
        .get_slot_mut::<AudioDataStore>()
        .expect("AudioData state")
        .records
        .insert(clone.get_identity_hash().get(), record);
    result.set(clone.into());
}

fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<AudioDataStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.closed = true;
        record.bytes.clear();
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<AudioDataStore>() {
        store.constructor.remove(realm_id);
    }
}
