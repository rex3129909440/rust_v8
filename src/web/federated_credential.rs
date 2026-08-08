use std::collections::HashMap;

#[derive(Clone)]
struct FederatedCredentialRecord {
    provider: String,
    protocol: Option<String>,
    name: String,
    icon_url: String,
}

#[derive(Default)]
pub(crate) struct FederatedCredentialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, FederatedCredentialRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FederatedCredentialStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FederatedCredential", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FederatedCredentialStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::credential::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "FederatedCredential",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "provider", get_provider)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "protocol", get_protocol)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "iconURL", get_icon_url)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FederatedCredentialStore>()
        .ok_or_else(|| "FederatedCredential state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}
fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object.get(scope, key.into())
}
fn text(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>, name: &str) -> String {
    member(scope, object, name)
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value))
        .unwrap_or_default()
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "FederatedCredential must be constructed");
        return;
    }
    let Ok(options) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "FederatedCredential requires options");
        return;
    };
    let id = text(scope, options, "id");
    let provider = text(scope, options, "provider");
    if id.is_empty() || provider.is_empty() {
        crate::webidl::throw_type_error(scope, "id and provider are required");
        return;
    }
    super::credential::attach(scope, arguments.this(), id, "federated".to_owned());
    let protocol = member(scope, options, "protocol")
        .filter(|value| !value.is_undefined())
        .map(|value| crate::webidl::value_to_string(scope, value));
    let name = text(scope, options, "name");
    let icon_url = text(scope, options, "iconURL");
    scope
        .get_slot_mut::<FederatedCredentialStore>()
        .expect("FederatedCredential state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            FederatedCredentialRecord {
                provider,
                protocol,
                name,
                icon_url,
            },
        );
    result.set(arguments.this().into());
}
fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<FederatedCredentialRecord> {
    scope
        .get_slot::<FederatedCredentialStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    value: impl FnOnce(FederatedCredentialRecord) -> Option<String>,
) {
    let Some(value) = record(scope, arguments.this()).and_then(value) else {
        result.set(v8::null(scope).into());
        return;
    };
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}
fn get_provider(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| Some(x.provider));
}
fn get_protocol(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| x.protocol);
}
fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| Some(x.name));
}
fn get_icon_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_text(s, a, r, |x| Some(x.icon_url));
}
