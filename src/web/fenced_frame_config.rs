use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct FencedFrameConfigStore {
    constructor: crate::webidl::RealmConstructor,
    contexts: HashMap<i32, String>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(FencedFrameConfigStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "FencedFrameConfig", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<FencedFrameConfigStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "FencedFrameConfig",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "setSharedStorageContext",
        1,
        set_shared_storage_context,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<FencedFrameConfigStore>()
        .ok_or_else(|| "FencedFrameConfig state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

#[allow(dead_code)]
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let config = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, config, prototype.into()) != Some(true) {
        return Err("cannot create FencedFrameConfig".to_owned());
    }
    scope
        .get_slot_mut::<FencedFrameConfigStore>()
        .ok_or_else(|| "FencedFrameConfig state was not prepared".to_owned())?
        .contexts
        .insert(config.get_identity_hash().get(), String::new());
    Ok(config)
}

pub(crate) fn is_instance(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<FencedFrameConfigStore>()
        .is_some_and(|store| {
            store
                .contexts
                .contains_key(&object.get_identity_hash().get())
        })
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'FencedFrameConfig': Illegal constructor",
    );
}

fn set_shared_storage_context(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to execute 'setSharedStorageContext' on 'FencedFrameConfig': 1 argument required, but only 0 present.",
        );
        return;
    }
    let context = crate::webidl::value_to_string(scope, arguments.get(0));
    let Some(value) = scope
        .get_slot_mut::<FencedFrameConfigStore>()
        .and_then(|store| {
            store
                .contexts
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    *value = context;
}
