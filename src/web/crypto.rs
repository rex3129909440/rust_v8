use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CryptoStore {
    constructors: HashMap<i32, v8::Global<v8::Function>>,
    records: HashMap<i32, CryptoRecord>,
}

#[derive(Clone)]
struct CryptoRecord {
    subtle: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CryptoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Crypto", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<CryptoStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Crypto",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getRandomValues", 1, get_random_values)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "subtle", get_subtle)?;
    crate::webidl::define_method(scope, prototype, "randomUUID", 0, random_uuid)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CryptoStore>()
        .ok_or_else(|| "Crypto state was not prepared".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Crypto".to_owned());
    }
    let subtle = super::subtle_crypto::create(scope)?;
    let subtle = v8::Global::new(scope, subtle);
    scope
        .get_slot_mut::<CryptoStore>()
        .ok_or_else(|| "Crypto state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), CryptoRecord { subtle });
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn integer_array(value: v8::Local<'_, v8::Value>) -> bool {
    value.is_int8_array()
        || value.is_uint8_array()
        || value.is_uint8_clamped_array()
        || value.is_int16_array()
        || value.is_uint16_array()
        || value.is_int32_array()
        || value.is_uint32_array()
        || value.is_big_int64_array()
        || value.is_big_uint64_array()
}

#[cfg(windows)]
fn system_random(bytes: &mut [u8]) -> bool {
    use std::ffi::c_void;

    #[link(name = "bcrypt")]
    unsafe extern "system" {
        fn BCryptGenRandom(algorithm: *mut c_void, buffer: *mut u8, length: u32, flags: u32)
        -> i32;
    }
    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 2;
    unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            bytes.as_mut_ptr(),
            bytes.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        ) == 0
    }
}

pub(crate) fn fill_random(scope: &mut v8::PinScope<'_, '_>, bytes: &mut [u8]) -> bool {
    crate::determinism::fill_random(scope, bytes) || system_random(bytes)
}

#[cfg(not(windows))]
fn system_random(bytes: &mut [u8]) -> bool {
    use std::io::Read;
    std::fs::File::open("/dev/urandom")
        .and_then(|mut source| source.read_exact(bytes))
        .is_ok()
}

fn get_random_values(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !scope.get_slot::<CryptoStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let value = arguments.get(0);
    if !integer_array(value) {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The provided ArrayBufferView is not an integer array type".to_owned(),
            "TypeMismatchError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let Ok(view) = v8::Local::<v8::ArrayBufferView>::try_from(value) else {
        crate::webidl::throw_type_error(scope, "getRandomValues requires an integer typed array");
        return;
    };
    if view.byte_length() > 65_536 {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "The requested length exceeds 65,536 bytes".to_owned(),
            "QuotaExceededError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let bytes =
        unsafe { std::slice::from_raw_parts_mut(view.data().cast::<u8>(), view.byte_length()) };
    if !fill_random(scope, bytes) {
        crate::webidl::throw_type_error(scope, "The system random generator failed");
        return;
    }
    result.set(value);
}

fn get_subtle(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = scope.get_slot::<CryptoStore>().and_then(|store| {
        store
            .records
            .get(&arguments.this().get_identity_hash().get())
    }) {
        result.set(v8::Local::new(scope, &record.subtle).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn random_uuid(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !scope.get_slot::<CryptoStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&arguments.this().get_identity_hash().get())
    }) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let mut bytes = [0_u8; 16];
    if !fill_random(scope, &mut bytes) {
        crate::webidl::throw_type_error(scope, "The system random generator failed");
        return;
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let value = format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    );
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<CryptoStore>() {
        store.constructors.remove(&realm_id);
    }
}
