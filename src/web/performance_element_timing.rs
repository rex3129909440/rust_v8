use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PerformanceElementTimingStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ElementTimingRecord>,
}

#[derive(Clone)]
struct ElementTimingRecord {
    render_time: f64,
    load_time: f64,
    intersection_rect: v8::Global<v8::Object>,
    identifier: String,
    natural_width: i32,
    natural_height: i32,
    id: String,
    element: Option<v8::Global<v8::Value>>,
    url: String,
    paint_time: f64,
    presentation_time: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PerformanceElementTimingStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PerformanceElementTiming", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PerformanceElementTimingStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PerformanceElementTiming",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "renderTime", get_render_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "loadTime", get_load_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "intersectionRect",
        get_intersection_rect,
    )?;
    crate::webidl::define_readonly_accessor(scope, prototype, "identifier", get_identifier)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "naturalWidth", get_natural_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "naturalHeight", get_natural_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "element", get_element)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "paintTime", get_paint_time)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "presentationTime",
        get_presentation_time,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::performance_entry::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PerformanceElementTimingStore>()
        .ok_or_else(|| "PerformanceElementTiming state was not prepared".to_owned())?
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
        "Failed to construct 'PerformanceElementTiming': Illegal constructor",
    );
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    name: String,
    start_time: f64,
    element: Option<v8::Local<'_, v8::Value>>,
    url: String,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let timing = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, timing, prototype.into()) != Some(true) {
        return Err("cannot create PerformanceElementTiming".to_owned());
    }
    super::performance_entry::attach(
        scope,
        timing,
        name.clone(),
        "element".to_owned(),
        start_time,
        0.0,
    );
    let rect = v8::Object::new(scope);
    define_number(scope, rect, "x", 0.0);
    define_number(scope, rect, "y", 0.0);
    define_number(scope, rect, "width", 0.0);
    define_number(scope, rect, "height", 0.0);
    let element = element.map(|value| v8::Global::new(scope, value));
    let intersection_rect = v8::Global::new(scope, rect);
    scope
        .get_slot_mut::<PerformanceElementTimingStore>()
        .ok_or_else(|| "PerformanceElementTiming state was not prepared".to_owned())?
        .records
        .insert(
            timing.get_identity_hash().get(),
            ElementTimingRecord {
                render_time: start_time,
                load_time: start_time,
                intersection_rect,
                identifier: name.clone(),
                natural_width: 0,
                natural_height: 0,
                id: name,
                element,
                url,
                paint_time: 0.0,
                presentation_time: 0.0,
            },
        );
    Ok(timing)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ElementTimingRecord> {
    scope
        .get_slot::<PerformanceElementTimingStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ElementTimingRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&ElementTimingRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_render_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.render_time);
}
fn get_load_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.load_time);
}
fn get_paint_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.paint_time);
}
fn get_presentation_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.presentation_time);
}
fn get_identifier(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.identifier);
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.id);
}
fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |v| &v.url);
}
fn get_natural_width(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.natural_width).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_natural_height(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.natural_height).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_intersection_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.intersection_rect).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
fn get_element(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    if let Some(element) = record.element {
        result.set(v8::Local::new(scope, &element));
    } else {
        result.set(v8::null(scope).into());
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
fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    define_value(scope, object, name, v8::Number::new(scope, value).into());
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
fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let Some(base) = super::performance_entry::record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = super::performance_entry::to_object(scope, &base);
    define_number(scope, output, "renderTime", record.render_time);
    define_number(scope, output, "loadTime", record.load_time);
    define_value(
        scope,
        output,
        "intersectionRect",
        v8::Local::new(scope, &record.intersection_rect).into(),
    );
    define_string(scope, output, "identifier", &record.identifier);
    define_value(
        scope,
        output,
        "naturalWidth",
        v8::Integer::new(scope, record.natural_width).into(),
    );
    define_value(
        scope,
        output,
        "naturalHeight",
        v8::Integer::new(scope, record.natural_height).into(),
    );
    define_string(scope, output, "id", &record.id);
    if let Some(element) = record.element {
        define_value(scope, output, "element", v8::Local::new(scope, &element));
    } else {
        define_value(scope, output, "element", v8::null(scope).into());
    }
    define_string(scope, output, "url", &record.url);
    define_number(scope, output, "paintTime", record.paint_time);
    define_number(scope, output, "presentationTime", record.presentation_time);
    result.set(output.into());
}
