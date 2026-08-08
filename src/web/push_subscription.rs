use std::collections::HashMap;
#[derive(Clone)]
struct PushSubscriptionRecord {
    endpoint: String,
    expiration: Option<f64>,
    options: v8::Global<v8::Object>,
    p256dh: Vec<u8>,
    auth: Vec<u8>,
    active: bool,
}
#[derive(Default)]
pub(crate) struct PushSubscriptionStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PushSubscriptionRecord>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(PushSubscriptionStore::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let c = ensure(s)?;
    crate::webidl::define_global(s, "PushSubscription", c.into())
}
fn ensure<'s>(s: &mut v8::PinScope<'s, '_>) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(s);
    if let Some(c) = s
        .get_slot::<PushSubscriptionStore>()
        .and_then(|x| x.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(s, &c));
    }
    let c = crate::webidl::create_function(
        s,
        "PushSubscription",
        0,
        v8::ConstructorBehavior::Allow,
        illegal,
    )?;
    let p = crate::webidl::prototype(s, c)?;
    crate::webidl::reset_constructor_order(s, p)?;
    crate::webidl::define_readonly_accessor(s, p, "endpoint", get_endpoint)?;
    crate::webidl::define_readonly_accessor(s, p, "expirationTime", get_expiration)?;
    crate::webidl::define_readonly_accessor(s, p, "options", get_options)?;
    crate::webidl::define_method(s, p, "getKey", 1, get_key)?;
    crate::webidl::define_method(s, p, "toJSON", 0, to_json)?;
    crate::webidl::define_method(s, p, "unsubscribe", 0, unsubscribe)?;
    crate::webidl::finish_constructor(s, p, c)?;
    let realm_constructor = v8::Global::new(s, c);
    s.get_slot_mut::<PushSubscriptionStore>()
        .ok_or_else(|| "PushSubscription state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(c)
}
pub(crate) fn create<'s>(
    s: &mut v8::PinScope<'s, '_>,
    endpoint: String,
    options: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let c = ensure(s)?;
    let p = crate::webidl::prototype(s, c)?;
    let o = v8::Object::new(s);
    if crate::webidl::set_platform_prototype(s, o, p.into()) != Some(true) {
        return Err("cannot create PushSubscription".to_owned());
    }
    let record = PushSubscriptionRecord {
        endpoint,
        expiration: None,
        options: v8::Global::new(s, options),
        p256dh: (1..=65).collect(),
        auth: (1..=16).collect(),
        active: true,
    };
    s.get_slot_mut::<PushSubscriptionStore>()
        .ok_or_else(|| "state missing".to_owned())?
        .records
        .insert(o.get_identity_hash().get(), record);
    Ok(o)
}
fn record(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
) -> Option<PushSubscriptionRecord> {
    s.get_slot::<PushSubscriptionStore>()?
        .records
        .get(&o.get_identity_hash().get())
        .cloned()
}
fn illegal(
    s: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        s,
        "Failed to construct 'PushSubscription': Illegal constructor",
    )
}
fn get_endpoint(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    if let Some(v) = v8::String::new(s, &x.endpoint) {
        r.set(v.into())
    }
}
fn get_expiration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    match x.expiration {
        Some(v) => r.set(v8::Number::new(s, v).into()),
        None => r.set(v8::null(s).into()),
    }
}
fn get_options(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Local::new(s, &x.options).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_key(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(s, a.get(0));
    let bytes = match name.as_str() {
        "p256dh" => Some(x.p256dh),
        "auth" => Some(x.auth),
        _ => None,
    };
    match bytes {
        Some(v) => {
            let backing = v8::ArrayBuffer::new_backing_store_from_vec(v).make_shared();
            r.set(v8::ArrayBuffer::with_backing_store(s, &backing).into())
        }
        None => r.set(v8::null(s).into()),
    }
}
fn unsubscribe(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let active = record(s, a.this()).is_some_and(|x| x.active);
    if let Some(x) = s
        .get_slot_mut::<PushSubscriptionStore>()
        .and_then(|x| x.records.get_mut(&a.this().get_identity_hash().get()))
    {
        x.active = false
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    }
    let value = v8::Boolean::new(s, active);
    if let Ok(p) = super::writable_stream::resolved_promise(s, value.into()) {
        r.set(p.into())
    }
}
fn to_json(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    let Some(x) = record(s, a.this()) else {
        crate::webidl::throw_type_error(s, "Illegal invocation");
        return;
    };
    let o = v8::Object::new(s);
    define_text(s, o, "endpoint", &x.endpoint);
    let expiration = v8::null(s);
    define_value(s, o, "expirationTime", expiration.into());
    let keys = v8::Object::new(s);
    define_text(s, keys, "p256dh", &base64_url(&x.p256dh));
    define_text(s, keys, "auth", &base64_url(&x.auth));
    define_value(s, o, "keys", keys.into());
    r.set(o.into())
}
fn base64_url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((bytes.len() * 4 + 2) / 3);
    let mut index = 0;
    while index + 3 <= bytes.len() {
        let value = ((bytes[index] as u32) << 16)
            | ((bytes[index + 1] as u32) << 8)
            | bytes[index + 2] as u32;
        output.push(ALPHABET[((value >> 18) & 63) as usize] as char);
        output.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        output.push(ALPHABET[((value >> 6) & 63) as usize] as char);
        output.push(ALPHABET[(value & 63) as usize] as char);
        index += 3;
    }
    match bytes.len() - index {
        1 => {
            let value = (bytes[index] as u32) << 16;
            output.push(ALPHABET[((value >> 18) & 63) as usize] as char);
            output.push(ALPHABET[((value >> 12) & 63) as usize] as char);
        }
        2 => {
            let value = ((bytes[index] as u32) << 16) | ((bytes[index + 1] as u32) << 8);
            output.push(ALPHABET[((value >> 18) & 63) as usize] as char);
            output.push(ALPHABET[((value >> 12) & 63) as usize] as char);
            output.push(ALPHABET[((value >> 6) & 63) as usize] as char);
        }
        _ => {}
    }
    output
}
fn define_text(s: &v8::PinScope<'_, '_>, o: v8::Local<'_, v8::Object>, n: &str, v: &str) {
    if let Some(v) = v8::String::new(s, v) {
        define_value(s, o, n, v.into())
    }
}
fn define_value(
    s: &v8::PinScope<'_, '_>,
    o: v8::Local<'_, v8::Object>,
    n: &str,
    v: v8::Local<'_, v8::Value>,
) {
    if let Some(k) = v8::String::new(s, n) {
        let _ = o.define_own_property(s, k.into(), v, v8::PropertyAttribute::NONE);
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PushSubscriptionStore>() {
        store.constructor.remove(realm_id);
    }
}
