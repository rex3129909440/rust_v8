#[derive(Default)]
pub(crate) struct LinearAccelerationSensorStore {
    constructor: crate::webidl::RealmConstructor,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LinearAccelerationSensorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "LinearAccelerationSensor", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<LinearAccelerationSensorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "LinearAccelerationSensor",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let parent = super::accelerometer::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::define_to_string_tag(scope, prototype, "LinearAccelerationSensor")?;
    crate::webidl::lock_constructor_prototype(scope, constructor)?;
    let persistent = v8::Global::new(scope, constructor);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<LinearAccelerationSensorStore>()
        .ok_or_else(|| "LinearAccelerationSensor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, persistent);
    Ok(constructor)
}
fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(scope, "Please use the 'new' operator");
        return;
    }
    super::accelerometer::attach(
        scope,
        arguments.this(),
        super::accelerometer::AccelerometerKind::LinearAcceleration,
    );
    result.set(arguments.this().into())
}
