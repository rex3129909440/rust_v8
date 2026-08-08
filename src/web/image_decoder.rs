use std::collections::HashMap;
#[derive(Clone)]
struct DecoderRecord {
    media_type: String,
    closed: bool,
    decode_count: u64,
    completed: v8::Global<v8::Promise>,
    tracks: v8::Global<v8::Object>,
}
#[derive(Default)]
pub(crate) struct ImageDecoderStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DecoderRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageDecoderStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageDecoder", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<ImageDecoderStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageDecoder",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "complete", get_complete)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "completed", get_completed)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "tracks", get_tracks)?;
    crate::webidl::define_method(scope, prototype, "close", 0, close)?;
    crate::webidl::define_method(scope, prototype, "decode", 0, decode)?;
    crate::webidl::define_method(scope, prototype, "reset", 0, reset)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let is_type_supported = crate::webidl::create_function(
        scope,
        "isTypeSupported",
        1,
        v8::ConstructorBehavior::Throw,
        is_type_supported,
    )?;
    let key = crate::webidl::string(scope, "isTypeSupported")?;
    let _ = constructor.define_own_property(
        scope,
        key.into(),
        is_type_supported.into(),
        v8::PropertyAttribute::NONE,
    );
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ImageDecoderStore>()
        .ok_or_else(|| "ImageDecoder state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> v8::Local<'s, v8::Value> {
    crate::webidl::string(scope, name)
        .ok()
        .and_then(|key| object.get(scope, key.into()))
        .unwrap_or_else(|| v8::undefined(scope).into())
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ImageDecoder requires an init dictionary");
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "ImageDecoder init must be an object");
        return;
    };
    let media_type = crate::webidl::value_to_string(scope, member(scope, init, "type"));
    if media_type.is_empty() {
        crate::webidl::throw_type_error(scope, "ImageDecoder type is required");
        return;
    }
    let tracks = match super::image_track_list::create(scope) {
        Ok(v) => v,
        Err(message) => {
            crate::webidl::throw_type_error(scope, &message);
            return;
        }
    };
    let completed =
        match super::writable_stream::resolved_promise(scope, v8::undefined(scope).into()) {
            Ok(v) => v,
            Err(_) => return,
        };
    let record = DecoderRecord {
        media_type,
        closed: false,
        decode_count: 0,
        completed: v8::Global::new(scope, completed),
        tracks: v8::Global::new(scope, tracks),
    };
    scope
        .get_slot_mut::<ImageDecoderStore>()
        .expect("ImageDecoder state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into())
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DecoderRecord> {
    scope
        .get_slot::<ImageDecoderStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.media_type)
    {
        result.set(value.into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_complete(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, !record.closed).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_completed(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.completed).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_tracks(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.tracks).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn close(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<ImageDecoderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.closed = true
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<ImageDecoderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.decode_count = 0
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn decode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = scope.get_slot_mut::<ImageDecoderStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if record.closed {
        crate::webidl::throw_type_error(scope, "ImageDecoder is closed");
        return;
    }
    record.decode_count += 1;
    let object = v8::Object::new(scope);
    if let Ok(image) = super::image_bitmap::create(scope, 1, 1, vec![0, 0, 0, 0]) {
        let key = v8::String::new(scope, "image").unwrap();
        let _ = object.set(scope, key.into(), image.into());
    }
    let key = v8::String::new(scope, "complete").unwrap();
    let _ = object.set(scope, key.into(), v8::Boolean::new(scope, true).into());
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, object.into()) {
        result.set(promise.into())
    }
}
fn is_type_supported(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let media_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let supported = crate::fingerprint_environment::media_type_matches(
        &crate::fingerprint::edge(scope).media.image_decoder_types,
        &media_type,
    );
    if let Ok(promise) =
        super::writable_stream::resolved_promise(scope, v8::Boolean::new(scope, supported).into())
    {
        result.set(promise.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ImageDecoderStore>() {
        store.constructor.remove(realm_id);
    }
}
