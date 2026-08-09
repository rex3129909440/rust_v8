use std::collections::HashMap;

#[derive(Clone)]
struct GamepadRecord {
    id: String,
    index: u32,
    connected: bool,
    timestamp: f64,
    mapping: String,
    axes: v8::Global<v8::Array>,
    buttons: v8::Global<v8::Array>,
    actuator: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct GamepadStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, GamepadRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(GamepadStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Gamepad", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<GamepadStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "Gamepad",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "id", get_id)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "index", get_index)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "connected", get_connected)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "timestamp", get_timestamp)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "mapping", get_mapping)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "axes", get_axes)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "buttons", get_buttons)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "vibrationActuator",
        get_vibration_actuator,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<GamepadStore>()
        .ok_or_else(|| "Gamepad state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    id: &str,
    index: u32,
    connected: bool,
    mapping: &str,
    axes_values: &[f64],
    button_values: &[f64],
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create Gamepad".to_owned());
    }
    let axes = v8::Array::new(scope, axes_values.len() as i32);
    for (index, value) in axes_values.iter().enumerate() {
        let _ = axes.set_index(scope, index as u32, v8::Number::new(scope, *value).into());
    }
    let buttons = v8::Array::new(scope, button_values.len() as i32);
    for (index, value) in button_values.iter().enumerate() {
        let button = super::gamepad_button::create(scope, *value, *value >= 1.0, *value > 0.0)?;
        let _ = buttons.set_index(scope, index as u32, button.into());
    }
    let actuator = super::gamepad_haptic_actuator::create(scope)?;
    let timestamp = super::performance::now_for_current_realm(scope).unwrap_or_else(|| {
        crate::determinism::relative_high_resolution_milliseconds(
            scope,
            crate::determinism::elapsed_milliseconds(scope),
            0.0,
        )
    });
    let record = GamepadRecord {
        id: id.to_owned(),
        index,
        connected,
        timestamp,
        mapping: mapping.to_owned(),
        axes: v8::Global::new(scope, axes),
        buttons: v8::Global::new(scope, buttons),
        actuator: v8::Global::new(scope, actuator),
    };
    scope
        .get_slot_mut::<GamepadStore>()
        .ok_or_else(|| "Gamepad state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), record);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<GamepadRecord> {
    scope
        .get_slot::<GamepadStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_string(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&GamepadRecord) -> &str,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, select(&record)) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.id);
}
fn get_mapping(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_string(s, a, r, |record| &record.mapping);
}
fn get_index(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Integer::new_from_unsigned(s, record.index).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_connected(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, record.connected).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_timestamp(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Number::new(s, record.timestamp).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_axes(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Local::new(s, &record.axes).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_buttons(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Local::new(s, &record.buttons).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
fn get_vibration_actuator(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(s, a.this()) {
        r.set(v8::Local::new(s, &record.actuator).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
