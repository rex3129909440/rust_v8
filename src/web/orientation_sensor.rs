use std::collections::HashMap;

#[derive(Clone, Copy)]
pub(crate) enum OrientationKind {
    Absolute,
    Relative,
}

#[derive(Default)]
pub(crate) struct OrientationSensorStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashMap<i32, OrientationKind>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OrientationSensorStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "OrientationSensor", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<OrientationSensorStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let parent = super::sensor::ensure_constructor(scope)?;
    let constructor = crate::webidl::create_function(
        scope,
        "OrientationSensor",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "quaternion", get_quaternion)?;
    crate::webidl::define_method(scope, prototype, "populateMatrix", 1, populate_matrix)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<OrientationSensorStore>()
        .ok_or_else(|| "OrientationSensor state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'OrientationSensor': Illegal constructor",
    );
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    kind: OrientationKind,
) {
    super::sensor::attach(scope, object);
    scope
        .get_slot_mut::<OrientationSensorStore>()
        .expect("OrientationSensor state")
        .instances
        .insert(object.get_identity_hash().get(), kind);
}

fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<OrientationSensorStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains_key(&object.get_identity_hash().get())
        })
}

fn quaternion(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> Option<[f64; 4]> {
    let kind = scope
        .get_slot::<OrientationSensorStore>()?
        .instances
        .get(&object.get_identity_hash().get())?;
    let sensors = &crate::fingerprint::edge(scope).sensors;
    Some(match kind {
        OrientationKind::Absolute => sensors.absolute_orientation_quaternion,
        OrientationKind::Relative => sensors.relative_orientation_quaternion,
    })
}

fn get_quaternion(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let array = v8::Array::new(scope, 4);
    let values = quaternion(scope, arguments.this()).expect("valid OrientationSensor");
    for (index, value) in values.into_iter().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Number::new(scope, value).into());
    }
    result.set(array.into());
}

fn populate_matrix(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if !valid(scope, arguments.this()) {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    let Ok(matrix) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "populateMatrix requires an array-like object");
        return;
    };
    let [x, y, z, w] = quaternion(scope, arguments.this()).expect("valid OrientationSensor");
    let values = [
        1.0 - 2.0 * (y * y + z * z),
        2.0 * (x * y + z * w),
        2.0 * (x * z - y * w),
        0.0,
        2.0 * (x * y - z * w),
        1.0 - 2.0 * (x * x + z * z),
        2.0 * (y * z + x * w),
        0.0,
        2.0 * (x * z + y * w),
        2.0 * (y * z - x * w),
        1.0 - 2.0 * (x * x + y * y),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ];
    for (index, value) in values.into_iter().enumerate() {
        let _ = matrix.set_index(scope, index as u32, v8::Number::new(scope, value).into());
    }
}
