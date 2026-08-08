use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct InputDeviceInfoStore {
    constructor: crate::webidl::RealmConstructor,
    capabilities: HashMap<i32, InputCapabilities>,
}

#[derive(Clone, Default)]
pub(crate) struct InputCapabilities {
    pub auto_gain_control: Vec<bool>,
    pub echo_cancellation: Vec<bool>,
    pub noise_suppression: Vec<bool>,
    pub sample_rates: Vec<f64>,
    pub channel_counts: Vec<f64>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(InputDeviceInfoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "InputDeviceInfo", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let existing = scope
        .get_slot::<InputDeviceInfoStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        return Ok(v8::Local::new(scope, &existing));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "InputDeviceInfo",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "getCapabilities", 0, get_capabilities)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::media_device_info::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<InputDeviceInfoStore>()
        .ok_or_else(|| "InputDeviceInfo state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    device_id: String,
    kind: String,
    label: String,
    group_id: String,
    capabilities: InputCapabilities,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create InputDeviceInfo".to_owned());
    }
    super::media_device_info::attach(scope, object, device_id, kind, label, group_id);
    scope
        .get_slot_mut::<InputDeviceInfoStore>()
        .ok_or_else(|| "InputDeviceInfo state was not prepared".to_owned())?
        .capabilities
        .insert(object.get_identity_hash().get(), capabilities);
    Ok(object)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'InputDeviceInfo': Illegal constructor",
    );
}

fn get_capabilities(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let capabilities = scope
        .get_slot::<InputDeviceInfoStore>()
        .and_then(|store| {
            store
                .capabilities
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned();
    let Some(capabilities) = capabilities else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let output = v8::Object::new(scope);
    define_bool_array(
        scope,
        output,
        "autoGainControl",
        &capabilities.auto_gain_control,
    );
    define_number_array(scope, output, "channelCount", &capabilities.channel_counts);
    define_bool_array(
        scope,
        output,
        "echoCancellation",
        &capabilities.echo_cancellation,
    );
    define_bool_array(
        scope,
        output,
        "noiseSuppression",
        &capabilities.noise_suppression,
    );
    define_number_array(scope, output, "sampleRate", &capabilities.sample_rates);
    result.set(output.into());
}

fn define_bool_array(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    values: &[bool],
) {
    if values.is_empty() {
        return;
    }
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().copied().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Boolean::new(scope, value).into());
    }
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), array.into());
    }
}

fn define_number_array(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    name: &str,
    values: &[f64],
) {
    if values.is_empty() {
        return;
    }
    let array = v8::Array::new(scope, values.len() as i32);
    for (index, value) in values.iter().copied().enumerate() {
        let _ = array.set_index(scope, index as u32, v8::Number::new(scope, value).into());
    }
    if let Some(key) = v8::String::new(scope, name) {
        let _ = object.set(scope, key.into(), array.into());
    }
}
