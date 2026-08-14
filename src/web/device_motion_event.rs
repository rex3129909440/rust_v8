use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DeviceMotionEventRecord {
    pub(crate) acceleration: Option<v8::Global<v8::Object>>,
    pub(crate) acceleration_including_gravity: Option<v8::Global<v8::Object>>,
    pub(crate) rotation_rate: Option<v8::Global<v8::Object>>,
    pub(crate) interval: f64,
}
#[derive(Default)]
pub(crate) struct DeviceMotionEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, DeviceMotionEventRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DeviceMotionEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DeviceMotionEvent", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DeviceMotionEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DeviceMotionEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::device_motion_event_acceleration_property::define(scope, prototype)?;
    super::device_motion_event_acceleration_including_gravity_property::define(scope, prototype)?;
    super::device_motion_event_rotation_rate_property::define(scope, prototype)?;
    super::device_motion_event_interval_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DeviceMotionEventStore>()
        .ok_or_else(|| "DeviceMotionEvent state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let event_type = crate::webidl::string(scope, "")?;
    constructor
        .new_instance(scope, &[event_type.into()])
        .ok_or_else(|| "cannot create DeviceMotionEvent".to_owned())
}

pub(crate) fn numeric(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<f64> {
    let key = v8::String::new(scope, name)?;
    object?.get(scope, key.into())?.number_value(scope)
}
pub(crate) fn object_member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Object>> {
    let key = v8::String::new(scope, name)?;
    let value = object?.get(scope, key.into())?;
    v8::Local::<v8::Object>::try_from(value).ok()
}
pub(crate) fn acceleration_values(
    scope: &v8::PinScope<'_, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    (
        numeric(scope, object, "x"),
        numeric(scope, object, "y"),
        numeric(scope, object, "z"),
    )
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "DeviceMotionEvent requires an event type");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let event_type = crate::webidl::value_to_string(scope, arguments.get(0));
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        event_type,
        bubbles,
        cancelable,
        composed,
    );
    let acceleration = object_member(scope, init, "acceleration").map(|value| {
        let (x, y, z) = acceleration_values(scope, Some(value));
        let object = super::device_motion_event_acceleration::create(scope, x, y, z).unwrap();
        v8::Global::new(scope, object)
    });
    let gravity = object_member(scope, init, "accelerationIncludingGravity").map(|value| {
        let (x, y, z) = acceleration_values(scope, Some(value));
        let object = super::device_motion_event_acceleration::create(scope, x, y, z).unwrap();
        v8::Global::new(scope, object)
    });
    let rotation = object_member(scope, init, "rotationRate").map(|value| {
        let alpha = numeric(scope, Some(value), "alpha");
        let beta = numeric(scope, Some(value), "beta");
        let gamma = numeric(scope, Some(value), "gamma");
        let object =
            super::device_motion_event_rotation_rate::create(scope, alpha, beta, gamma).unwrap();
        v8::Global::new(scope, object)
    });
    let interval = numeric(scope, init, "interval").unwrap_or(0.0);
    scope
        .get_slot_mut::<DeviceMotionEventStore>()
        .expect("DeviceMotionEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            DeviceMotionEventRecord {
                acceleration,
                acceleration_including_gravity: gravity,
                rotation_rate: rotation,
                interval,
            },
        );
    result.set(arguments.this().into());
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DeviceMotionEventRecord> {
    scope
        .get_slot::<DeviceMotionEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn get_acceleration(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        match x.acceleration {
            Some(value) => r.set(v8::Local::new(s, &value).into()),
            None => r.set(v8::null(s).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_acceleration_including_gravity(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        match x.acceleration_including_gravity {
            Some(value) => r.set(v8::Local::new(s, &value).into()),
            None => r.set(v8::null(s).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_rotation_rate(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        match x.rotation_rate {
            Some(value) => r.set(v8::Local::new(s, &value).into()),
            None => r.set(v8::null(s).into()),
        }
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
pub(crate) fn get_interval(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Number::new(s, x.interval).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
