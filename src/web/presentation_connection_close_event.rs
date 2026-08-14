use std::collections::HashMap;
#[derive(Clone)]
pub(crate) struct CloseRecord {
    pub(crate) reason: String,
    pub(crate) message: String,
}
#[derive(Default)]
pub(crate) struct PresentationConnectionCloseEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, CloseRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PresentationConnectionCloseEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PresentationConnectionCloseEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PresentationConnectionCloseEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PresentationConnectionCloseEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::presentation_connection_close_event_reason_property::define(s, p)?;
    super::presentation_connection_close_event_message_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PresentationConnectionCloseEventStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn member<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'s, v8::Object>,
    n: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let k = v8::String::new(s, n)?;
    o.get(s, k.into())
}
pub(crate) fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 2 {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PresentationConnectionCloseEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(t) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    if !a.get(1).is_object() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PresentationConnectionCloseEvent': The provided value is not of type 'PresentationConnectionCloseEventInit'.",
        );
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    if init
        .and_then(|init| member(s, init, "reason"))
        .is_none_or(|value| value.is_undefined())
    {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'PresentationConnectionCloseEvent': Failed to read the 'reason' property from 'PresentationConnectionCloseEventInit': Required member is undefined.",
        );
        return;
    }
    let reason = init
        .and_then(|o| member(s, o, "reason"))
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_else(|| "error".to_owned());
    let message = init
        .and_then(|o| member(s, o, "message"))
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_default();
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), t, bubbles, cancelable, composed);
    s.get_slot_mut::<PresentationConnectionCloseEventStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            CloseRecord { reason, message },
        );
    r.set(a.this().into())
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<CloseRecord> {
    s.get_slot::<PresentationConnectionCloseEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(CloseRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn reason(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.reason)
}
pub(crate) fn message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.message)
}
