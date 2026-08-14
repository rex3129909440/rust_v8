use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PromiseRejectionEventStore {
    pub(crate) constructors: HashMap<i32, v8::Global<v8::Function>>,
    pub(crate) records: HashMap<i32, PromiseRejectionRecord>,
}

#[derive(Clone)]
pub(crate) struct PromiseRejectionRecord {
    pub(crate) promise: v8::Global<v8::Promise>,
    pub(crate) reason: v8::Global<v8::Value>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PromiseRejectionEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PromiseRejectionEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PromiseRejectionEventStore>()
        .and_then(|store| store.constructors.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PromiseRejectionEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::promise_rejection_event_promise_property::define(scope, prototype)?;
    super::promise_rejection_event_reason_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PromiseRejectionEventStore>()
        .ok_or_else(|| "PromiseRejectionEvent state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PromiseRejectionEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PromiseRejectionEvent': The provided value is not of type 'PromiseRejectionEventInit'.",
        );
        return;
    };
    let Some(promise_value) = property(scope, init, "promise") else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PromiseRejectionEvent': Failed to read the 'promise' property from 'PromiseRejectionEventInit': Required member is undefined.",
        );
        return;
    };
    if promise_value.is_undefined() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PromiseRejectionEvent': Failed to read the 'promise' property from 'PromiseRejectionEventInit': Required member is undefined.",
        );
        return;
    }
    let Ok(promise) = v8::Local::<v8::Promise>::try_from(promise_value) else {
        crate::webidl::throw_type_error(scope, "promise is not a Promise");
        return;
    };
    let reason = property(scope, init, "reason").unwrap_or_else(|| v8::undefined(scope).into());
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        super::event::boolean_property(scope, init, "bubbles"),
        super::event::boolean_property(scope, init, "cancelable"),
        super::event::boolean_property(scope, init, "composed"),
    );
    let record = PromiseRejectionRecord {
        promise: v8::Global::new(scope, promise),
        reason: v8::Global::new(scope, reason),
    };
    scope
        .get_slot_mut::<PromiseRejectionEventStore>()
        .expect("PromiseRejectionEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PromiseRejectionRecord> {
    scope
        .get_slot::<PromiseRejectionEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn get_promise(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.promise).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_reason(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.reason));
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PromiseRejectionEventStore>() {
        store.constructors.remove(&realm_id);
    }
}
