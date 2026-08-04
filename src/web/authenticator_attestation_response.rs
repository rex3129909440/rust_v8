use std::collections::HashMap;

#[derive(Clone)]
struct AttestationRecord {
    object: Vec<u8>,
    authenticator_data: Vec<u8>,
    public_key: Option<Vec<u8>>,
    algorithm: i32,
    transports: Vec<String>,
}

#[derive(Default)]
pub(crate) struct AuthenticatorAttestationResponseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AttestationRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AuthenticatorAttestationResponseStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(
        scope,
        "AuthenticatorAttestationResponse",
        constructor.into(),
    )
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<AuthenticatorAttestationResponseStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AuthenticatorAttestationResponse",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "attestationObject",
        get_attestation_object,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getAuthenticatorData",
        0,
        get_authenticator_data,
    )?;
    crate::webidl::define_method(scope, prototype, "getPublicKey", 0, get_public_key)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getPublicKeyAlgorithm",
        0,
        get_public_key_algorithm,
    )?;
    crate::webidl::define_method(scope, prototype, "getTransports", 0, get_transports)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::authenticator_response::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AuthenticatorAttestationResponseStore>()
        .ok_or_else(|| "AuthenticatorAttestationResponse state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(s, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    client_data: Vec<u8>,
    object: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let value = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, value, prototype.into()) != Some(true) {
        return Err("cannot create AuthenticatorAttestationResponse".to_owned());
    }
    super::authenticator_response::attach(scope, value, client_data);
    scope
        .get_slot_mut::<AuthenticatorAttestationResponseStore>()
        .ok_or_else(|| "AuthenticatorAttestationResponse state missing".to_owned())?
        .records
        .insert(
            value.get_identity_hash().get(),
            AttestationRecord {
                object,
                authenticator_data: Vec::new(),
                public_key: None,
                algorithm: -7,
                transports: vec!["internal".to_owned()],
            },
        );
    Ok(value)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AttestationRecord> {
    scope
        .get_slot::<AuthenticatorAttestationResponseStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_buffer(
    scope: &mut v8::PinScope<'_, '_>,
    bytes: Vec<u8>,
    mut result: v8::ReturnValue<'_>,
) {
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    result.set(v8::ArrayBuffer::with_backing_store(scope, &backing).into());
}
fn get_attestation_object(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_buffer(s, v.object, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_authenticator_data(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_buffer(s, v.authenticator_data, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_public_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(bytes) = v.public_key {
            return_buffer(s, bytes, r)
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_public_key_algorithm(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Integer::new(s, v.algorithm).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_transports(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(s, v.transports.len() as i32);
    for (i, item) in v.transports.iter().enumerate() {
        if let Some(value) = v8::String::new(s, item) {
            let _ = array.set_index(s, i as u32, value.into());
        }
    }
    r.set(array.into());
}
