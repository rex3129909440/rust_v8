use std::collections::HashMap;

#[derive(Clone)]
struct PublicKeyRecord {
    raw_id: Vec<u8>,
    response: v8::Global<v8::Object>,
    attachment: Option<String>,
}

#[derive(Default)]
pub(crate) struct PublicKeyCredentialStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PublicKeyRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PublicKeyCredentialStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PublicKeyCredential", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<PublicKeyCredentialStore>()
        .and_then(|s| s.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PublicKeyCredential",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rawId", get_raw_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "response", get_response)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "authenticatorAttachment",
        get_attachment,
    )?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getClientExtensionResults",
        0,
        get_client_extension_results,
    )?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::credential::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "getClientCapabilities",
        0,
        get_client_capabilities,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "isConditionalMediationAvailable",
        0,
        true_promise,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "isUserVerifyingPlatformAuthenticatorAvailable",
        0,
        true_promise,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "parseCreationOptionsFromJSON",
        1,
        identity,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "parseRequestOptionsFromJSON",
        1,
        identity,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "signalAllAcceptedCredentials",
        1,
        void_promise,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "signalCurrentUserDetails",
        1,
        void_promise,
    )?;
    crate::webidl::define_method(
        scope,
        constructor.into(),
        "signalUnknownCredential",
        1,
        void_promise,
    )?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PublicKeyCredentialStore>()
        .ok_or_else(|| "PublicKeyCredential state missing".to_owned())?
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
    id: String,
    raw_id: Vec<u8>,
    response: v8::Local<'s, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create PublicKeyCredential".to_owned());
    }
    super::credential::attach(scope, object, id, "public-key".to_owned());
    let response = v8::Global::new(scope, response);
    scope
        .get_slot_mut::<PublicKeyCredentialStore>()
        .ok_or_else(|| "PublicKeyCredential state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            PublicKeyRecord {
                raw_id,
                response,
                attachment: Some("platform".to_owned()),
            },
        );
    Ok(object)
}
fn record(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>) -> Option<PublicKeyRecord> {
    s.get_slot::<PublicKeyCredentialStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn get_raw_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(v) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let b = v8::ArrayBuffer::new_backing_store_from_vec(v.raw_id).make_shared();
    r.set(v8::ArrayBuffer::with_backing_store(s, &b).into())
}
fn get_response(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        r.set(v8::Local::new(s, &v.response).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_attachment(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        if let Some(text) = v.attachment
            && let Some(value) = v8::String::new(s, &text)
        {
            r.set(value.into())
        } else {
            r.set(v8::null(s).into())
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_client_extension_results(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if record(s, a.this()).is_some() {
        r.set(v8::Object::new(s).into())
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
    if let (Some(k), Some(value)) = (v8::String::new(s, "type"), v8::String::new(s, "public-key")) {
        let _ = o.set(s, k.into(), value.into());
    }
    let b = v8::ArrayBuffer::new_backing_store_from_vec(v.raw_id).make_shared();
    if let Some(k) = v8::String::new(s, "rawId") {
        let _ = o.set(
            s,
            k.into(),
            v8::ArrayBuffer::with_backing_store(s, &b).into(),
        );
    }
    r.set(o.into())
}
fn resolve(
    s: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Ok(p) = super::writable_stream::resolved_promise(s, value) {
        r.set(p.into())
    }
}
fn get_client_capabilities(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = v8::Object::new(s);
    resolve(s, value.into(), r)
}
fn true_promise(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = v8::Boolean::new(s, true);
    resolve(s, value.into(), r)
}
fn identity(
    _: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    r.set(a.get(0))
}
fn void_promise(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    let value = v8::undefined(s);
    resolve(s, value.into(), r)
}
