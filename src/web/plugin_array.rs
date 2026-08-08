use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PluginArrayStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PluginArrayStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PluginArray", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PluginArrayStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PluginArray",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)?;
    crate::webidl::define_method(scope, prototype, "refresh", 0, refresh)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PluginArrayStore>()
        .ok_or_else(|| "PluginArray state was not prepared".to_owned())?
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
        "Failed to construct 'PluginArray': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let array = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, array, prototype.into()) != Some(true) {
        return Err("cannot create PluginArray".to_owned());
    }
    if !crate::fingerprint::navigator(scope).pdf_viewer_enabled {
        scope
            .get_slot_mut::<PluginArrayStore>()
            .ok_or_else(|| "PluginArray state was not prepared".to_owned())?
            .records
            .insert(array.get_identity_hash().get(), Vec::new());
        return Ok(array);
    }
    let configured = crate::fingerprint::edge(scope).plugins.plugins.clone();
    let mut values = Vec::with_capacity(configured.len());
    for (index, profile) in configured.iter().enumerate() {
        let plugin = super::plugin::create(scope, profile)?;
        define_value(scope, array, &index.to_string(), plugin.into());
        define_value(scope, array, &profile.name, plugin.into());
        values.push(v8::Global::new(scope, plugin));
    }
    scope
        .get_slot_mut::<PluginArrayStore>()
        .ok_or_else(|| "PluginArray state was not prepared".to_owned())?
        .records
        .insert(array.get_identity_hash().get(), values);
    Ok(array)
}

fn define_value(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.create_data_property(scope, key.into(), value);
    }
}

pub(crate) fn items(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Global<v8::Object>>> {
    scope
        .get_slot::<PluginArrayStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = items(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.len() as i32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(plugin) = record.get(index) {
        result.set(v8::Local::new(scope, plugin).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = items(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let found = record.iter().find(|plugin| {
        let plugin = v8::Local::new(scope, *plugin);
        super::plugin::name(scope, plugin).is_some_and(|value| value == name)
    });
    if let Some(plugin) = found {
        result.set(v8::Local::new(scope, plugin).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn refresh(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if items(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
