use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcTransformEventStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcTransformEventStore::default());
}

pub(crate) fn install_in_worker_realm(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure(scope)?;
    crate::webidl::define_global(scope, "RTCTransformEvent", constructor.into())
}

fn ensure<'s>(scope: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<RtcTransformEventStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let parent = global_function(scope, "Event")?;
    let constructor = crate::webidl::create_function(
        scope,
        "RTCTransformEvent",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_transform_event_transformer_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcTransformEventStore>()
        .ok_or_else(|| "RTCTransformEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn global_function<'s>(
    scope: &v8::PinScope<'s, '_>,
    name: &str,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let key = crate::webidl::string(scope, name)?;
    let value = scope
        .get_current_context()
        .global(scope)
        .get(scope, key.into())
        .ok_or_else(|| format!("{name} is unavailable"))?;
    v8::Local::<v8::Function>::try_from(value).map_err(|_| format!("{name} is not a function"))
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
    transformer: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create RTCTransformEvent".to_owned());
    }
    super::event::attach(scope, event, "rtctransform".to_owned(), false, false, false);
    let transformer = v8::Global::new(scope, transformer);
    scope
        .get_slot_mut::<RtcTransformEventStore>()
        .ok_or_else(|| "RTCTransformEvent state was not prepared".to_owned())?
        .records
        .insert(event.get_identity_hash().get(), transformer);
    Ok(event)
}

pub(crate) fn get_transformer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<RtcTransformEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, value).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<RtcTransformEventStore>() {
        store.constructor.remove(realm_id);
    }
}
