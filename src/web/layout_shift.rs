use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LayoutShiftStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, LayoutShiftRecord>,
}

#[derive(Clone)]
struct LayoutShiftRecord {
    value: f64,
    had_recent_input: bool,
    last_input_time: f64,
    sources: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LayoutShiftStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LayoutShift", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<LayoutShiftStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "LayoutShift",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "value", get_value)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "hadRecentInput",
        get_had_recent_input,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "lastInputTime",
        get_last_input_time,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "sources", get_sources)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<LayoutShiftStore>()
        .ok_or_else(|| "LayoutShift state was not prepared".to_owned())?
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
        "Failed to construct 'LayoutShift': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    start_time: f64,
    value: f64,
    had_recent_input: bool,
    last_input_time: f64,
    sources: Vec<v8::Local<'s, v8::Object>>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create LayoutShift".to_owned());
    }
    super::performance_entry::attach(
        scope,
        object,
        String::new(),
        "layout-shift".to_owned(),
        start_time,
        0.0,
    );
    let sources = sources
        .into_iter()
        .map(|source| v8::Global::new(scope, source))
        .collect();
    scope
        .get_slot_mut::<LayoutShiftStore>()
        .ok_or_else(|| "LayoutShift state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            LayoutShiftRecord {
                value,
                had_recent_input,
                last_input_time,
                sources,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LayoutShiftRecord> {
    scope
        .get_slot::<LayoutShiftStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&LayoutShiftRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_value(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.value)
}
fn get_last_input_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.last_input_time)
}
fn get_had_recent_input(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.had_recent_input).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_sources(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let array = v8::Array::new(scope, record.sources.len() as i32);
    for (index, source) in record.sources.iter().enumerate() {
        let source = v8::Local::new(scope, source);
        let _ = array.set_index(scope, index as u32, source.into());
    }
    result.set(array.into())
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
    let base = super::performance_entry::record(scope, arguments.this())
        .map(|value| super::performance_entry::to_object(scope, &value))
        .unwrap_or_else(|| v8::Object::new(scope));
    define(
        scope,
        base,
        "value",
        v8::Number::new(scope, record.value).into(),
    );
    define(
        scope,
        base,
        "hadRecentInput",
        v8::Boolean::new(scope, record.had_recent_input).into(),
    );
    define(
        scope,
        base,
        "lastInputTime",
        v8::Number::new(scope, record.last_input_time).into(),
    );
    let sources = v8::Array::new(scope, record.sources.len() as i32);
    for (index, source) in record.sources.iter().enumerate() {
        let value = v8::Local::new(scope, source);
        let _ = sources.set_index(scope, index as u32, value.into());
    }
    define(scope, base, "sources", sources.into());
    result.set(base.into())
}
fn define(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: v8::Local<'_, v8::Value>,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), value);
    }
}
