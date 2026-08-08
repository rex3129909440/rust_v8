use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LargestContentfulPaintStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, PaintRecord>,
}

#[derive(Clone)]
struct PaintRecord {
    render_time: f64,
    load_time: f64,
    size: u64,
    id: String,
    url: String,
    element: Option<v8::Global<v8::Object>>,
    paint_time: f64,
    presentation_time: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LargestContentfulPaintStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LargestContentfulPaint", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<LargestContentfulPaintStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "LargestContentfulPaint",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "renderTime", get_render_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "loadTime", get_load_time)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "size", get_size)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "url", get_url)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "element", get_element)?;
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
        .get_slot_mut::<LargestContentfulPaintStore>()
        .ok_or_else(|| "LargestContentfulPaint state was not prepared".to_owned())?
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
        "Failed to construct 'LargestContentfulPaint': Illegal constructor",
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    start_time: f64,
    render_time: f64,
    load_time: f64,
    size: u64,
    id: String,
    url: String,
    element: Option<v8::Local<'s, v8::Object>>,
    paint_time: f64,
    presentation_time: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create LargestContentfulPaint".to_owned());
    }
    super::performance_entry::attach(
        scope,
        object,
        String::new(),
        "largest-contentful-paint".to_owned(),
        start_time,
        0.0,
    );
    let element = element.map(|value| v8::Global::new(scope, value));
    scope
        .get_slot_mut::<LargestContentfulPaintStore>()
        .ok_or_else(|| "LargestContentfulPaint state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            PaintRecord {
                render_time,
                load_time,
                size,
                id,
                url,
                element,
                paint_time,
                presentation_time,
            },
        );
    Ok(object)
}
fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<PaintRecord> {
    scope
        .get_slot::<LargestContentfulPaintStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PaintRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_render_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.render_time)
}
fn get_load_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.load_time)
}
fn get_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.size as f64)
}
fn get_paint_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.paint_time)
}
fn get_presentation_time(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |x| x.presentation_time)
}
fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PaintRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into())
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}
fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.id)
}
fn get_url(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |x| &x.url)
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
    if let Some(value) = record.element {
        result.set(v8::Local::new(scope, &value).into())
    } else {
        result.set(v8::null(scope).into())
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
    let base = super::performance_entry::record(scope, arguments.this())
        .map(|value| super::performance_entry::to_object(scope, &value))
        .unwrap_or_else(|| v8::Object::new(scope));
    define(
        scope,
        base,
        "renderTime",
        v8::Number::new(scope, record.render_time).into(),
    );
    define(
        scope,
        base,
        "loadTime",
        v8::Number::new(scope, record.load_time).into(),
    );
    define(
        scope,
        base,
        "size",
        v8::Number::new(scope, record.size as f64).into(),
    );
    if let Some(value) = v8::String::new(scope, &record.id) {
        define(scope, base, "id", value.into())
    }
    if let Some(value) = v8::String::new(scope, &record.url) {
        define(scope, base, "url", value.into())
    }
    if let Some(value) = record.element {
        define(scope, base, "element", v8::Local::new(scope, &value).into())
    } else {
        define(scope, base, "element", v8::null(scope).into())
    }
    define(
        scope,
        base,
        "paintTime",
        v8::Number::new(scope, record.paint_time).into(),
    );
    define(
        scope,
        base,
        "presentationTime",
        v8::Number::new(scope, record.presentation_time).into(),
    );
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
