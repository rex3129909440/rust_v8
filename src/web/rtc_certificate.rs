use std::collections::HashMap;
#[derive(Default)]
pub(crate) struct RtcCertificateStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, CertificateRecord>,
}

#[derive(Clone)]
struct CertificateRecord {
    expires: f64,
    algorithm: String,
    fingerprint: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(RtcCertificateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "RTCCertificate", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<RtcCertificateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "RTCCertificate",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "expires", get_expires)?;
    crate::webidl::define_method(scope, prototype, "getFingerprints", 0, get_fingerprints)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<RtcCertificateStore>()
        .ok_or_else(|| "RTCCertificate state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'RTCCertificate': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    algorithm_value: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let certificate = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, certificate, prototype.into()) != Some(true) {
        return Err("cannot create RTCCertificate".to_owned());
    }
    let requested = algorithm_name(scope, algorithm_value);
    let algorithm = if requested.eq_ignore_ascii_case("sha-384") {
        "sha-384".to_owned()
    } else {
        "sha-256".to_owned()
    };
    let now = crate::determinism::date_epoch_milliseconds(scope);
    let record = CertificateRecord {
        expires: now + 2_592_000_000.0,
        algorithm,
        fingerprint:
            "00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:10:21:32:43:54:65:76:87:98:A9:BA:CB:DC:ED:FE:0F"
                .to_owned(),
    };
    scope
        .get_slot_mut::<RtcCertificateStore>()
        .ok_or_else(|| "RTCCertificate state was not prepared".to_owned())?
        .records
        .insert(certificate.get_identity_hash().get(), record);
    Ok(certificate)
}

fn algorithm_name(scope: &v8::PinScope<'_, '_>, value: v8::Local<'_, v8::Value>) -> String {
    if let Ok(object) = v8::Local::<v8::Object>::try_from(value) {
        if let Some(key) = v8::String::new(scope, "name") {
            if let Some(name) = object.get(scope, key.into()) {
                return crate::webidl::value_to_string(scope, name);
            }
        }
    }
    crate::webidl::value_to_string(scope, value)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CertificateRecord> {
    scope
        .get_slot::<RtcCertificateStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_expires(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.expires).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    let Some(key) = v8::String::new(scope, name) else {
        return;
    };
    let Some(value) = v8::String::new(scope, value) else {
        return;
    };
    let _ =
        object.define_own_property(scope, key.into(), value.into(), v8::PropertyAttribute::NONE);
}

fn get_fingerprints(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let entry = v8::Object::new(scope);
    define_string(scope, entry, "algorithm", &record.algorithm);
    define_string(scope, entry, "value", &record.fingerprint);
    let fingerprints = v8::Array::new(scope, 1);
    let _ = fingerprints.set_index(scope, 0, entry.into());
    result.set(fingerprints.into());
}
