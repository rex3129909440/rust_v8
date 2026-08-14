use std::collections::HashMap;
#[derive(Clone)]
struct EndpointData {
    number: u32,
    direction: String,
    kind: String,
    packet_size: u32,
}
#[derive(Default)]
pub(crate) struct UsbEndpointStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, EndpointData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(UsbEndpointStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "USBEndpoint", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<UsbEndpointStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "USBEndpoint",
        3,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "endpointNumber", number)?;
    crate::webidl::define_readonly_accessor(s, p, "direction", direction)?;
    crate::webidl::define_readonly_accessor(s, p, "type", kind)?;
    crate::webidl::define_readonly_accessor(s, p, "packetSize", packet_size)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<UsbEndpointStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 3 {
        crate::webidl::throw_type_error(s, "3 arguments required");
        return;
    }
    if a.get(0).is_null_or_undefined() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBEndpoint': parameter 1 is not of type 'USBAlternateInterface'.",
        );
        return;
    }
    let valid_alternate = v8::Local::<v8::Object>::try_from(a.get(0))
        .ok()
        .is_some_and(|object| {
            super::structured_clone::inherits_platform_interface(s, object, "USBAlternateInterface")
        });
    if !valid_alternate {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'USBEndpoint': parameter 1 is not of type 'USBAlternateInterface'.",
        );
        return;
    }
    let number = a.get(1).uint32_value(s).unwrap_or(1);
    let direction = crate::webidl::value_to_string(s, a.get(2));
    s.get_slot_mut::<UsbEndpointStore>()
        .unwrap()
        .records
        .insert(
            a.this().get_identity_hash().get(),
            EndpointData {
                number,
                direction,
                kind: "bulk".to_owned(),
                packet_size: 64,
            },
        );
    r.set(a.this().into())
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    number: u32,
    direction: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create USBEndpoint".to_owned());
    }
    s.get_slot_mut::<UsbEndpointStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            EndpointData {
                number,
                direction,
                kind: "bulk".to_owned(),
                packet_size: 64,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<EndpointData> {
    s.get_slot::<UsbEndpointStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn number(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.number).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn packet_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, v.packet_size).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(EndpointData) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn direction(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.direction)
}
fn kind(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.kind)
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<UsbEndpointStore>() {
        store.constructor.remove(realm_id);
    }
}
