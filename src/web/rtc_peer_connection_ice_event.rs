use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcPeerConnectionIceEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, Option<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcPeerConnectionIceEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCPeerConnectionIceEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcPeerConnectionIceEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCPeerConnectionIceEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_peer_connection_ice_event_candidate_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcPeerConnectionIceEventStore>()
        .ok_or_else(|| "RTCPeerConnectionIceEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCPeerConnectionIceEvent': 1 argument required",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let candidate = init
        .and_then(|object| property(scope, object, "candidate"))
        .and_then(|value| {
            if value.is_null_or_undefined() {
                None
            } else {
                v8::Local::<v8::Object>::try_from(value).ok()
            }
        })
        .map(|candidate| v8::Global::new(scope, candidate));
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let bubbles =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "bubbles"));
    let cancelable =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "cancelable"));
    let composed =
        init.is_some_and(|object| super::event::boolean_property(scope, object, "composed"));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    scope
        .get_slot_mut::<RtcPeerConnectionIceEventStore>()
        .expect("RTCPeerConnectionIceEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), candidate);
    result.set(arguments.this().into());
}

pub(crate) fn get_candidate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<RtcPeerConnectionIceEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    match value {
        Some(Some(candidate)) => result.set(v8::Local::new(scope, &candidate).into()),
        Some(None) => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}

pub(crate) fn property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'s, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}
