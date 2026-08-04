use std::collections::HashMap;
#[derive(Clone)]
struct IdentityRecord {
    token: String,
    auto_selected: bool,
    config_url: String,
}
#[derive(Default)]
pub(crate) struct IdentityCredentialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdentityRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(IdentityCredentialStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "IdentityCredential", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<IdentityCredentialStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "IdentityCredential",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "token", token)?;
    crate::webidl::define_readonly_accessor(s, p, "isAutoSelected", auto_selected)?;
    crate::webidl::finish_constructor(s, p, c)?;
    crate::webidl::define_readonly_accessor(s, p, "configURL", config_url)?;
    let parent = super::credential::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    crate::webidl::define_method(s, c.into(), "disconnect", 1, disconnect)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<IdentityCredentialStore>()
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
    token: String,
    config_url: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create IdentityCredential".to_owned());
    }
    super::credential::attach(s, o, id, "identity".to_owned());
    s.get_slot_mut::<IdentityCredentialStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            IdentityRecord {
                token,
                auto_selected: false,
                config_url,
            },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<IdentityRecord> {
    s.get_slot::<IdentityCredentialStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    f: impl FnOnce(IdentityRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &f(v))
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn token(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.token)
}
fn config_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.config_url)
}
fn auto_selected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, v.auto_selected).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn disconnect(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let x = v8::undefined(s);
    if let Ok(p) = super::writable_stream::resolved_promise(s, x.into()) {
        r.set(p.into())
    }
}
