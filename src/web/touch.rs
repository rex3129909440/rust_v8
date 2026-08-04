use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct TouchStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, TouchRecord>,
}

#[derive(Clone)]
struct TouchRecord {
    identifier: i32,
    target: v8::Global<v8::Object>,
    screen_x: f64,
    screen_y: f64,
    client_x: f64,
    client_y: f64,
    page_x: f64,
    page_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation_angle: f64,
    force: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(TouchStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Touch", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<TouchStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Touch",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "identifier", get_identifier)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "target", get_target)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "screenX", get_screen_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "screenY", get_screen_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "clientX", get_client_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "clientY", get_client_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pageX", get_page_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "pageY", get_page_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "radiusX", get_radius_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "radiusY", get_radius_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "rotationAngle", get_rotation_angle)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "force", get_force)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<TouchStore>()
        .ok_or_else(|| "Touch state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Failed to construct 'Touch': use new");
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "1 argument required");
        return;
    }
    let Ok(init) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "TouchInit must be an object");
        return;
    };
    let Some(target) = object_property(scope, init, "target") else {
        crate::webidl::throw_type_error(scope, "TouchInit.target is required");
        return;
    };
    let record = TouchRecord {
        identifier: integer_property(scope, init, "identifier", 0),
        target: v8::Global::new(scope, target),
        screen_x: number_property(scope, init, "screenX", 0.0),
        screen_y: number_property(scope, init, "screenY", 0.0),
        client_x: number_property(scope, init, "clientX", 0.0),
        client_y: number_property(scope, init, "clientY", 0.0),
        page_x: number_property(scope, init, "pageX", 0.0),
        page_y: number_property(scope, init, "pageY", 0.0),
        radius_x: number_property(scope, init, "radiusX", 0.0),
        radius_y: number_property(scope, init, "radiusY", 0.0),
        rotation_angle: number_property(scope, init, "rotationAngle", 0.0),
        force: number_property(scope, init, "force", 0.0),
    };
    let object = arguments.this();
    scope
        .get_slot_mut::<TouchStore>()
        .expect("Touch state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope.get_slot::<TouchStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

fn record(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<TouchRecord> {
    scope
        .get_slot::<TouchStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn object_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    v8::Local::<v8::Object>::try_from(value).ok()
}

fn number_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: f64,
) -> f64 {
    let Some(key) = v8::String::new(scope, name) else {
        return default;
    };
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default)
}

fn integer_property(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    default: i32,
) -> i32 {
    let Some(key) = v8::String::new(scope, name) else {
        return default;
    };
    object
        .get(scope, key.into())
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.int32_value(scope))
        .unwrap_or(default)
}

fn get_identifier(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, record.identifier).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_target(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &record.target).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&TouchRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_screen_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.screen_x);
}
fn get_screen_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.screen_y);
}
fn get_client_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_x);
}
fn get_client_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.client_y);
}
fn get_page_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.page_x);
}
fn get_page_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.page_y);
}
fn get_radius_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.radius_x);
}
fn get_radius_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.radius_y);
}
fn get_rotation_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.rotation_angle);
}
fn get_force(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |v| v.force);
}
