use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) struct RectRecord {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Default)]
pub(crate) struct DomRectReadOnlyStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, RectRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DomRectReadOnlyStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DOMRectReadOnly", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<DomRectReadOnlyStore>()
        .and_then(|store| store.constructor.get(realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DOMRectReadOnly",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "x", get_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "y", get_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "width", get_width)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "height", get_height)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "top", get_top)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "right", get_right)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "bottom", get_bottom)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "left", get_left)?;
    crate::webidl::define_method(scope, prototype, "toJSON", 0, to_json)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    crate::webidl::define_method(scope, constructor.into(), "fromRect", 0, from_rect)?;
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DomRectReadOnlyStore>()
        .ok_or_else(|| "DOMRectReadOnly state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn from_value(
    scope: &v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
) -> RectRecord {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return RectRecord {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    };
    let property = |name: &str| {
        v8::String::new(scope, name)
            .and_then(|key| object.get(scope, key.into()))
            .filter(|value| !value.is_undefined())
            .and_then(|value| value.number_value(scope))
            .unwrap_or(0.0)
    };
    RectRecord {
        x: property("x"),
        y: property("y"),
        width: property("width"),
        height: property("height"),
    }
}

fn from_rect(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let rect = from_value(scope, arguments.get(0));
    match create(scope, rect) {
        Ok(rect) => result.set(rect.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "DOMRectReadOnly must be constructed with new");
        return;
    }
    let value = |index: i32| {
        if arguments.get(index).is_undefined() {
            0.0
        } else {
            arguments.get(index).number_value(scope).unwrap_or(f64::NAN)
        }
    };
    attach(
        scope,
        arguments.this(),
        RectRecord {
            x: value(0),
            y: value(1),
            width: value(2),
            height: value(3),
        },
    );
    result.set(arguments.this().into());
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    record: RectRecord,
) {
    if let Some(store) = scope.get_slot_mut::<DomRectReadOnlyStore>() {
        store
            .records
            .insert(object.get_identity_hash().get(), record);
    }
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    record: RectRecord,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let rect = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, rect, prototype.into()) != Some(true) {
        return Err("cannot create DOMRectReadOnly".to_owned());
    }
    attach(scope, rect, record);
    Ok(rect)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<RectRecord> {
    scope
        .get_slot::<DomRectReadOnlyStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .copied()
}

pub(crate) fn update(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    change: impl FnOnce(&mut RectRecord),
) -> bool {
    let Some(record) = scope
        .get_slot_mut::<DomRectReadOnlyStore>()
        .and_then(|store| store.records.get_mut(&object.get_identity_hash().get()))
    else {
        return false;
    };
    change(record);
    true
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(RectRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.x)
}
fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.y)
}
fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.width)
}
fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.height)
}
fn get_top(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.y.min(rect.y + rect.height))
}
fn get_right(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.x.max(rect.x + rect.width))
}
fn get_bottom(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.y.max(rect.y + rect.height))
}
fn get_left(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |rect| rect.x.min(rect.x + rect.width))
}

fn to_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(rect) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let object = v8::Object::new(scope);
    define_number(scope, object, "x", rect.x);
    define_number(scope, object, "y", rect.y);
    define_number(scope, object, "width", rect.width);
    define_number(scope, object, "height", rect.height);
    define_number(scope, object, "top", rect.y.min(rect.y + rect.height));
    define_number(scope, object, "right", rect.x.max(rect.x + rect.width));
    define_number(scope, object, "bottom", rect.y.max(rect.y + rect.height));
    define_number(scope, object, "left", rect.x.min(rect.x + rect.width));
    result.set(object.into());
}

fn define_number(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    value: f64,
) {
    if let Some(key) = v8::String::new(scope, name) {
        let _ =
            object.create_data_property(scope, key.into(), v8::Number::new(scope, value).into());
    }
}

pub(crate) fn cleanup_realm(scope: &mut v8::PinScope<'_, '_>, realm_id: i32) {
    if let Some(store) = scope.get_slot_mut::<DomRectReadOnlyStore>() {
        store.constructor.remove(realm_id);
    }
}
