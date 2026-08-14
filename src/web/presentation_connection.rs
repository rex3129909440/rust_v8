use std::collections::HashMap;
#[derive(Clone)]
struct ConnectionRecord {
    id: String,
    url: String,
    state: String,
    on_connect: Option<v8::Global<v8::Value>>,
    on_close: Option<v8::Global<v8::Value>>,
    on_terminate: Option<v8::Global<v8::Value>>,
    binary_type: String,
    on_message: Option<v8::Global<v8::Value>>,
    messages: Vec<v8::Global<v8::Value>>,
}
#[derive(Default)]
pub(crate) struct PresentationConnectionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ConnectionRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PresentationConnectionStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PresentationConnection", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<PresentationConnectionStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "PresentationConnection",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "id", id)?;
    crate::webidl::define_readonly_accessor(s, p, "url", url)?;
    crate::webidl::define_readonly_accessor(s, p, "state", state)?;
    crate::webidl::define_accessor(s, p, "onconnect", get_connect, set_connect)?;
    crate::webidl::define_accessor(s, p, "onclose", get_close, set_close)?;
    crate::webidl::define_accessor(s, p, "onterminate", get_terminate, set_terminate)?;
    crate::webidl::define_accessor(s, p, "binaryType", get_binary_type, set_binary_type)?;
    crate::webidl::define_accessor(s, p, "onmessage", get_message, set_message)?;
    crate::webidl::define_method(s, p, "close", 0, close)?;
    crate::webidl::define_method(s, p, "send", 1, send)?;
    crate::webidl::define_method(s, p, "terminate", 0, terminate)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::event_target::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PresentationConnectionStore>()
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
    id: String,
    url: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PresentationConnection".to_owned());
    }
    super::event_target::attach(s, o);
    s.get_slot_mut::<PresentationConnectionStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            ConnectionRecord {
                id,
                url,
                state: "connected".to_owned(),
                on_connect: None,
                on_close: None,
                on_terminate: None,
                binary_type: "arraybuffer".to_owned(),
                on_message: None,
                messages: Vec::new(),
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<ConnectionRecord> {
    s.get_slot::<PresentationConnectionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(ConnectionRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn id(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    text(s, a, r, |v| v.id)
}
fn url(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    text(s, a, r, |v| v.url)
}
fn state(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.state)
}
fn get_binary_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.binary_type)
}
fn handler(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
    n: u8,
) {
    let Some(record) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let h = match n {
        0 => record.on_connect,
        1 => record.on_close,
        2 => record.on_terminate,
        _ => record.on_message,
    };
    super::window_event_handler_support::return_handler(s, h, r)
}
fn get_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 0)
}
fn get_close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 1)
}
fn get_terminate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 2)
}
fn get_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    handler(s, a, r, 3)
}
fn set_handler(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, n: u8) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PresentationConnectionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        match n {
            0 => v.on_connect = h,
            1 => v.on_close = h,
            2 => v.on_terminate = h,
            _ => v.on_message = h,
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn set_connect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 0)
}
fn set_close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 1)
}
fn set_terminate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 2)
}
fn set_message(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    set_handler(s, a, 3)
}
fn set_binary_type(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PresentationConnectionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.binary_type = value
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn close(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<PresentationConnectionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.state = "closed".to_owned()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn terminate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if let Some(v) = s
        .get_slot_mut::<PresentationConnectionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.state = "terminated".to_owned()
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn send(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let message = v8::Global::new(s, a.get(0));
    if let Some(v) = s
        .get_slot_mut::<PresentationConnectionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.messages.push(message)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
