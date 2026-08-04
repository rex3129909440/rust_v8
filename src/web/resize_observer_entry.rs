use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ResizeObserverEntryStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, EntryRecord>,
}

#[derive(Clone)]
struct EntryRecord {
    target: v8::Global<v8::Value>,
    content_rect: v8::Global<v8::Value>,
    content_box_size: v8::Global<v8::Value>,
    border_box_size: v8::Global<v8::Value>,
    device_pixel_content_box_size: v8::Global<v8::Value>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ResizeObserverEntryStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "ResizeObserverEntry", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<ResizeObserverEntryStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "ResizeObserverEntry",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "target", get_target)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "contentRect", get_content_rect)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "contentBoxSize",
        get_content_box_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "borderBoxSize",
        get_border_box_size,
    )?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "devicePixelContentBoxSize",
        get_device_pixel_content_box_size,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<ResizeObserverEntryStore>()
        .ok_or_else(|| "ResizeObserverEntry state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    target: v8::Local<'s, v8::Object>,
    content_width: f64,
    content_height: f64,
    border_width: f64,
    border_height: f64,
    device_width: f64,
    device_height: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create ResizeObserverEntry".to_owned());
    }
    let content_rect = v8::Object::new(scope);
    define_number(scope, content_rect, "x", 0.0)?;
    define_number(scope, content_rect, "y", 0.0)?;
    define_number(scope, content_rect, "width", content_width)?;
    define_number(scope, content_rect, "height", content_height)?;
    define_number(scope, content_rect, "top", 0.0)?;
    define_number(scope, content_rect, "right", content_width)?;
    define_number(scope, content_rect, "bottom", content_height)?;
    define_number(scope, content_rect, "left", 0.0)?;
    let content_size = super::resize_observer_size::create(scope, content_width, content_height)?;
    let border_size = super::resize_observer_size::create(scope, border_width, border_height)?;
    let device_size = super::resize_observer_size::create(scope, device_width, device_height)?;
    let content_box_size = v8::Array::new(scope, 1);
    let border_box_size = v8::Array::new(scope, 1);
    let device_pixel_content_box_size = v8::Array::new(scope, 1);
    let _ = content_box_size.set_index(scope, 0, content_size.into());
    let _ = border_box_size.set_index(scope, 0, border_size.into());
    let _ = device_pixel_content_box_size.set_index(scope, 0, device_size.into());
    let target_value: v8::Local<v8::Value> = target.into();
    let content_rect_value: v8::Local<v8::Value> = content_rect.into();
    let content_box_size_value: v8::Local<v8::Value> = content_box_size.into();
    let border_box_size_value: v8::Local<v8::Value> = border_box_size.into();
    let device_pixel_content_box_size_value: v8::Local<v8::Value> =
        device_pixel_content_box_size.into();
    let record = EntryRecord {
        target: v8::Global::new(scope, target_value),
        content_rect: v8::Global::new(scope, content_rect_value),
        content_box_size: v8::Global::new(scope, content_box_size_value),
        border_box_size: v8::Global::new(scope, border_box_size_value),
        device_pixel_content_box_size: v8::Global::new(scope, device_pixel_content_box_size_value),
    };
    scope
        .get_slot_mut::<ResizeObserverEntryStore>()
        .ok_or_else(|| "ResizeObserverEntry state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) -> Result<(), String> {
    let key = v8::String::new(scope, name).ok_or_else(|| "cannot allocate property".to_owned())?;
    let value = v8::Number::new(scope, value);
    if object.define_own_property(
        scope,
        key.into(),
        value.into(),
        v8::PropertyAttribute::READ_ONLY,
    ) != Some(true)
    {
        return Err(format!("cannot define {name}"));
    }
    Ok(())
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'ResizeObserverEntry': Illegal constructor",
    );
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<EntryRecord> {
    scope
        .get_slot::<ResizeObserverEntryStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_object(
    scope: &v8::PinScope<'_, '_>,
    object: &v8::Global<v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    result.set(v8::Local::new(scope, object).into());
}

fn get_target(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.target, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_content_rect(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.content_rect, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_content_box_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.content_box_size, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_border_box_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.border_box_size, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_device_pixel_content_box_size(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    if let Some(v) = record(s, a.this()) {
        return_object(s, &v.device_pixel_content_box_size, r)
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
