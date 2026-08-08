use std::collections::HashMap;

#[derive(Clone)]
struct SubImageRecord {
    viewport: v8::Global<v8::Object>,
}

#[derive(Default)]
pub(crate) struct XrSubImageStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, SubImageRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(XrSubImageStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "XRSubImage", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<XrSubImageStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "XRSubImage",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(scope, prototype, "viewport", get_viewport)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<XrSubImageStore>()
        .ok_or_else(|| "XRSubImage state missing".to_owned())?
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

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    let viewport = super::xr_viewport::create(scope)?;
    let viewport = v8::Global::new(scope, viewport);
    scope
        .get_slot_mut::<XrSubImageStore>()
        .ok_or_else(|| "XRSubImage state missing".to_owned())?
        .records
        .insert(
            object.get_identity_hash().get(),
            SubImageRecord { viewport },
        );
    Ok(())
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create XRSubImage".to_owned());
    }
    attach(scope, object)?;
    Ok(object)
}

fn get_viewport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(state) = scope
        .get_slot::<XrSubImageStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .cloned()
    {
        result.set(v8::Local::new(scope, &state.viewport).into());
    } else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
    }
}
