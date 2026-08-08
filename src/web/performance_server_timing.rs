use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceServerTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ServerTimingRecord>,
}

#[derive(Clone)]
struct ServerTimingRecord {
    name: String,
    duration: f64,
    description: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceServerTimingStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceServerTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    let existing = scope
        .get_slot::<PerformanceServerTimingStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceServerTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "name", get_name)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "duration", get_duration)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "description", get_description)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceServerTimingStore>()
        .ok_or_else(|| "PerformanceServerTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceServerTiming': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    duration: f64,
    description: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceServerTiming".to_owned());
    }
    scope
        .get_slot_mut::<PerformanceServerTimingStore>()
        .ok_or_else(|| "PerformanceServerTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            ServerTimingRecord {
                name,
                duration,
                description,
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ServerTimingRecord> {
    scope
        .get_slot::<PerformanceServerTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ServerTimingRecord) -> &str,
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
    return_string(s, a, r, |v| &v.name);
}
fn get_description(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.description);
}
fn get_duration(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, record.duration).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
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

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    if let Some(value) = v8::String::new(scope, &record.name) {
        define_value(scope, output, "name", value.into());
    }
    define_value(
        scope,
        output,
        "duration",
        v8::Number::new(scope, record.duration).into(),
    );
    if let Some(value) = v8::String::new(scope, &record.description) {
        define_value(scope, output, "description", value.into());
    }
    result.set(output.into());
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<PerformanceServerTimingStore>() {
        store.constructor.remove(realm_id);
    }
}
