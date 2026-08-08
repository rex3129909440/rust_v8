use std::collections::HashMap;
#[derive(Clone)]
struct DigitalRecord {
    protocol: String,
    data: v8::Global<v8::Value>,
}
#[derive(Default)]
pub(crate) struct DigitalCredentialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, DigitalRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(DigitalCredentialStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "DigitalCredential", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<DigitalCredentialStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "DigitalCredential",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "protocol", protocol)?;
    crate::webidl::define_readonly_accessor(s, p, "data", data)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::credential::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    crate::webidl::define_method(
        s,
        c.into(),
        "userAgentAllowsProtocol",
        1,
        user_agent_allows_protocol,
    )?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<DigitalCredentialStore>()
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
    protocol: String,
    data: v8::Local<'s, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create DigitalCredential".to_owned());
    }
    super::credential::attach(s, o, id, "digital".to_owned());
    let data = v8::Global::new(s, data);
    s.get_slot_mut::<DigitalCredentialStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            DigitalRecord { protocol, data },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<DigitalRecord> {
    s.get_slot::<DigitalCredentialStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.protocol)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.data))
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    if let (Some(k), Some(x)) = (
        v8::String::new(s, "protocol"),
        v8::String::new(s, &v.protocol),
    ) {
        let _ = o.set(s, k.into(), x.into());
    }
    if let Some(k) = v8::String::new(s, "data") {
        let x = v8::Local::new(s, &v.data);
        let _ = o.set(s, k.into(), x);
    }
    r.set(o.into())
}
fn user_agent_allows_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let name = crate::webidl::value_to_string(s, a.get(0));
    let allowed = !name.trim().is_empty();
    let value = v8::Boolean::new(s, allowed);
    if let Ok(p) = super::writable_stream::resolved_promise(s, value.into()) {
        r.set(p.into())
    }
}
