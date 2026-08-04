use std::collections::HashMap;

#[derive(Clone)]
struct NotRestoredReasonsRecord {
    src: String,
    id: String,
    name: String,
    url: String,
    reasons: v8::Global<v8::Value>,
    children: v8::Global<v8::Value>,
}

#[derive(Default)]
pub(crate) struct NotRestoredReasonsStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, NotRestoredReasonsRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NotRestoredReasonsStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "NotRestoredReasons", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<NotRestoredReasonsStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "NotRestoredReasons",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "src", get_src)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "reasons", get_reasons)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "children", get_children)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<NotRestoredReasonsStore>()
        .ok_or_else(|| "NotRestoredReasons state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    src: String,
    id: String,
    name: String,
    url: String,
    reasons: v8::Local<'_, v8::Value>,
    children: v8::Local<'_, v8::Value>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let value = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, value, prototype.into()) != Some(true) {
        return Err("cannot create NotRestoredReasons".to_owned());
    }
    let record = NotRestoredReasonsRecord {
        src,
        id,
        name,
        url,
        reasons: v8::Global::new(scope, reasons),
        children: v8::Global::new(scope, children),
    };
    scope
        .get_slot_mut::<NotRestoredReasonsStore>()
        .ok_or_else(|| "NotRestoredReasons state was not prepared".to_owned())?
        .records
        .insert(value.get_identity_hash().get(), record);
    Ok(value)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'NotRestoredReasons': Illegal constructor",
    );
}

fn record_for(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<NotRestoredReasonsRecord> {
    scope
        .get_slot::<NotRestoredReasonsStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_text(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&NotRestoredReasonsRecord) -> &str,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(value) = v8::String::new(scope, select(&record)) {
        result.set(value.into());
    }
}

fn get_src(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_text(scope, arguments, result, |record| &record.src);
}

fn get_id(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_text(scope, arguments, result, |record| &record.id);
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_text(scope, arguments, result, |record| &record.name);
}

fn get_url(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    return_text(scope, arguments, result, |record| &record.url);
}

fn get_reasons(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.reasons));
}

fn get_children(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    result.set(v8::Local::new(scope, &record.children));
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record_for(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let value = v8::Object::new(scope);
    if define_text(scope, value, "src", &record.src).is_err()
        || define_text(scope, value, "id", &record.id).is_err()
        || define_text(scope, value, "name", &record.name).is_err()
        || define_text(scope, value, "url", &record.url).is_err()
    {
        return;
    }
    let reasons = v8::Local::new(scope, &record.reasons);
    if define_value(scope, value, "reasons", reasons).is_err() {
        return;
    }
    let children = v8::Local::new(scope, &record.children);
    if define_value(scope, value, "children", children).is_ok() {
        result.set(value.into());
    }
}

fn define_text(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    text: &str,
) -> Result<(), String> {
    let value = crate::webidl::string(scope, text)?;
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
        Err(format!("cannot define NotRestoredReasons.{name}"))
    }
}
