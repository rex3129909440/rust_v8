use std::collections::HashMap;
#[derive(Clone)]
struct TrackListRecord {
    track: v8::Global<v8::Object>,
    ready: v8::Global<v8::Promise>,
}
#[derive(Default)]
pub(crate) struct ImageTrackListStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TrackListRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ImageTrackListStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ImageTrackList", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<ImageTrackListStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ImageTrackList",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "selectedIndex", get_selected_index)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "selectedTrack", get_selected_track)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "ready", get_ready)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let iterator =
        crate::webidl::create_function(scope, "values", 0, v8::ConstructorBehavior::Throw, values)?;
    let iterator_key = v8::Symbol::get_iterator(scope);
    if prototype.define_own_property(
        scope,
        iterator_key.into(),
        iterator.into(),
        v8::PropertyAttribute::DONT_ENUM,
    ) != Some(true)
    {
        return Err("cannot define ImageTrackList iterator".to_owned());
    }
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ImageTrackListStore>()
        .ok_or_else(|| "ImageTrackList state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn illegal(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ImageTrackList".to_owned());
    }
    let track = super::image_track::create(scope)?;
    let ready = super::writable_stream::resolved_promise(scope, v8::undefined(scope).into())?;
    let record = TrackListRecord {
        track: v8::Global::new(scope, track),
        ready: v8::Global::new(scope, ready),
    };
    scope
        .get_slot_mut::<ImageTrackListStore>()
        .ok_or_else(|| "ImageTrackList state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<TrackListRecord> {
    scope
        .get_slot::<ImageTrackListStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn get_length(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new(s, 1).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_selected_index(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Integer::new(s, 0).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_selected_track(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.track).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_ready(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.ready).into())
    } else {
        if let Some(promise) = crate::webidl::rejected_type_error_promise(
            s,
            "Failed to read the 'ready' property from 'ImageTrackList': Illegal invocation",
        ) {
            r.set(promise.into())
        }
    }
}

fn values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(state) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let tracks = v8::Array::new(scope, 1);
    let track = v8::Local::new(scope, &state.track);
    let _ = tracks.set_index(scope, 0, track.into());
    let iterator_key = v8::Symbol::get_iterator(scope);
    let Some(iterator) = tracks.get(scope, iterator_key.into()) else {
        return;
    };
    let Ok(iterator) = v8::Local::<v8::Function>::try_from(iterator) else {
        return;
    };
    if let Some(value) = iterator.call(scope, tracks.into(), &[]) {
        result.set(value);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<ImageTrackListStore>() {
        store.constructor.remove(realm_id);
    }
}
