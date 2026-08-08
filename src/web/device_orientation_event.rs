use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DeviceOrientationEventRecord {
    pub(crate) alpha: Option<f64>,
    pub(crate) beta: Option<f64>,
    pub(crate) gamma: Option<f64>,
    pub(crate) absolute: bool,
}
#[derive(Default)]
pub(crate) struct DeviceOrientationEventStore {
    pub(crate) constructor: crate::webidl::RealmConstructor,
    pub(crate) records: HashMap<i32, DeviceOrientationEventRecord>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(DeviceOrientationEventStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "DeviceOrientationEvent", constructor.into())
}
pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<DeviceOrientationEventStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "DeviceOrientationEvent",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::device_orientation_event_alpha_property::define(scope, prototype)?;
    super::device_orientation_event_beta_property::define(scope, prototype)?;
    super::device_orientation_event_gamma_property::define(scope, prototype)?;
    super::device_orientation_event_absolute_property::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<DeviceOrientationEventStore>()
        .ok_or_else(|| "DeviceOrientationEvent state was not prepared".to_owned())?
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
        .ok_or_else(|| "cannot create DeviceOrientationEvent".to_owned())
}

pub(crate) fn member<'s>(
    scope: &v8::PinScope<'s, '_>,
    object: Option<v8::Local<'_, v8::Object>>,
    name: &str,
) -> Option<v8::Local<'s, v8::Value>> {
    let key = v8::String::new(scope, name)?;
    object?.get(scope, key.into())
}
pub(crate) fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() || arguments.length() < 1 {
        crate::webidl::throw_type_error(scope, "DeviceOrientationEvent requires an event type");
        return;
    }
    let init = v8::Local::<v8::Object>::try_from(arguments.get(1)).ok();
    let (bubbles, cancelable, composed) = super::event::event_init(scope, arguments.get(1));
    super::event::attach(
        scope,
        arguments.this(),
        crate::webidl::value_to_string(scope, arguments.get(0)),
        bubbles,
        cancelable,
        composed,
    );
    let number = |name| {
        member(scope, init, name)
            .filter(|v| !v.is_null())
            .and_then(|v| v.number_value(scope))
    };
    let alpha = number("alpha");
    let beta = number("beta");
    let gamma = number("gamma");
    let absolute = member(scope, init, "absolute").is_some_and(|v| v.boolean_value(scope));
    scope
        .get_slot_mut::<DeviceOrientationEventStore>()
        .expect("DeviceOrientationEvent state")
        .records
        .insert(
            arguments.this().get_identity_hash().get(),
            DeviceOrientationEventRecord {
                alpha,
                beta,
                gamma,
                absolute,
            },
        );
    result.set(arguments.this().into())
}
pub(crate) fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<DeviceOrientationEventRecord> {
    scope
        .get_slot::<DeviceOrientationEventStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}
pub(crate) fn number_get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
    field: impl FnOnce(DeviceOrientationEventRecord) -> Option<f64>,
) {
    let x = record(scope, arguments.this());
    match x.clone().and_then(field) {
        Some(v) => result.set(v8::Number::new(scope, v).into()),
        None if x.is_some() => result.set(v8::null(scope).into()),
        None => crate::webidl::throw_type_error(scope, "Illegal invocation"),
    }
}
pub(crate) fn get_alpha(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.alpha)
}
pub(crate) fn get_beta(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.beta)
}
pub(crate) fn get_gamma(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    r: v8::ReturnValue<'_>,
) {
    number_get(s, a, r, |x| x.gamma)
}
pub(crate) fn get_absolute(
    s: &mut v8::PinScope<'_, '_>,
    a: v8::FunctionCallbackArguments<'_>,
    mut r: v8::ReturnValue<'_>,
) {
    if let Some(x) = record(s, a.this()) {
        r.set(v8::Boolean::new(s, x.absolute).into())
    } else {
        crate::webidl::throw_type_error(s, "Illegal invocation")
    }
}
