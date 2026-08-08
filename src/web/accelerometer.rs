use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) enum AccelerometerKind {
    Accelerometer,
    Gravity,
    LinearAcceleration,
}

#[derive(Default)]
pub(crate) struct AccelerometerStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashMap<i32, AccelerometerKind>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AccelerometerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "Accelerometer", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AccelerometerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::sensor::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "Accelerometer",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "x", get_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "y", get_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "z", get_z)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AccelerometerStore>()
        .ok_or_else(|| "Accelerometer state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: AccelerometerKind,
) {
    super::sensor::attach(scope, object);
    if let Some(store) = scope.get_slot_mut::<AccelerometerStore>() {
        store
            .instances
            .insert(object.get_identity_hash().get(), kind);
    }
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'Accelerometer': Please use the 'new' operator.",
        );
        return;
    }
    attach(scope, arguments.this(), AccelerometerKind::Accelerometer);
    result.set(arguments.this().into());
}

fn coordinate(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    component: usize,
) {
    let kind = scope
        .get_slot::<AccelerometerStore>()
        .and_then(|store| {
            store
                .instances
                .get(&arguments.this().get_identity_hash().get())
        })
        .copied();
    if let Some(kind) = kind {
        let sensors = &crate::fingerprint::edge(scope).sensors;
        let vector = match kind {
            AccelerometerKind::Accelerometer => sensors.accelerometer,
            AccelerometerKind::Gravity => sensors.gravity,
            AccelerometerKind::LinearAcceleration => sensors.linear_acceleration,
        };
        result.set(v8::Number::new(scope, vector[component]).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

fn get_x(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    coordinate(scope, arguments, result, 0);
}
fn get_y(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    coordinate(scope, arguments, result, 1);
}
fn get_z(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    coordinate(scope, arguments, result, 2);
}
