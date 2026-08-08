use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NetworkInformationStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NetworkInformationRecord>,
}

#[derive(Clone)]
pub(crate) struct NetworkInformationRecord {
    pub(crate) onchange: Option<v8::Global<v8::Value>>,
    pub(crate) effective_type: String,
    pub(crate) rtt: u32,
    pub(crate) downlink: f64,
    pub(crate) save_data: bool,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NetworkInformationStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NetworkInformation", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<NetworkInformationStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NetworkInformation",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::network_information_onchange_property::define(scope, prototype)?;
    super::network_information_effective_type_property::define(scope, prototype)?;
    super::network_information_rtt_property::define(scope, prototype)?;
    super::network_information_downlink_property::define(scope, prototype)?;
    super::network_information_save_data_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NetworkInformationStore>()
        .ok_or_else(|| "NetworkInformation state was not prepared".to_owned())?
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
        return Err("cannot create NetworkInformation".to_owned());
    }
    super::event_target::attach(scope, object);
    let network = crate::fingerprint::navigator(scope).network.clone();
    scope
        .get_slot_mut::<NetworkInformationStore>()
        .ok_or_else(|| "NetworkInformation state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            NetworkInformationRecord {
                onchange: None,
                effective_type: network.effective_type,
                rtt: network.rtt,
                downlink: network.downlink,
                save_data: network.save_data,
            },
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
        "Failed to construct 'NetworkInformation': Illegal constructor",
    )
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NetworkInformationRecord> {
    scope
        .get_slot::<NetworkInformationStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn set_onchange(
    scope: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = if a.get(0).is_null_or_undefined() {
        None
    } else {
        Some(v8::Global::new(scope, a.get(0)))
    };
    if let Some(v) = scope
        .get_slot_mut::<NetworkInformationStore>()
        .and_then(|store| store.records.get_mut(&a.this().get_identity_hash().get()))
    {
        v.onchange = value
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<NetworkInformationStore>() {
        store.constructor.remove(realm_id);
    }
}
