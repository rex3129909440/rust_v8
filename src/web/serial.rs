use std::collections::HashMap;
#[derive(Clone, Default)]
struct SerialData {
    on_connect: Option<v8::Global<v8::Value>>,
    on_disconnect: Option<v8::Global<v8::Value>>,
    ports: Vec<v8::Global<v8::Object>>,
}
#[derive(Default)]
pub(crate) struct SerialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SerialData>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(SerialStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "Serial", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(v) = s
        .get_slot::<SerialStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c =
        crate::webidl::create_function(s, "Serial", 0, v8::ConstructorBehavior::Allow, illegal)?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_accessor(s, p, "onconnect", get_connect, set_connect)?;
    crate::webidl::define_accessor(s, p, "ondisconnect", get_disconnect, set_disconnect)?;
    crate::webidl::define_method(s, p, "getPorts", 0, get_ports)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let tag = v8::Symbol::get_to_string_tag(s);
    let _ = p.delete(s, tag.into());
    crate::webidl::define_method(s, p, "requestPort", 0, request_port)?;
    crate::webidl::define_to_string_tag(s, p, "Serial")?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<SerialStore>()
        .unwrap()
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create Serial".to_owned());
    }
    super::event_target::attach(s, o);
    let configured = crate::fingerprint::edge(s)
        .hardware_devices
        .serial_ports
        .clone();
    let mut ports = Vec::with_capacity(configured.len());
    for profile in configured {
        let port = super::serial_port::create(s, profile)?;
        ports.push(v8::Global::new(s, port));
    }
    s.get_slot_mut::<SerialStore>().unwrap().records.insert(
        o.get_identity_hash().get(),
        SerialData {
            ports,
            ..SerialData::default()
        },
    );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<SerialData> {
    s.get_slot::<SerialStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn promise(s: &mut v8::PinScope<'_, '_>, v: v8::Local<'_, v8::Value>, mut r: v8::ReturnValue<'_>) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, v) {
        r.set(p.into())
    }
}
fn get_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.on_connect),
        r,
    )
}
fn get_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    super::window_event_handler_support::return_handler(
        s,
        record(s, a.this()).and_then(|v| v.on_disconnect),
        r,
    )
}
fn set_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<SerialStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_connect = h
    }
}
fn set_disconnect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<SerialStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.on_disconnect = h
    }
}
fn get_ports(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, v.ports.len() as i32);
    for (i, item) in v.ports.iter().enumerate() {
        let x = v8::Local::new(s, item);
        let _ = array.set_index(s, i as u32, x.into());
    }
    promise(s, array.into(), r)
}
fn request_port(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_none() {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let profile = crate::fingerprint::edge(s)
        .hardware_devices
        .serial_ports
        .first()
        .cloned()
        .unwrap_or_default();
    match super::serial_port::create(s, profile) {
        Ok(port) => {
            let stored = v8::Global::new(s, port);
            if let Some(v) = s
                .get_slot_mut::<SerialStore>()
                .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
            {
                v.ports.push(stored)
            }
            promise(s, port.into(), r)
        }
        Err(e) => crate::webidl::throw_type_error(s, &e),
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<SerialStore>() {
        store.constructor.remove(realm_id);
    }
}
