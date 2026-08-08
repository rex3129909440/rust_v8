use std::collections::HashMap;
#[derive(Clone)]
struct IdentityErrorRecord {
    code: String,
    url: Option<String>,
}
#[derive(Default)]
pub(crate) struct IdentityCredentialErrorStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, IdentityErrorRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(IdentityCredentialErrorStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "IdentityCredentialError", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(v) = s
        .get_slot::<IdentityCredentialErrorStore>()
        .and_then(|x| x.constructor.get(crate::webidl::realm_id(s)))
        .cloned()
    {
        return Ok(v8::Local::new(s, &v));
    }
    let c = crate::webidl::create_function(
        s,
        "IdentityCredentialError",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "code", code)?;
    crate::webidl::define_readonly_accessor(s, p, "url", url)?;
    crate::webidl::define_readonly_accessor(s, p, "error", error)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let parent = super::dom_exception::ensure_constructor(s)?;
    crate::webidl::inherit(s, c, parent)?;
    let realm_id = crate::webidl::realm_id(s);
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<IdentityCredentialErrorStore>()
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
    code: String,
    url: Option<String>,
    message: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create IdentityCredentialError".to_owned());
    }
    super::dom_exception::attach(s, o, "IdentityCredentialError".to_owned(), message, 0);
    s.get_slot_mut::<IdentityCredentialErrorStore>()
        .unwrap()
        .records
        .insert(
            o.get_identity_hash().get(),
            IdentityErrorRecord { code, url },
        );
    Ok(o)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<IdentityErrorRecord> {
    s.get_slot::<IdentityCredentialErrorStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn code(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this())
        && let Some(x) = v8::String::new(s, &v.code)
    {
        r.set(x.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(x) = v.url.and_then(|x| v8::String::new(s, &x)) {
            r.set(x.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn error(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(a.this().into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
