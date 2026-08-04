use std::collections::HashMap;

#[derive(Clone)]
struct AudioListenerRecord {
    position_x: v8::Global<v8::Object>,
    position_y: v8::Global<v8::Object>,
    position_z: v8::Global<v8::Object>,
    forward_x: v8::Global<v8::Object>,
    forward_y: v8::Global<v8::Object>,
    forward_z: v8::Global<v8::Object>,
    up_x: v8::Global<v8::Object>,
    up_y: v8::Global<v8::Object>,
    up_z: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct AudioListenerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, AudioListenerRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AudioListenerStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AudioListener", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<AudioListenerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AudioListener",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionX", get_position_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionY", get_position_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "positionZ", get_position_z)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "forwardX", get_forward_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "forwardY", get_forward_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "forwardZ", get_forward_z)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upX", get_up_x)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upY", get_up_y)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "upZ", get_up_z)?;
    crate::webidl::define_method(scope, prototype, "setOrientation", 6, set_orientation)?;
    crate::webidl::define_method(scope, prototype, "setPosition", 3, set_position)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AudioListenerStore>()
        .ok_or_else(|| "AudioListener state was not prepared".to_owned())?
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
        "Failed to construct 'AudioListener': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let listener = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, listener, prototype.into()) != Some(true) {
        return Err("cannot create AudioListener".to_owned());
    }
    let minimum = -f32::MAX;
    let maximum = f32::MAX;
    let position_x = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let position_y = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let position_z = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let forward_x = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let forward_y = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let forward_z = super::audio_param::create(scope, context, -1.0, minimum, maximum)?;
    let up_x = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let up_y = super::audio_param::create(scope, context, 1.0, minimum, maximum)?;
    let up_z = super::audio_param::create(scope, context, 0.0, minimum, maximum)?;
    let record = AudioListenerRecord {
        position_x: v8::Global::new(scope, position_x),
        position_y: v8::Global::new(scope, position_y),
        position_z: v8::Global::new(scope, position_z),
        forward_x: v8::Global::new(scope, forward_x),
        forward_y: v8::Global::new(scope, forward_y),
        forward_z: v8::Global::new(scope, forward_z),
        up_x: v8::Global::new(scope, up_x),
        up_y: v8::Global::new(scope, up_y),
        up_z: v8::Global::new(scope, up_z),
    };
    scope
        .get_slot_mut::<AudioListenerStore>()
        .ok_or_else(|| "AudioListener state was not prepared".to_owned())?
        .records
        .insert(listener.get_identity_hash().get(), record);
    Ok(listener)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<AudioListenerRecord> {
    scope
        .get_slot::<AudioListenerStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn return_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    select: impl FnOnce(&AudioListenerRecord) -> &v8::Global<v8::Object>,
) {
    if let Some(record) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, select(&record)).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn position_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<[f32; 3]> {
    let record = record(scope, object)?;
    Some([
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_x), time)?,
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_y), time)?,
        super::audio_param::value_at(scope, v8::Local::new(scope, &record.position_z), time)?,
    ])
}

fn get_position_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.position_x)
}
fn get_position_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.position_y)
}
fn get_position_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.position_z)
}
fn get_forward_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.forward_x)
}
fn get_forward_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.forward_y)
}
fn get_forward_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.forward_z)
}
fn get_up_x(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.up_x)
}
fn get_up_y(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.up_y)
}
fn get_up_z(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    return_parameter(s, a, r, |v| &v.up_z)
}

fn number(
    scope: &v8::PinScope<'_, '_>,
    arguments: &v8::FunctionCallbackArguments<'_>,
    index: i32,
) -> f32 {
    arguments.get(index).number_value(scope).unwrap_or(0.0) as f32
}

fn update_parameter(
    scope: &mut v8::PinScope<'_, '_>,
    parameter: &v8::Global<v8::Object>,
    value: f32,
) {
    let parameter = v8::Local::new(scope, parameter);
    super::audio_param::set_current_value(scope, parameter, value);
}

fn set_position(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    update_parameter(scope, &record.position_x, number(scope, &arguments, 0));
    update_parameter(scope, &record.position_y, number(scope, &arguments, 1));
    update_parameter(scope, &record.position_z, number(scope, &arguments, 2));
}

fn set_orientation(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    update_parameter(scope, &record.forward_x, number(scope, &arguments, 0));
    update_parameter(scope, &record.forward_y, number(scope, &arguments, 1));
    update_parameter(scope, &record.forward_z, number(scope, &arguments, 2));
    update_parameter(scope, &record.up_x, number(scope, &arguments, 3));
    update_parameter(scope, &record.up_y, number(scope, &arguments, 4));
    update_parameter(scope, &record.up_z, number(scope, &arguments, 5));
}
