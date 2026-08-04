use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcTrackEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, TrackEventRecord>,
}

#[derive(Clone)]
pub(crate) struct TrackEventRecord {
    pub(crate) receiver: v8::Global<v8::Object>,
    pub(crate) track: v8::Global<v8::Object>,
    pub(crate) streams: v8::Global<v8::Array>,
    pub(crate) transceiver: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcTrackEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCTrackEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcTrackEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCTrackEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_track_event_receiver_property::define(scope, prototype)?;
    super::rtc_track_event_track_property::define(scope, prototype)?;
    super::rtc_track_event_streams_property::define(scope, prototype)?;
    super::rtc_track_event_transceiver_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcTrackEventStore>()
        .ok_or_else(|| "RTCTrackEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    arguments: v8::FunctionCallbackArguments<'s>,
    mut result: v8::ReturnValue<'s>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCTrackEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "RTCTrackEventInit must be an object");
        return;
    };
    let Some(receiver) = required_object(scope, init, "receiver") else {
        return;
    };
    let Some(track) = required_object(scope, init, "track") else {
        return;
    };
    let Some(transceiver) = required_object(scope, init, "transceiver") else {
        return;
    };
    let streams = array_property(scope, init, "streams");
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles = super::event::boolean_property(scope, init, "bubbles");
    let cancelable = super::event::boolean_property(scope, init, "cancelable");
    let composed = super::event::boolean_property(scope, init, "composed");
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    attach(
        scope,
        arguments.this(),
        receiver,
        track,
        streams,
        transceiver,
    );
    result.set(arguments.this().into());
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    event_type: String,
    receiver: v8::Local<'s, v8::Object>,
    track: v8::Local<'s, v8::Object>,
    streams: v8::Local<'s, v8::Array>,
    transceiver: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create RTCTrackEvent".to_owned());
    }
    super::event::attach(scope, event, event_type, false, false, false);
    attach(scope, event, receiver, track, streams, transceiver);
    Ok(event)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
    receiver: v8::Local<'_, v8::Object>,
    track: v8::Local<'_, v8::Object>,
    streams: v8::Local<'_, v8::Array>,
    transceiver: v8::Local<'_, v8::Object>,
) {
    let record = TrackEventRecord {
        receiver: v8::Global::new(scope, receiver),
        track: v8::Global::new(scope, track),
        streams: v8::Global::new(scope, streams),
        transceiver: v8::Global::new(scope, transceiver),
    };
    scope
        .get_slot_mut::<RtcTrackEventStore>()
        .expect("RTCTrackEvent state")
        .records
        .insert(event.get_identity_hash().get(), record);
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    event: v8::Local<'_, v8::Object>,
) -> Option<TrackEventRecord> {
    scope
        .get_slot::<RtcTrackEventStore>()?
        .records
        .get(&event.get_identity_hash().get())
        .cloned()
}

pub(crate) fn required_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    init: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = init.get(scope, key.into());
    let Some(value) = value else {
        crate::webidl::throw_type_error(scope, &format!("Required member '{name}' is undefined"));
        return None;
    };
    match v8::Local::<v8::Object>::try_from(value) {
        Ok(value) if !value.is_null_or_undefined() => Some(value),
        _ => {
            crate::webidl::throw_type_error(scope, &format!("{name} is not a valid object"));
            None
        }
    }
}

pub(crate) fn array_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    init: v8::Local<'s, v8::Object>,
    name: &str,
) -> v8::Local<'s, v8::Array> {
    let Some(key) = v8::String::new(scope, name) else {
        return v8::Array::new(scope, 0);
    };
    init.get(scope, key.into())
        .and_then(|value| v8::Local::<v8::Array>::try_from(value).ok())
        .unwrap_or_else(|| v8::Array::new(scope, 0))
}

pub(crate) fn get_receiver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.receiver).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.track).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_streams(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.streams).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_transceiver(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.transceiver).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
