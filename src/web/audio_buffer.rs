use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct AudioBufferStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioBufferRecord>,
}

#[derive(Clone)]
struct AudioBufferRecord {
    length: u32,
    sample_rate: f64,
    channels: Vec<v8::Global<v8::Float32Array>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioBufferStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioBuffer", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<AudioBufferStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioBuffer",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duration", get_duration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sampleRate", get_sample_rate)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "numberOfChannels",
        get_number_of_channels,
    )?;
    crate::webidl::define_method(scope, prototype, "copyFromChannel", 2, copy_from_channel)?;
    crate::webidl::define_method(scope, prototype, "copyToChannel", 2, copy_to_channel)?;
    crate::webidl::define_method(scope, prototype, "getChannelData", 1, get_channel_data)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioBufferStore>()
        .ok_or_else(|| "AudioBuffer state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioBuffer': 1 argument required",
        );
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioBuffer': The provided value is not of type 'AudioBufferOptions'.",
        );
        return;
    };
    let length_key = v8::String::new(scope, "length").expect("length key");
    if options
        .get(scope, length_key.into())
        .is_none_or(|value| value.is_undefined())
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioBuffer': Failed to read the 'length' property from 'AudioBufferOptions': Required member is undefined.",
        );
        return;
    }
    let sample_rate_key = v8::String::new(scope, "sampleRate").expect("sampleRate key");
    if options
        .get(scope, sample_rate_key.into())
        .is_none_or(|value| value.is_undefined())
    {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AudioBuffer': Failed to read the 'sampleRate' property from 'AudioBufferOptions': Required member is undefined.",
        );
        return;
    }
    let length = super::event::number_property(scope, options, "length", 0.0) as u32;
    let sample_rate = super::event::number_property(scope, options, "sampleRate", 0.0);
    let channels = super::event::number_property(scope, options, "numberOfChannels", 1.0) as u32;
    if length == 0
        || !(1..=32).contains(&channels)
        || !(3_000.0..=768_000.0).contains(&sample_rate)
        || !sample_rate.is_finite()
    {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The AudioBuffer options are outside the supported range".to_owned(),
            "NotSupportedError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    match attach(scope, arguments.this(), channels, length, sample_rate) {
        Ok(()) => result.set(arguments.this().into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    number_of_channels: u32,
    length: u32,
    sample_rate: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    if number_of_channels == 0 || length == 0 || sample_rate <= 0.0 {
        return Err("AudioBuffer dimensions must be positive".to_owned());
    }
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create AudioBuffer".to_owned());
    }
    attach(scope, object, number_of_channels, length, sample_rate)?;
    Ok(object)
}

pub(crate) fn is_buffer(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<AudioBufferStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn duration(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<f64> {
    let record = record(scope, object)?;
    Some(record.length as f64 / record.sample_rate)
}

pub(crate) fn number_of_channels(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<u32> {
    Some(record(scope, object)?.channels.len() as u32)
}

pub(crate) fn length(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<u32> {
    Some(record(scope, object)?.length)
}

pub(crate) fn sample(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    channel_index: u32,
    sample_index: u32,
) -> Option<f32> {
    let record = record(scope, object)?;
    let channel = record.channels.get(channel_index as usize)?;
    let channel = v8::Local::new(scope, channel);
    channel
        .get_index(scope, sample_index)
        .and_then(|value| value.number_value(scope))
        .map(|value| value as f32)
}

pub(crate) fn set_sample(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    channel_index: u32,
    sample_index: u32,
    sample: f32,
) -> bool {
    let Some(record) = record(scope, object) else {
        return false;
    };
    let Some(channel) = record.channels.get(channel_index as usize) else {
        return false;
    };
    let channel = v8::Local::new(scope, channel);
    channel.set_index(
        scope,
        sample_index,
        v8::Number::new(scope, f64::from(sample)).into(),
    ) == Some(true)
}

fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    number_of_channels: u32,
    length: u32,
    sample_rate: f64,
) -> Result<(), String> {
    let mut channels = Vec::with_capacity(number_of_channels as usize);
    for _ in 0..number_of_channels {
        let bytes = vec![0_u8; length as usize * std::mem::size_of::<f32>()];
        let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
        let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
        let array = v8::Float32Array::new(scope, buffer, 0, length as usize)
            .ok_or_else(|| "cannot create AudioBuffer channel storage".to_owned())?;
        channels.push(v8::Global::new(scope, array));
    }
    scope
        .get_slot_mut::<AudioBufferStore>()
        .ok_or_else(|| "AudioBuffer state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AudioBufferRecord {
                length,
                sample_rate,
                channels,
            },
        );
    Ok(())
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioBufferRecord> {
    scope
        .get_slot::<AudioBufferStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn apply_fingerprint_noise(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) {
    let fingerprint = &crate::fingerprint::edge(scope).rendering.audio;
    if fingerprint.channel_noise_amplitude == 0.0 {
        return;
    }
    let seed = fingerprint.noise_seed;
    let amplitude = fingerprint.channel_noise_amplitude;
    let Some(record) = record(scope, object) else {
        return;
    };
    for (channel_index, channel) in record.channels.iter().enumerate() {
        let channel = v8::Local::new(scope, channel);
        for sample_index in 0..record.length {
            let ordinal = ((channel_index as u64) << 32) | sample_index as u64;
            let mut value = seed ^ ordinal.wrapping_mul(0x9e37_79b9_7f4a_7c15);
            value ^= value >> 30;
            value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value ^= value >> 27;
            value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^= value >> 31;
            let unit = ((value >> 40) as f32) / ((1_u32 << 24) as f32);
            let noise = (unit * 2.0 - 1.0) * amplitude;
            let current = channel
                .get_index(scope, sample_index as u32)
                .and_then(|value| value.number_value(scope))
                .unwrap_or(0.0) as f32;
            let sample = current + noise;
            let sample = v8::Number::new(scope, sample as f64);
            let _ = channel.set_index(scope, sample_index, sample.into());
        }
    }
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.length).into());
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
        result.set(v8::Number::new(scope, record.length as f64 / record.sample_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_sample_rate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.sample_rate).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_number_of_channels(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.channels.len() as u32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn channel<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    index_value: v8::Local<'_, v8::Value>,
) -> Option<v8::Local<'s, v8::Float32Array>> {
    let index = index_value.uint32_value(scope).unwrap_or(0) as usize;
    let Some(record) = record(scope, object) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return None;
    };
    let Some(channel) = record.channels.get(index) else {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The channel index is outside the buffer".to_owned(),
            "IndexSizeError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return None;
    };
    Some(v8::Local::new(scope, channel))
}

fn get_channel_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(channel) = channel(scope, arguments.this(), arguments.get(0)) {
        result.set(channel.into());
    }
}

fn copy_from_channel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(destination) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "destination must be a Float32Array");
        return;
    };
    let Some(source) = channel(scope, arguments.this(), arguments.get(1)) else {
        return;
    };
    let offset = arguments.get(2).uint32_value(scope).unwrap_or(0);
    let length_key = v8::String::new(scope, "length").unwrap();
    let length = destination
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        let value = source
            .get_index(scope, offset.saturating_add(index))
            .unwrap_or_else(|| v8::undefined(scope).into());
        let _ = destination.set_index(scope, index, value);
    }
}

fn copy_to_channel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "source must be a Float32Array");
        return;
    };
    let Some(destination) = channel(scope, arguments.this(), arguments.get(1)) else {
        return;
    };
    let offset = arguments.get(2).uint32_value(scope).unwrap_or(0);
    let length_key = v8::String::new(scope, "length").unwrap();
    let length = source
        .get(scope, length_key.into())
        .and_then(|value| value.uint32_value(scope))
        .unwrap_or(0);
    for index in 0..length {
        if offset.saturating_add(index) >= destination.length() as u32 {
            break;
        }
        if let Some(value) = source.get_index(scope, index) {
            let _ = destination.set_index(scope, offset + index, value);
        }
    }
}
