use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct AbsoluteOrientationSensorStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AbsoluteOrientationSensorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AbsoluteOrientationSensor", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AbsoluteOrientationSensorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::orientation_sensor::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "AbsoluteOrientationSensor",
        0,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AbsoluteOrientationSensorStore>()
        .ok_or_else(|| "AbsoluteOrientationSensor state was not prepared".to_owned())?
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
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'AbsoluteOrientationSensor': Please use the 'new' operator.",
        );
        return;
    }
    super::orientation_sensor::attach(
        scope,
        arguments.this(),
        super::orientation_sensor::OrientationKind::Absolute,
    );
    scope
        .get_slot_mut::<AbsoluteOrientationSensorStore>()
        .expect("AbsoluteOrientationSensor state")
        .instances
        .insert(arguments.this().get_identity_hash().get());
    result.set(arguments.this().into());
}
