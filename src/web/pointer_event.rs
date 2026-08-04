use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct PointerEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, PointerRecord>,
}

#[derive(Clone)]
pub(crate) struct PointerRecord {
    pub(crate) pointer_id: i32,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) pressure: f32,
    pub(crate) tilt_x: i32,
    pub(crate) tilt_y: i32,
    pub(crate) azimuth_angle: f64,
    pub(crate) altitude_angle: f64,
    pub(crate) tangential_pressure: f32,
    pub(crate) twist: u32,
    pub(crate) pointer_type: String,
    pub(crate) is_primary: bool,
    pub(crate) persistent_device_id: i32,
}

pub(crate) fn is_pointer_event(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> bool {
    scope.get_slot::<PointerEventStore>().is_some_and(|store| {
        store
            .records
            .contains_key(&object.get_identity_hash().get())
    })
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(PointerEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "PointerEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<PointerEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "PointerEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::pointer_event_pointer_id_property::define(scope, prototype)?;
    super::pointer_event_width_property::define(scope, prototype)?;
    super::pointer_event_height_property::define(scope, prototype)?;
    super::pointer_event_pressure_property::define(scope, prototype)?;
    super::pointer_event_tilt_x_property::define(scope, prototype)?;
    super::pointer_event_tilt_y_property::define(scope, prototype)?;
    super::pointer_event_azimuth_angle_property::define(scope, prototype)?;
    super::pointer_event_altitude_angle_property::define(scope, prototype)?;
    super::pointer_event_tangential_pressure_property::define(scope, prototype)?;
    super::pointer_event_twist_property::define(scope, prototype)?;
    super::pointer_event_pointer_type_property::define(scope, prototype)?;
    super::pointer_event_is_primary_property::define(scope, prototype)?;
    super::pointer_event_get_predicted_events::define(scope, prototype)?;
    super::pointer_event_persistent_device_id_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    super::pointer_event_get_coalesced_events::define(scope, prototype)?;
    let parent = super::mouse_event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<PointerEventStore>()
        .ok_or_else(|| "PointerEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'PointerEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let mouse_data = super::mouse_event::read_init(scope, arguments.get(1));
    super::mouse_event::attach(scope, arguments.this(), event_type, mouse_data);
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let record = PointerRecord {
        pointer_id: integer(scope, init, "pointerId", 0),
        width: number(scope, init, "width", 1.0),
        height: number(scope, init, "height", 1.0),
        pressure: number(scope, init, "pressure", 0.0) as f32,
        tilt_x: integer(scope, init, "tiltX", 0),
        tilt_y: integer(scope, init, "tiltY", 0),
        azimuth_angle: number(scope, init, "azimuthAngle", 0.0),
        altitude_angle: number(scope, init, "altitudeAngle", std::f64::consts::FRAC_PI_2),
        tangential_pressure: number(scope, init, "tangentialPressure", 0.0) as f32,
        twist: number(scope, init, "twist", 0.0) as u32,
        pointer_type: string_property(scope, init, "pointerType").unwrap_or_default(),
        is_primary: boolean(scope, init, "isPrimary"),
        persistent_device_id: integer(scope, init, "persistentDeviceId", 0),
    };
    scope
        .get_slot_mut::<PointerEventStore>()
        .expect("PointerEvent state")
        .records
        .insert(arguments.this().get_identity_hash().get(), record);
    result.set(arguments.this().into());
}

pub(crate) fn get_coalesced_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let events = v8::Array::new(scope, 1);
    let _ = events.set_index(scope, 0, arguments.this().into());
    result.set(events.into());
}

pub(crate) fn number(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: f64,
) -> f64 {
    object
        .map(|object| super::event::number_property(scope, object, name, default))
        .unwrap_or(default)
}

pub(crate) fn integer(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default: i32,
) -> i32 {
    number(scope, object, name, f64::from(default)) as i32
}

pub(crate) fn boolean(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> bool {
    object
        .map(|object| super::event::boolean_property(scope, object, name))
        .unwrap_or(false)
}

pub(crate) fn string_property(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<String> {
    let object = object?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<PointerRecord> {
    scope
        .get_slot::<PointerEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_integer(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PointerRecord) -> i32,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&PointerRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_pointer_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_integer(s, a, r, |record| record.pointer_id);
}

pub(crate) fn get_width(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.width);
}

pub(crate) fn get_height(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.height);
}

pub(crate) fn get_pressure(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| f64::from(record.pressure));
}

pub(crate) fn get_tilt_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_integer(s, a, r, |record| record.tilt_x);
}

pub(crate) fn get_tilt_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_integer(s, a, r, |record| record.tilt_y);
}

pub(crate) fn get_azimuth_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.azimuth_angle);
}

pub(crate) fn get_altitude_angle(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.altitude_angle);
}

pub(crate) fn get_tangential_pressure(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| f64::from(record.tangential_pressure));
}

pub(crate) fn get_twist(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.twist).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_pointer_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        if let Some(value) = v8::String::new(scope, &record.pointer_type) {
            result.set(value.into());
        }
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_is_primary(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Boolean::new(scope, record.is_primary).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_predicted_events(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_some() {
        result.set(v8::Array::new(scope, 0).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_persistent_device_id(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_integer(s, a, r, |record| record.persistent_device_id);
}
