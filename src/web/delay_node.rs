use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct DelayNodeStore {
    constructor: crate::webidl::RealmConstructor,
    delay_times: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DelayNodeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DelayNode", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DelayNodeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DelayNode",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "delayTime", get_delay_time)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::audio_node::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DelayNodeStore>()
        .ok_or_else(|| "DelayNode state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
    context: v8::Local<'_, v8::Object>,
    max_delay_time: f64,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let options = v8::Object::new(scope);
    let key = v8::String::new(scope, "maxDelayTime")
        .ok_or_else(|| "cannot create DelayNode options".to_owned())?;
    let value = v8::Number::new(scope, max_delay_time);
    if options.create_data_property(scope, key.into(), value.into()) != Some(true) {
        return Err("cannot set DelayNode options".to_owned());
    }
    constructor
        .new_instance(scope, &[context.into(), options.into()])
        .ok_or_else(|| "cannot create DelayNode".to_owned())
}

fn number_option(
    scope: &v8::PinScope<'_, '_>,
    options: Option<v8::Local<'_, v8::Object>>,
    name: &str,
    default_value: f64,
) -> f64 {
    options
        .and_then(|options| {
            let key = v8::String::new(scope, name)?;
            options.get(scope, key.into())
        })
        .filter(|value| !value.is_undefined())
        .and_then(|value| value.number_value(scope))
        .unwrap_or(default_value)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "DelayNode requires a BaseAudioContext");
        return;
    }
    let Ok(context) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DelayNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    };
    if !super::base_audio_context::is_context(scope, context) {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'DelayNode': parameter 1 is not of type 'BaseAudioContext'.",
        );
        return;
    }
    let options = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let maximum = number_option(scope, options, "maxDelayTime", 1.0);
    let initial = number_option(scope, options, "delayTime", 0.0);
    if !maximum.is_finite() || maximum <= 0.0 || initial < 0.0 || initial > maximum {
        crate::webidl::throw_type_error(scope, "DelayNode delay is outside the permitted range");
        return;
    }
    let delay_time =
        match super::audio_param::create(scope, context, initial as f32, 0.0, maximum as f32) {
            Ok(delay_time) => delay_time,
            Err(message) => {
                crate::webidl::throw_type_error(scope, &message);
                return;
            }
        };
    super::audio_node::attach(scope, arguments.this(), Some(context), 1, 1);
    let delay_time = v8::Global::new(scope, delay_time);
    scope
        .get_slot_mut::<DelayNodeStore>()
        .expect("DelayNode state")
        .delay_times
        .insert(arguments.this().get_identity_hash().get(), delay_time);
    result.set(arguments.this().into());
}

fn get_delay_time(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(delay_time) = scope
        .get_slot::<DelayNodeStore>()
        .and_then(|store| {
            store
                .delay_times
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &delay_time).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}

pub(crate) fn delay_at(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    time: f64,
) -> Option<f32> {
    let delay_time = scope
        .get_slot::<DelayNodeStore>()?
        .delay_times
        .get(&object.get_identity_hash().get())?;
    super::audio_param::value_at(scope, v8::Local::new(scope, delay_time), time)
}
