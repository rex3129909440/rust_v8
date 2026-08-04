use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum KeyAlgorithm {
    Aes { name: String, length: u16 },
    Hmac { hash: String, length: u32 },
    Hkdf,
    Pbkdf2,
}

impl KeyAlgorithm {
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Aes { name, .. } => name,
            Self::Hmac { .. } => "HMAC",
            Self::Hkdf => "HKDF",
            Self::Pbkdf2 => "PBKDF2",
        }
    }
}

#[derive(Clone)]
pub(crate) struct CryptoKeyRecord {
    pub(crate) key_type: String,
    pub(crate) extractable: bool,
    pub(crate) algorithm: KeyAlgorithm,
    pub(crate) usages: Vec<String>,
    pub(crate) material: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct CryptoKeyStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, CryptoKeyRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CryptoKeyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CryptoKey", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CryptoKeyStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CryptoKey",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "extractable", get_extractable)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "algorithm", get_algorithm)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "usages", get_usages)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CryptoKeyStore>()
        .ok_or_else(|| "CryptoKey state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'CryptoKey': Illegal constructor",
    );
}

pub(crate) fn create_secret<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    extractable: bool,
    algorithm: KeyAlgorithm,
    usages: Vec<String>,
    material: Vec<u8>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CryptoKey".to_owned());
    }
    scope
        .get_slot_mut::<CryptoKeyStore>()
        .ok_or_else(|| "CryptoKey state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            CryptoKeyRecord {
                key_type: "secret".to_owned(),
                extractable,
                algorithm,
                usages,
                material,
            },
        );
    Ok(object)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<CryptoKeyRecord> {
    scope
        .get_slot::<CryptoKeyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn record_from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> Option<CryptoKeyRecord> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    record(scope, object)
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this())
        && let Some(value) = v8::String::new(scope, &record.key_type)
    {
        result.set(value.into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_extractable(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.extractable).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_algorithm(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Ok(algorithm) = algorithm_object(scope, &record.algorithm) else {
        return;
    };
    result.set(algorithm.into());
}

fn get_usages(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let usages = v8::Array::new(scope, record.usages.len() as i32);
    for (index, usage) in record.usages.iter().enumerate() {
        let Some(value) = v8::String::new(scope, usage) else {
            return;
        };
        let _ = usages.set_index(scope, index as u32, value.into());
    }
    let _ = usages.set_integrity_level(scope, v8::IntegrityLevel::Frozen);
    result.set(usages.into());
}

fn algorithm_object<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    algorithm: &KeyAlgorithm,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let object = v8::Object::new(scope);
    define_string(scope, object, "name", algorithm.name())?;
    match algorithm {
        KeyAlgorithm::Aes { length, .. } => {
            define_number(scope, object, "length", u32::from(*length))?;
        }
        KeyAlgorithm::Hmac { hash, length } => {
            let hash_object = v8::Object::new(scope);
            define_string(scope, hash_object, "name", hash)?;
            define_value(scope, object, "hash", hash_object.into())?;
            define_number(scope, object, "length", *length)?;
        }
        KeyAlgorithm::Hkdf | KeyAlgorithm::Pbkdf2 => {}
    }
    Ok(object)
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) -> Result<(), String> {
    let value = crate::webidl::string(scope, value)?;
    define_value(scope, object, name, value.into())
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: u32,
) -> Result<(), String> {
    let value = v8::Integer::new_from_unsigned(scope, value);
    define_value(scope, object, name, value.into())
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) -> Result<(), String> {
    let key = crate::webidl::string(scope, name)?;
    if object.define_own_property(scope, key.into(), value, v8::PropertyAttribute::NONE)
        == Some(true)
    {
        Ok(())
    } else {
        Err(format!("cannot define CryptoKey.algorithm.{name}"))
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CryptoKeyStore>() {
        store.constructors.remove(&realm_id);
    }
}
