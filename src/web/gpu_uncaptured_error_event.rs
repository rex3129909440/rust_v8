use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct GpuUncapturedErrorEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) errors: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GpuUncapturedErrorEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "GPUUncapturedErrorEvent", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(value) = scope
        .get_slot::<GpuUncapturedErrorEventStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "GPUUncapturedErrorEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::gpu_uncaptured_error_event_error_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GpuUncapturedErrorEventStore>()
        .ok_or_else(|| "GPUUncapturedErrorEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Please use the 'new' operator");
        return;
    }
    if arguments.length() < 2 {
        crate::webidl::throw_type_error(scope, "2 arguments required");
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "eventInitDict is required");
        return;
    };
    let Some(key) = v8::String::new(scope, "error") else {
        return;
    };
    let Some(value) = init.get(scope, key.into()) else {
        return;
    };
    let Ok(error) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "error must be a GPUError");
        return;
    };
    let object = arguments.this();
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(scope, object, event_type, bubbles, cancelable, composed);
    let persistent = v8::Global::new(scope, error);
    if let Some(store) = scope.get_slot_mut::<GpuUncapturedErrorEventStore>() {
        store
            .errors
            .insert(object.get_identity_hash().get(), persistent);
    }
    result.set(object.into())
}
pub(crate) fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let error = scope
        .get_slot::<GpuUncapturedErrorEventStore>()
        .and_then(|store| {
            store
                .errors
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    if let Some(error) = error {
        result.set(v8::Local::new(scope, &error).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<GpuUncapturedErrorEventStore>() {
        store.constructor.remove(realm_id);
    }
}
