use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcErrorEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcErrorEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCErrorEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcErrorEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCErrorEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_error_event_error_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcErrorEventStore>()
        .ok_or_else(|| "RTCErrorEvent state was not prepared".to_owned())?
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
            "Failed to construct 'RTCErrorEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "RTCErrorEventInit must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "error") else {
        return;
    };
    let Some(value) = init.get(scope, key.into()) else {
        crate::webidl::throw_type_error(scope, "Required member 'error' is undefined");
        return;
    };
    let Ok(error) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "error is not an RTCError");
        return;
    };
    if value.is_null_or_undefined() {
        crate::webidl::throw_type_error(scope, "error is not an RTCError");
        return;
    }
    if !super::rtc_error::is_instance(scope, error) {
        crate::webidl::throw_type_error(scope, "error is not an RTCError");
        return;
    }
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        super::event::boolean_property(scope, init, "bubbles"),
        super::event::boolean_property(scope, init, "cancelable"),
        super::event::boolean_property(scope, init, "composed"),
    );
    let error = v8::Global::new(scope, error);
    scope
        .get_slot_mut::<RtcErrorEventStore>()
        .expect("RTCErrorEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), error);
    result.set(arguments.this().into());
}

pub(crate) fn get_error(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(error) = scope.get_slot::<RtcErrorEventStore>().and_then(|store| {
        store
            .records
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Local::new(scope, error).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
