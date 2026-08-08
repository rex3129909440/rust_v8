use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct XrDomOverlayStateStore {
    constructor: crate::webidl::RealmConstructor,
    instances: HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrDomOverlayStateStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRDOMOverlayState", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrDomOverlayStateStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRDOMOverlayState",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "type", get_type)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrDomOverlayStateStore>()
        .ok_or_else(|| "XRDOMOverlayState state missing".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(scope, "Illegal constructor");
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRDOMOverlayState".to_owned());
    }
    scope
        .get_slot_mut::<XrDomOverlayStateStore>()
        .ok_or_else(|| "XRDOMOverlayState state missing".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

fn get_type(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let valid = scope
        .get_slot::<XrDomOverlayStateStore>()
        .is_some_and(|store| {
            store
                .instances
                .contains(&arguments.this().get_identity_hash().get())
        });
    if !valid {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    if let Some(value) = v8::String::new(scope, "screen") {
        result.set(value.into());
    }
}
