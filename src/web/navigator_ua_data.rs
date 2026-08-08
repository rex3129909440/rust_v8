use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigatorUaDataStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NavigatorUaDataRecord>,
}

#[derive(Clone)]
pub(crate) struct NavigatorUaDataRecord {
    pub(crate) profile: crate::fingerprint::UserAgentDataFingerprint,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigatorUaDataStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NavigatorUAData", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<NavigatorUaDataStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NavigatorUAData",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::navigator_ua_data_brands_property::define(scope, prototype)?;
    super::navigator_ua_data_mobile_property::define(scope, prototype)?;
    super::navigator_ua_data_platform_property::define(scope, prototype)?;
    super::navigator_ua_data_get_high_entropy_values::define(scope, prototype)?;
    super::navigator_ua_data_to_json::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NavigatorUaDataStore>()
        .ok_or_else(|| "NavigatorUAData state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create NavigatorUAData".to_owned());
    }
    let profile = crate::fingerprint::navigator(scope).user_agent_data.clone();
    scope
        .get_slot_mut::<NavigatorUaDataStore>()
        .ok_or_else(|| "NavigatorUAData state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NavigatorUaDataRecord { profile },
        );
    Ok(object)
}
fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NavigatorUAData': Illegal constructor",
    )
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NavigatorUaDataRecord> {
    scope
        .get_slot::<NavigatorUaDataStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn brands_array<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &NavigatorUaDataRecord,
    full: bool,
) -> v8::Local<'s, v8::Array> {
    let array = v8::Array::new(scope, record.profile.brands.len() as i32);
    for (index, brand) in record.profile.brands.iter().enumerate() {
        let item = v8::Object::new(scope);
        define_string(scope, item, "brand", &brand.brand);
        let version = if full {
            &brand.full_version
        } else {
            &brand.version
        };
        define_string(scope, item, "version", version);
        let _ = array.set_index(scope, index as u32, item.into());
    }
    array
}
pub(crate) fn to_object<'s>(
    scope: &v8::PinScope<'s, '_>,
    record: &NavigatorUaDataRecord,
) -> v8::Local<'s, v8::Object> {
    let object = v8::Object::new(scope);
    define(
        scope,
        object,
        "brands",
        brands_array(scope, record, false).into(),
    );
    define(
        scope,
        object,
        "mobile",
        v8::Boolean::new(scope, record.profile.mobile).into(),
    );
    define_string(scope, object, "platform", &record.profile.platform);
    object
}
pub(crate) fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}
pub(crate) fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define(scope, object, name, value.into())
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<NavigatorUaDataStore>() {
        store.constructor.remove(realm_id);
    }
}
