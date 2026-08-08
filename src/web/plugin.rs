use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PluginStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PluginRecord>,
}

#[derive(Clone)]
struct PluginRecord {
    name: String,
    filename: String,
    description: String,
    mime_types: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PluginStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Plugin", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PluginStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Plugin",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "filename", get_filename)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "description", get_description)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "length", get_length)?;
    crate::webidl::define_method(scope, prototype, "item", 1, item)?;
    crate::webidl::define_method(scope, prototype, "namedItem", 1, named_item)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_indexed_iterator(scope, prototype)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PluginStore>()
        .ok_or_else(|| "Plugin state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'Plugin': Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    profile: &crate::PluginFingerprint,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let plugin = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, plugin, prototype.into()) != Some(true) {
        return Err("cannot create Plugin".to_owned());
    }
    let record = PluginRecord {
        name: profile.name.clone(),
        filename: profile.filename.clone(),
        description: profile.description.clone(),
        mime_types: Vec::new(),
    };
    scope
        .get_slot_mut::<PluginStore>()
        .ok_or_else(|| "Plugin state was not prepared".to_owned())?
        .records
        .insert(plugin.get_identity_hash().get(), record);
    let mut mime_types = Vec::with_capacity(profile.mime_types.len());
    for (index, mime) in profile.mime_types.iter().enumerate() {
        let object = super::mime_type::create(
            scope,
            &mime.mime_type,
            &mime.suffixes,
            &mime.description,
            plugin,
        )?;
        define_value(scope, plugin, &index.to_string(), object.into());
        define_value(scope, plugin, &mime.mime_type, object.into());
        mime_types.push(v8::Global::new(scope, object));
    }
    scope
        .get_slot_mut::<PluginStore>()
        .ok_or_else(|| "Plugin state was not prepared".to_owned())?
        .records
        .get_mut(&plugin.get_identity_hash().get())
        .expect("Plugin record")
        .mime_types = mime_types;
    Ok(plugin)
}

fn define_string(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: &str,
) {
    if let Some(value) = v8::String::new(scope, value) {
        define_value(scope, object, name, value.into());
    }
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

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<PluginRecord> {
    scope
        .get_slot::<PluginStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn name(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.name)
}

pub(crate) fn mime_types<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<v8::Local<'s, v8::Object>>> {
    record(scope, object).map(|record| {
        record
            .mime_types
            .iter()
            .map(|mime| v8::Local::new(scope, mime))
            .collect()
    })
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PluginRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_name(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.name);
}

fn get_filename(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.filename);
}

fn get_description(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.description);
}

fn get_length(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.mime_types.len() as i32).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let index = arguments.get(0).uint32_value(scope).unwrap_or(u32::MAX) as usize;
    if let Some(value) = record.mime_types.get(index) {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}

fn named_item(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let name = crate::webidl::value_to_string(scope, arguments.get(0));
    let found = record.mime_types.iter().find(|mime| {
        let mime = v8::Local::new(scope, *mime);
        super::mime_type::mime_type(scope, mime).is_some_and(|value| value == name)
    });
    if let Some(value) = found {
        result.set(v8::Local::new(scope, value).into());
    } else {
        result.set(v8::null(scope).into());
    }
}
