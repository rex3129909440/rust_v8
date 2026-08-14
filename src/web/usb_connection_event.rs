use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct UsbConnectionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, v8::Global<v8::Object>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbConnectionEventStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBConnectionEvent", c.into())
}
pub(crate) fn ensure<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbConnectionEventStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBConnectionEvent",
        2,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    super::usb_connection_event_device_property::define(s, p)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbConnectionEventStore>()
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
            "Failed to construct 'USBConnectionEvent': 2 arguments required, but only 1 present.",
        );
        return;
    }
    let Some(t) = crate::webidl::dom_string(s, a.get(0)) else {
        return;
    };
    let Ok(init) = v8::Local::<v8::Object>::try_from(a.get(1)) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBConnectionEvent': The provided value is not of type 'USBConnectionEventInit'.",
        );
        return;
    };
    let Some(device_value) = member(s, init, "device").filter(|value| !value.is_undefined()) else {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBConnectionEvent': Failed to read the 'device' property from 'USBConnectionEventInit': Required member is undefined.",
        );
        return;
    };
    let Ok(device) = v8::Local::<v8::Object>::try_from(device_value) else {
        crate::webidl::throw_type_error(s, "device is not a USBDevice");
        return;
    };
    let (bubbles, cancelable, composed) = super::event::event_init(s, a.get(1));
    super::event::attach(s, a.this(), t, bubbles, cancelable, composed);
    let device = v8::Global::new(s, device);
    s.get_slot_mut::<UsbConnectionEventStore>()
        .unwrap()
        .records
        .insert(a.this().get_identity_hash().get(), device);
    r.set(a.this().into())
}
pub(crate) fn device(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot::<UsbConnectionEventStore>()
        .and_then(|x| x.records.get(&a.this().get_identity_hash().get()))
        .cloned()
    {
        r.set(v8::Local::new(s, &v).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbConnectionEventStore>() {
        store.constructor.remove(realm_id);
    }
}
