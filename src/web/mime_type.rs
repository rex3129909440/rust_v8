use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MimeTypeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, MimeTypeRecord>,
}

#[derive(Clone)]
struct MimeTypeRecord {
    mime_type: String,
    suffixes: String,
    description: String,
    enabled_plugin: v8::Global<v8::Object>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(MimeTypeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "MimeType", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<MimeTypeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "MimeType",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "suffixes", get_suffixes)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "description", get_description)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "enabledPlugin", get_enabled_plugin)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<MimeTypeStore>()
        .ok_or_else(|| "MimeType state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    mime_type: &str,
    suffixes: &str,
    description: &str,
    enabled_plugin: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create MimeType".to_owned());
    }
    let enabled_plugin = v8::Global::new(scope, enabled_plugin);
    scope
        .get_slot_mut::<MimeTypeStore>()
        .ok_or_else(|| "MimeType state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            MimeTypeRecord {
                mime_type: mime_type.to_owned(),
                suffixes: suffixes.to_owned(),
                description: description.to_owned(),
                enabled_plugin,
            },
        );
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Failed to construct 'MimeType': Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<MimeTypeRecord> {
    scope
        .get_slot::<MimeTypeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn mime_type(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<String> {
    record(scope, object).map(|record| record.mime_type)
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&MimeTypeRecord) -> &str,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.mime_type);
}

fn get_suffixes(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.suffixes);
}

fn get_description(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_string(scope, arguments, result, |record| &record.description);
}

fn get_enabled_plugin(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.enabled_plugin).into());
}
