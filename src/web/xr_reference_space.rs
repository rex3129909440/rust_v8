use std::collections::HashMap;

#[derive(Clone, Default)]
struct ReferenceSpaceRecord {
    on_reset: Option<v8::Global<v8::Value>>,
}

#[derive(Default)]
pub(crate) struct XrReferenceSpaceStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ReferenceSpaceRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrReferenceSpaceStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRReferenceSpace", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<XrReferenceSpaceStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRReferenceSpace",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_accessor(scope, prototype, "onreset", get_on_reset, set_on_reset)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "getOffsetReferenceSpace",
        1,
        get_offset_reference_space,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::xr_space::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrReferenceSpaceStore>()
        .ok_or_else(|| "XRReferenceSpace state missing".to_owned())?
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
        return Err("cannot create XRReferenceSpace".to_owned());
    }
    super::event_target::attach(scope, object);
    scope
        .get_slot_mut::<XrReferenceSpaceStore>()
        .ok_or_else(|| "XRReferenceSpace state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            ReferenceSpaceRecord::default(),
        );
    Ok(object)
}

fn record(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<ReferenceSpaceRecord> {
    scope
        .get_slot::<XrReferenceSpaceStore>()?
        .records
        .get(&object.get_identity_hash().get())
        .cloned()
}

fn get_on_reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Some(record) = record(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    super::window_event_handler_support::return_handler(scope, record.on_reset, result);
}

fn set_on_reset(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    let Some(record) = scope
        .get_slot_mut::<XrReferenceSpaceStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    record.on_reset = handler;
}

fn get_offset_reference_space(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if record(scope, arguments.this()).is_none() {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    match create(scope) {
        Ok(reference_space) => result.set(reference_space.into()),
        Err(message) => crate::webidl::throw_type_error(scope, &message),
    }
}
