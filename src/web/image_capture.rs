use std::collections::HashMap;
#[derive(Clone)]
struct CaptureRecord {
    track: v8::Global<v8::Object>,
    photos: u64,
}
#[derive(Default)]
pub(crate) struct ImageCaptureStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CaptureRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageCaptureStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageCapture", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<ImageCaptureStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageCapture",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "track", get_track)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getPhotoCapabilities",
        0,
        get_photo_capabilities,
    )?;
    crate::webidl::define_method(scope, prototype, "getPhotoSettings", 0, get_photo_settings)?;
    crate::webidl::define_method(scope, prototype, "grabFrame", 0, grab_frame)?;
    crate::webidl::define_method(scope, prototype, "takePhoto", 0, take_photo)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<ImageCaptureStore>()
        .ok_or_else(|| "ImageCapture state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "ImageCapture requires a MediaStreamTrack");
        return;
    }
    let Ok(track) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "track must be an object");
        return;
    };
    let track = v8::Global::new(scope, track);
    scope
        .get_slot_mut::<ImageCaptureStore>()
        .expect("ImageCapture state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            CaptureRecord { track, photos: 0 },
        );
    result.set(arguments.this().into())
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CaptureRecord> {
    scope
        .get_slot::<ImageCaptureStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_track(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.track).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into())
    }
}
fn get_photo_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let object = v8::Object::new(scope);
    let red_eye = v8::String::new(scope, "redEyeReduction").unwrap();
    let never = v8::String::new(scope, "never").unwrap();
    let _ = object.set(scope, red_eye.into(), never.into());
    resolve(scope, object.into(), result)
}
fn get_photo_settings(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    resolve(scope, v8::Object::new(scope).into(), result)
}
fn grab_frame(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(bitmap) = super::image_bitmap::create(scope, 1, 1, vec![0, 0, 0, 0]) {
        resolve(scope, bitmap.into(), result)
    }
}
fn take_photo(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot_mut::<ImageCaptureStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) {
        record.photos += 1
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Ok(blob) = super::blob::create(scope, Vec::new(), "image/png") {
        resolve(scope, blob.into(), result)
    }
}
