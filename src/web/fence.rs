use std::collections::{HashMap, HashSet};

#[derive(Default)]
struct FenceRecord {
    nested_configs: Vec<v8::Global<v8::Object>>,
    reports: Vec<HashMap<String, String>>,
    automatic_beacon: HashMap<String, String>,
}

#[derive(Default)]
pub(crate) struct FenceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, FenceRecord>,
    native_objects: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FenceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Fence", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FenceStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Fence",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getNestedConfigs", 0, get_nested_configs)?;
    crate::webidl::define_method(scope, prototype, "reportEvent", 1, report_event)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setReportEventDataForAutomaticBeacons",
        1,
        set_automatic_beacon,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FenceStore>()
        .ok_or_else(|| "Fence state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    nested_configs: Vec<v8::Local<'_, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let fence = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, fence, prototype.into()) != Some(true) {
        return Err("cannot create Fence".to_owned());
    }
    let record = FenceRecord {
        nested_configs: nested_configs
            .into_iter()
            .map(|config| v8::Global::new(scope, config))
            .collect(),
        ..FenceRecord::default()
    };
    let identity = fence.get_identity_hash().get();
    let store = scope
        .get_slot_mut::<FenceStore>()
        .ok_or_else(|| "Fence state was not prepared".to_owned())?;
    store.native_objects.insert(identity);
    store.records.insert(identity, record);
    Ok(fence)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Fence': Illegal constructor");
}

fn get_nested_configs(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let configs = scope
        .get_slot::<FenceStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .map(|record| record.nested_configs.clone());
    let Some(configs) = configs else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let values = v8::Array::new(scope, configs.len() as i32);
    for (index, config) in configs.iter().enumerate() {
        let config = v8::Local::new(scope, config);
        let _ = values.set_index(scope, index as u32, config.into());
    }
    result.set(values.into());
}

fn report_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_dictionary(scope, arguments, false);
}

fn set_automatic_beacon(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    update_dictionary(scope, arguments, true);
}

fn update_dictionary(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    automatic: bool,
) {
    if scope
        .get_slot::<FenceStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .is_none()
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "Fence reporting requires one event data argument.");
        return;
    }
    let Ok(data) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "Fence event data must be an object");
        return;
    };
    let dictionary = dictionary(scope, data);
    let Some(record) = scope.get_slot_mut::<FenceStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if automatic {
        record.automatic_beacon = dictionary;
    } else {
        record.reports.push(dictionary);
    }
}

fn dictionary(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> HashMap<String, String> {
    let mut output = HashMap::new();
    if let Some(names) = object.get_own_property_names(scope, Default::default()) {
        for index in 0..names.length() {
            let Some(key) = names.get_index(scope, index) else {
                continue;
            };
            let name = crate::webidl::value_to_string(scope, key);
            if let Some(value) = object.get(scope, key) {
                output.insert(name, crate::webidl::value_to_string(scope, value));
            }
        }
    }
    output
}
