use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcEncodedVideoFrameStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, VideoFrameRecord>,
}

#[derive(Clone)]
struct VideoFrameRecord {
    frame_type: String,
    timestamp: f64,
    data: v8::Global<v8::Value>,
    metadata: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcEncodedVideoFrameStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCEncodedVideoFrame", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<RtcEncodedVideoFrameStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCEncodedVideoFrame",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timestamp", get_timestamp)?;
    crate::webidl::define_accessor(scope, prototype, "data", get_data, set_data)?;
    crate::webidl::define_method(scope, prototype, "getMetadata", 0, get_metadata)?;
    crate::webidl::define_method(scope, prototype, "toString", 0, to_string)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcEncodedVideoFrameStore>()
        .ok_or_else(|| "RTCEncodedVideoFrame state was not prepared".to_owned())?
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
            "Failed to construct 'RTCEncodedVideoFrame': 1 argument required",
        );
        return;
    }
    let Ok(source) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'RTCEncodedVideoFrame'");
        return;
    };
    let Some(record) = record(scope, source) else {
        crate::webidl::throw_type_error(scope, "parameter 1 is not of type 'RTCEncodedVideoFrame'");
        return;
    };
    scope
        .get_slot_mut::<RtcEncodedVideoFrameStore>()
        .expect("RTCEncodedVideoFrame state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    frame_type: String,
    timestamp: f64,
    data: v8::Local<'_, v8::Value>,
    metadata: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let frame = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, frame, prototype.into()) != Some(true) {
        return Err("cannot create RTCEncodedVideoFrame".to_owned());
    }
    let record = VideoFrameRecord {
        frame_type,
        timestamp,
        data: v8::Global::new(scope, data),
        metadata: v8::Global::new(scope, metadata),
    };
    scope
        .get_slot_mut::<RtcEncodedVideoFrameStore>()
        .ok_or_else(|| "RTCEncodedVideoFrame state was not prepared".to_owned())?
        .records
        .insert(frame.get_identity_hash().get(), record);
    Ok(frame)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<VideoFrameRecord> {
    scope
        .get_slot::<RtcEncodedVideoFrameStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.frame_type) {
            result.set(value.into());
        }
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
        result.set(v8::Number::new(scope, record.timestamp).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.data));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn set_data(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Global::new(scope, arguments.get(0));
    if let Some(record) = scope
        .get_slot_mut::<RtcEncodedVideoFrameStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    {
        record.data = value;
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_metadata(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.metadata).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn to_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "[object RTCEncodedVideoFrame]") {
        result.set(value.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RtcEncodedVideoFrameStore>() {
        store.constructor.remove(realm_id);
    }
}
