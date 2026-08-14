#[derive(Default)]
pub(crate) struct SharedStorageDeleteMethodStore {
    constructor: crate::webidl::RealmConstructor,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SharedStorageDeleteMethodStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedStorageDeleteMethod", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SharedStorageDeleteMethodStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SharedStorageDeleteMethod",
        1,
        v8::ConstructorBehavior::Allow,
        construct,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let parent = super::shared_storage_modifier_method::ensure_constructor(scope)?;
    crate::webidl::inherit(scope, constructor, parent)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SharedStorageDeleteMethodStore>()
        .ok_or_else(|| "SharedStorageDeleteMethod state was not prepared".to_owned())?
        .constructor
        .insert(realm_id, realm_constructor);
    Ok(constructor)
}

fn construct(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if !arguments.is_construct_call() {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SharedStorageDeleteMethod': Please use the 'new' operator.",
        );
        return;
    }
    if arguments.length() < 1 {
        crate::webidl::throw_type_error(
            scope,
            "Failed to construct 'SharedStorageDeleteMethod': 1 argument required, but only 0 present.",
        );
        return;
    }
    let Some(key) = crate::webidl::dom_string(scope, arguments.get(0)) else {
        return;
    };
    if key.is_empty() {
        if let Ok(exception) = super::dom_exception::create(
            scope,
            "Failed to construct 'SharedStorageDeleteMethod': Length of the \"key\" parameter is not valid".to_owned(),
            "DataError".to_owned(),
        ) {
            scope.throw_exception(exception.into());
        }
        return;
    }
    let with_lock =
        super::shared_storage_modifier_method::option_string(scope, arguments.get(1), "withLock");
    super::shared_storage_modifier_method::attach(
        scope,
        arguments.this(),
        super::shared_storage_modifier_method::SharedStorageOperation::Delete { key, with_lock },
    );
    result.set(arguments.this().into());
}
