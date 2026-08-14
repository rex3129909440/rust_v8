use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct RtcDtmfToneChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) tones: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcDtmfToneChangeEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCDTMFToneChangeEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcDtmfToneChangeEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCDTMFToneChangeEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_dtmf_tone_change_event_tone_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcDtmfToneChangeEventStore>()
        .ok_or_else(|| "RTCDTMFToneChangeEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
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
            "Failed to construct 'RTCDTMFToneChangeEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    if !arguments.get(1).is_null_or_undefined() && !arguments.get(1).is_object() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCDTMFToneChangeEvent': The provided value is not of type 'RTCDTMFToneChangeEventInit'.",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let tone = init
        .and_then(|init| property_string(scope, init, "tone"))
        .unwrap_or_default();
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        init.is_some_and(|init| super::event::boolean_property(scope, init, "bubbles")),
        init.is_some_and(|init| super::event::boolean_property(scope, init, "cancelable")),
        init.is_some_and(|init| super::event::boolean_property(scope, init, "composed")),
    );
    scope
        .get_slot_mut::<RtcDtmfToneChangeEventStore>()
        .expect("RTCDTMFToneChangeEvent state")
        .tones
        .insert(arguments.this().get_identity_hash().get(), tone);
    result.set(arguments.this().into());
}

pub(crate) fn property_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

pub(crate) fn get_tone(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(tone) = scope
        .get_slot::<RtcDtmfToneChangeEventStore>()
        .and_then(|store| store.tones.get(&arguments.this().get_identity_hash().get()))
    {
        if let Some(value) = v8::String::new(scope, tone) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
