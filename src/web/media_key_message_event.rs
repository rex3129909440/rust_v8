use std::collections::HashMap;
#[derive(Clone)]
pub(crate) struct MessageRecord {
    pub(crate) kind: String,
    pub(crate) message: v8::Global<v8::ArrayBuffer>,
}
#[derive(Default)]
pub(crate) struct MediaKeyMessageEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MessageRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MediaKeyMessageEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MediaKeyMessageEvent", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<MediaKeyMessageEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MediaKeyMessageEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::media_key_message_event_message_type_property::define(scope, prototype)?;
    super::media_key_message_event_message_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<MediaKeyMessageEventStore>()
        .ok_or_else(|| "MediaKeyMessageEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
pub(crate) fn member<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> v8::Local<'s, v8::Value> {
    v8::String::new(s, n)
        .and_then(|k| o.get(s, k.into()))
        .unwrap_or_else(|| v8::undefined(s).into())
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'MediaKeyMessageEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(event_type) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(1)) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'MediaKeyMessageEvent': The provided value is not of type 'MediaKeyMessageEventInit'.",
        );
        return;
    };
    if member(s, init, "message").is_undefined() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'MediaKeyMessageEvent': Failed to read the 'message' property from 'MediaKeyMessageEventInit': Required member is undefined.",
        );
        return;
    }
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), event_type, bubbles, cancelable, composed);
    let kind = crate::webidl::value_to_string(s, member(s, init, "messageType"));
    let message = v8::Local::<v8::ArrayBuffer>::try_from(member(s, init, "message"))
        .unwrap_or_else(|_| {
            let backing = v8::ArrayBuffer::new_backing_store_from_vec(Vec::new()).make_shared();
            v8::ArrayBuffer::with_backing_store(s, &backing)
        });
    let message = v8::Global::new(s, message);
    s.get_slot_mut::<MediaKeyMessageEventStore>()
        .expect("MediaKeyMessageEvent state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            MessageRecord { kind, message },
        );
    r.set(a.this().into())
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<MessageRecord> {
    s.get_slot::<MediaKeyMessageEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_message_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(value) = v8::String::new(s, &v.kind)
    {
        r.set(value.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.message).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
