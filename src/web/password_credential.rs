use std::collections::HashMap;
#[derive(Clone)]
struct PasswordRecord {
    password: String,
    name: String,
    icon_url: String,
}
#[derive(Default)]
pub(crate) struct PasswordCredentialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PasswordRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PasswordCredentialStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PasswordCredential", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<PasswordCredentialStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PasswordCredential",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "password", get_password)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "iconURL", get_icon_url)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::credential::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<PasswordCredentialStore>()
        .ok_or_else(|| "PasswordCredential state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn member<'s>(
    s: &v8::PinScope<'s, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
) -> v8::Local<'s, v8::Value> {
    v8::String::new(s, n)
        .and_then(|k| o.get(s, k.into()))
        .unwrap_or_else(|| v8::undefined(s).into())
}
fn construct(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if !a.is_construct_call() || a.length() < 1 {
        crate::webidl::throw_type_error(s, "PasswordCredential requires data");
        return;
    }
    let Ok(data) = v8::Local::<v8::Object>::try_from(a.get(0)) else {
        crate::webidl::throw_type_error(s, "PasswordCredential data must be an object");
        return;
    };
    let id = crate::webidl::value_to_string(s, member(s, data, "id"));
    let password = crate::webidl::value_to_string(s, member(s, data, "password"));
    let name = crate::webidl::value_to_string(s, member(s, data, "name"));
    let icon_url = crate::webidl::value_to_string(s, member(s, data, "iconURL"));
    if id.is_empty() || password.is_empty() {
        crate::webidl::throw_type_error(s, "id and password are required");
        return;
    }
    super::credential::attach(s, a.this(), id, "password".to_owned());
    s.get_slot_mut::<PasswordCredentialStore>()
        .expect("PasswordCredential state")
        .records
        .insert(
            a.this().get_identity_hash().get(),
            PasswordRecord {
                password,
                name,
                icon_url,
            },
        );
    r.set(a.this().into())
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PasswordRecord> {
    s.get_slot::<PasswordCredentialStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn text(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
    pick: impl FnOnce(PasswordRecord) -> String,
) {
    if let Some(v) = record(s, a.this())
        && let Some(value) = v8::String::new(s, &pick(v))
    {
        r.set(value.into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_password(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.password)
}
fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.name)
}
fn get_icon_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    text(s, a, r, |v| v.icon_url)
}
