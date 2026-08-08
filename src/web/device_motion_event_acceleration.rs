use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct AccelerationRecord {
    pub(crate) x: Option<f64>,
    pub(crate) y: Option<f64>,
    pub(crate) z: Option<f64>,
}

#[derive(Default)]
pub(crate) struct DeviceMotionEventAccelerationStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, AccelerationRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DeviceMotionEventAccelerationStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DeviceMotionEventAcceleration", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DeviceMotionEventAccelerationStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DeviceMotionEventAcceleration",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::device_motion_event_acceleration_x_property::define(scope, prototype)?;
    super::device_motion_event_acceleration_y_property::define(scope, prototype)?;
    super::device_motion_event_acceleration_z_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DeviceMotionEventAccelerationStore>()
        .ok_or_else(|| "DeviceMotionEventAcceleration state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}
pub(crate) fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'DeviceMotionEventAcceleration': Illegal constructor",
    );
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create DeviceMotionEventAcceleration".to_owned());
    }
    scope
        .get_slot_mut::<DeviceMotionEventAccelerationStore>()
        .ok_or_else(|| "DeviceMotionEventAcceleration state was not prepared".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            AccelerationRecord { x, y, z },
        );
    Ok(object)
}
pub(crate) fn coordinate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    value: impl FnOnce(AccelerationRecord) -> Option<f64>,
) {
    let record = scope
        .get_slot::<DeviceMotionEventAccelerationStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    match record.clone().and_then(value) {
        Some(value) => result.set(v8::Number::new(scope, value).into()),
        None if record.is_some() => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
pub(crate) fn get_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, |x| x.x);
}
pub(crate) fn get_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, |x| x.y);
}
pub(crate) fn get_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    coordinate(s, a, r, |x| x.z);
}
