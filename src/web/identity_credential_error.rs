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
        construct,
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
fn string_member(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<String> {
    let key = v8::String::new(scope, name)?;
    let value = object?.get(scope, key.into())?;
    if value.is_undefined() || value.is_null() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() {
        crate::webidl::throw_type_error(
            s,
            "Failed to construct 'IdentityCredentialError': Please use the 'new' operator, this DOM object constructor cannot be called as a function.",
        );
        return;
    }
    let message = if a.get(0).is_undefined() {
        String::new()
    } else {
        crate::webidl::value_to_string(s, a.get(0))
    };
    let options = v8::Local::<v8::Object>::try_from(a.get(1)).ok();
    let code = string_member(s, options, "error").unwrap_or_default();
    let url = string_member(s, options, "url");
    super::dom_exception::attach(
        s,
        a.this(),
        "IdentityCredentialError".to_owned(),
        message,
        0,
    );
    s.get_slot_mut::<IdentityCredentialErrorStore>()
        .expect("IdentityCredentialError state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            IdentityErrorRecord { code, url },
        );
    r.set(a.this().into())
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
            r.set(v8::String::empty(s).into())
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
    if let Some(v) = record(s, a.this()) {
        if let Some(value) = v8::String::new(s, &v.code) {
            r.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
