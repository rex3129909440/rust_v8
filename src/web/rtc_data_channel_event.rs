use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct RtcDataChannelEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Object>>,
    pub(crate) channel_identities: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcDataChannelEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCDataChannelEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcDataChannelEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCDataChannelEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::rtc_data_channel_event_channel_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcDataChannelEventStore>()
        .ok_or_else(|| "RTCDataChannelEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn register_channel(
    scope: &mut v8::PinScope<'_, '_>,
    channel: v8::Local<'_, v8::Object>,
) {
    if let Some(store) = scope.get_slot_mut::<RtcDataChannelEventStore>() {
        store
            .channel_identities
            .insert(channel.get_identity_hash().get());
    }
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 2 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'RTCDataChannelEvent': 2 arguments required",
        );
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(1)) else {
        crate::webidl::throw_type_error(scope, "RTCDataChannelEventInit must be an object");
        return;
    };
    let Some(key) = v8::String::new(scope, "channel") else {
        return;
    };
    let Some(value) = init.get(scope, key.into()) else {
        crate::webidl::throw_type_error(scope, "Required member 'channel' is undefined");
        return;
    };
    let Ok(channel) = v8::Local::<v8::Object>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "channel is not an RTCDataChannel");
        return;
    };
    let is_channel = scope
        .get_slot::<RtcDataChannelEventStore>()
        .is_some_and(|store| {
            store
                .channel_identities
                .contains(&channel.get_identity_hash().get())
        });
    if !is_channel {
        crate::webidl::throw_type_error(scope, "channel is not an RTCDataChannel");
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
    let channel = v8::Global::new(scope, channel);
    scope
        .get_slot_mut::<RtcDataChannelEventStore>()
        .expect("RTCDataChannelEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), channel);
    result.set(arguments.this().into());
}

pub(crate) fn get_channel(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(channel) = scope
        .get_slot::<RtcDataChannelEventStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
    {
        result.set(v8::Local::new(scope, channel).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
