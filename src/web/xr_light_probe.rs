use std::collections::HashMap;

#[derive(Clone)]
struct LightProbeRecord {
    space: v8::Global<v8::Object>,
    on_reflection_change: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrLightProbeStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, LightProbeRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrLightProbeStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRLightProbe", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrLightProbeStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRLightProbe",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "probeSpace", get_probe_space)?;
    crate::webidl::define_accessor(
        scope,
        prototype,
        "onreflectionchange",
        get_on_reflection_change,
        set_on_reflection_change,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::event_target::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrLightProbeStore>()
        .ok_or_else(|| "XRLightProbe state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor")
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRLightProbe".to_owned());
    }
    super::event_target::attach(scope, object);
    let space = super::xr_space::create(scope)?;
    let space = v8::Global::new(scope, space);
    let identity = object.get_identity_hash().get();
    scope
        .get_slot_mut::<XrLightProbeStore>()
        .ok_or_else(|| "XRLightProbe state missing".to_owned())?
        .records
        .insert(
            identity,
            LightProbeRecord {
                space,
                on_reflection_change: None,
            },
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<LightProbeRecord> {
    scope
        .get_slot::<XrLightProbeStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_probe_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(probe) = record(scope, arguments.this()) {
        result.set(v8::Local::new(scope, &probe.space).into())
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation")
    }
}

fn get_on_reflection_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(probe) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, probe.on_reflection_change, result);
}

fn set_on_reflection_change(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(probe) = scope.get_slot_mut::<XrLightProbeStore>().and_then(|store| {
        store
            .records
            .get_mut(&arguments.this().get_identity_hash().get())
    }) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    probe.on_reflection_change = handler;
}
