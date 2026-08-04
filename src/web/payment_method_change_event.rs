use std::collections::HashMap;
#[derive(Clone)]
pub(crate) struct MethodRecord {
    pub(crate) name: String,
    pub(crate) details: v8::Global<v8::Value>,
}
#[derive(Default)]
pub(crate) struct PaymentMethodChangeEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, MethodRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentMethodChangeEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PaymentMethodChangeEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentMethodChangeEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentMethodChangeEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::payment_method_change_event_method_name_property::define(s, p)?;
    super::payment_method_change_event_method_details_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::payment_request_update_event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentMethodChangeEventStore>()
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
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "event type required");
        return;
    }
    let event_type = crate::webidl::value_to_string(s, a.get(0));
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::payment_request_update_event::attach(
        s,
        a.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let init = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let name = init
        .and_then(|o| member(s, o, "methodName"))
        .map(|v| crate::webidl::value_to_string(s, v))
        .unwrap_or_default();
    let details = init
        .and_then(|o| member(s, o, "methodDetails"))
        .unwrap_or_else(|| v8::null(s).into());
    let details = v8::Global::new(s, details);
    s.get_slot_mut::<PaymentMethodChangeEventStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            MethodRecord { name, details },
        );
    r.set(a.this().into())
}
pub(crate) fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<MethodRecord> {
    s.get_slot::<PaymentMethodChangeEventStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
pub(crate) fn method_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.name)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn method_details(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.details))
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
