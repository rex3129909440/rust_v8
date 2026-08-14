use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub(crate) struct PaymentRequestUpdateEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
    pub(crate) updates: HashMap<i32, v8::Global<v8::Value>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PaymentRequestUpdateEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure_constructor(s)?;
    crate::webidl::define_global(s, "PaymentRequestUpdateEvent", c.into())
}
pub(crate) fn ensure_constructor<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PaymentRequestUpdateEventStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PaymentRequestUpdateEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::payment_request_update_event_update_with::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PaymentRequestUpdateEventStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
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
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(
        s,
        a.this(),
        crate::webidl::value_to_string(s, a.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    s.get_slot_mut::<PaymentRequestUpdateEventStore>()
        .expect("PaymentRequestUpdateEvent state")
        .instances
        .insert(a.this().get_identity_hash().get());
    r.set(a.this().into())
}
pub(crate) fn attach(
    s: &mut v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    event_type: String,
    bubbles: bool,
    cancelable: bool,
    composed: bool,
) {
    super::event::attach(s, o, event_type, bubbles, cancelable, composed);
    s.get_slot_mut::<PaymentRequestUpdateEventStore>()
        .expect("PaymentRequestUpdateEvent state")
        .instances
        .insert(o.get_identity_hash().get());
}
pub(crate) fn is_instance(s: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    s.get_slot::<PaymentRequestUpdateEventStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}
pub(crate) fn update_with(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let valid = s
        .get_slot::<PaymentRequestUpdateEventStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&a.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let v = v8::Global::new(s, a.get(0));
    s.get_slot_mut::<PaymentRequestUpdateEventStore>()
        .expect("PaymentRequestUpdateEvent state")
        .updates
        .insert(a.this().get_identity_hash().get(), v);
}
