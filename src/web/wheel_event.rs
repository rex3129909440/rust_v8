use std::collections::HashMap;

pub(crate) const DOM_DELTA_PIXEL: i32 = 0;
pub(crate) const DOM_DELTA_LINE: i32 = 1;
pub(crate) const DOM_DELTA_PAGE: i32 = 2;

#[derive(Default)]
pub(crate) struct WheelEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, WheelEventRecord>,
}

#[derive(Clone)]
pub(crate) struct WheelEventRecord {
    pub(crate) delta_x: f64,
    pub(crate) delta_y: f64,
    pub(crate) delta_z: f64,
    pub(crate) delta_mode: u32,
    pub(crate) wheel_delta_x: f64,
    pub(crate) wheel_delta_y: f64,
    pub(crate) wheel_delta: f64,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WheelEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "WheelEvent", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<WheelEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let parent = super::mouse_event::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "WheelEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::wheel_event_delta_x_property::define(scope, prototype)?;
    super::wheel_event_delta_y_property::define(scope, prototype)?;
    super::wheel_event_delta_z_property::define(scope, prototype)?;
    super::wheel_event_delta_mode_property::define(scope, prototype)?;
    super::wheel_event_wheel_delta_x_property::define(scope, prototype)?;
    super::wheel_event_wheel_delta_y_property::define(scope, prototype)?;
    super::wheel_event_wheel_delta_property::define(scope, prototype)?;
    define_wheel_constants(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "momentum", get_momentum)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    define_wheel_constants(scope, constructor.into())?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<WheelEventStore>()
        .ok_or_else(|| "WheelEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn get_momentum(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    result.set(v8::Boolean::new(scope, false).into());
}

pub(crate) fn define_wheel_constants(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_constant(scope, object, "DOM_DELTA_PIXEL", DOM_DELTA_PIXEL)?;
    crate::webidl::define_constant(scope, object, "DOM_DELTA_LINE", DOM_DELTA_LINE)?;
    crate::webidl::define_constant(scope, object, "DOM_DELTA_PAGE", DOM_DELTA_PAGE)
}

pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WheelEvent': Please use the 'new' operator",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'WheelEvent': 1 argument required",
        );
        return;
    }
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let mouse_data = super::mouse_event::read_init(scope, arguments.get(1));
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let delta_x = init
        .map(|init| super::event::number_property(scope, init, "deltaX", 0.0))
        .unwrap_or(0.0);
    let delta_y = init
        .map(|init| super::event::number_property(scope, init, "deltaY", 0.0))
        .unwrap_or(0.0);
    let delta_z = init
        .map(|init| super::event::number_property(scope, init, "deltaZ", 0.0))
        .unwrap_or(0.0);
    let record = WheelEventRecord {
        delta_x,
        delta_y,
        delta_z,
        delta_mode: init
            .map(|init| super::event::number_property(scope, init, "deltaMode", 0.0) as u32)
            .unwrap_or(0),
        // Chromium preserves constructor deltas in the legacy fields. Trusted
        // hardware input supplies separately normalized wheel ticks.
        wheel_delta_x: delta_x,
        wheel_delta_y: delta_y,
        wheel_delta: if delta_y != 0.0 { delta_y } else { delta_x },
    };
    let object = arguments.this();
    super::mouse_event::attach(scope, object, event_type, mouse_data);
    scope
        .get_slot_mut::<WheelEventStore>()
        .expect("WheelEvent state")
        .records
        .insert(object.get_identity_hash().get(), record);
    result.set(object.into());
}

pub(crate) fn create_with_data<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    mouse_data: super::mouse_event::MouseEventData,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    delta_mode: u32,
    wheel_delta_x: f64,
    wheel_delta_y: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let event = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, event, prototype.into()) != Some(true) {
        return Err("cannot create WheelEvent".to_owned());
    }
    super::mouse_event::attach(scope, event, "wheel".to_owned(), mouse_data);
    scope
        .get_slot_mut::<WheelEventStore>()
        .ok_or_else(|| "WheelEvent state was not prepared".to_owned())?
        .records
        .insert(
            event.get_identity_hash().get(),
            WheelEventRecord {
                delta_x,
                delta_y,
                delta_z,
                delta_mode,
                wheel_delta_x,
                wheel_delta_y,
                wheel_delta: if wheel_delta_y != 0.0 {
                    wheel_delta_y
                } else {
                    wheel_delta_x
                },
            },
        );
    Ok(event)
}

pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<WheelEventRecord> {
    scope
        .get_slot::<WheelEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn return_number(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&WheelEventRecord) -> f64,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Number::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_delta_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.delta_x);
}
pub(crate) fn get_delta_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.delta_y);
}
pub(crate) fn get_delta_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.delta_z);
}

pub(crate) fn get_delta_mode(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Integer::new_from_unsigned(scope, record.delta_mode).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn get_wheel_delta_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.wheel_delta_x);
}
pub(crate) fn get_wheel_delta_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.wheel_delta_y);
}
pub(crate) fn get_wheel_delta(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_number(s, a, r, |record| record.wheel_delta);
}
