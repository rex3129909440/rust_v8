use std::collections::HashMap;

#[derive(Clone)]
struct AssertionRecord {
    authenticator_data: Vec<u8>,
    signature: Vec<u8>,
    user_handle: Option<Vec<u8>>,
}

#[derive(Default)]
pub(crate) struct AuthenticatorAssertionResponseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AssertionRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AuthenticatorAssertionResponseStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AuthenticatorAssertionResponse", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<AuthenticatorAssertionResponseStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AuthenticatorAssertionResponse",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "authenticatorData",
        get_authenticator_data,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "signature", get_signature)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "userHandle", get_user_handle)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::authenticator_response::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AuthenticatorAssertionResponseStore>()
        .ok_or_else(|| "AuthenticatorAssertionResponse state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    client_data: Vec<u8>,
    authenticator_data: Vec<u8>,
    signature: Vec<u8>,
    user_handle: Option<Vec<u8>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create AuthenticatorAssertionResponse".to_owned());
    }
    super::authenticator_response::attach(scope, object, client_data);
    scope
        .get_slot_mut::<AuthenticatorAssertionResponseStore>()
        .ok_or_else(|| "AuthenticatorAssertionResponse state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AssertionRecord {
                authenticator_data,
                signature,
                user_handle,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AssertionRecord> {
    scope
        .get_slot::<AuthenticatorAssertionResponseStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn buffer(scope: &mut v8::PinScope<'_, '_>, bytes: Vec<u8>, mut result: v8::ReturnValue<'_>) {
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    result.set(v8::ArrayBuffer::with_backing_store(scope, &backing).into());
}

fn get_authenticator_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        buffer(s, v.authenticator_data, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_signature(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        buffer(s, v.signature, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_user_handle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(bytes) = v.user_handle {
            buffer(s, bytes, r)
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
