use std::collections::HashMap;

#[derive(Clone)]
pub(crate) enum SharedStorageOperation {
    Append {
        key: String,
        value: String,
        with_lock: Option<String>,
    },
    Clear {
        with_lock: Option<String>,
    },
    Delete {
        key: String,
        with_lock: Option<String>,
    },
    Set {
        key: String,
        value: String,
        ignore_if_present: bool,
        with_lock: Option<String>,
    },
}

#[derive(Default)]
pub(crate) struct SharedStorageModifierMethodStore {
    constructor: crate::webidl::RealmConstructor,
    operations: HashMap<i32, SharedStorageOperation>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SharedStorageModifierMethodStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SharedStorageModifierMethod", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<SharedStorageModifierMethodStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SharedStorageModifierMethod",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SharedStorageModifierMethodStore>()
        .ok_or_else(|| "SharedStorageModifierMethod state was not prepared".to_owned())?
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
        "Failed to construct 'SharedStorageModifierMethod': Illegal constructor",
    )
}

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    operation: SharedStorageOperation,
) {
    scope
        .get_slot_mut::<SharedStorageModifierMethodStore>()
        .expect("SharedStorageModifierMethod state")
        .operations
        .insert(object.get_identity_hash().get(), operation);
}

pub(crate) fn operation(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<SharedStorageOperation> {
    scope
        .get_slot::<SharedStorageModifierMethodStore>()?
        .operations
        .get(&object.get_identity_hash().get())
        .cloned()
}

pub(crate) fn option_string(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> Option<String> {
    let object = v8::Local::<v8::Object>::try_from(value).ok()?;
    let key = v8::String::new(scope, name)?;
    let value = object.get(scope, key.into())?;
    if value.is_undefined() {
        None
    } else {
        Some(crate::webidl::value_to_string(scope, value))
    }
}

pub(crate) fn option_bool(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    name: &str,
) -> bool {
    let Ok(object) = v8::Local::<v8::Object>::try_from(value) else {
        return false;
    };
    let Some(key) = v8::String::new(scope, name) else {
        return false;
    };
    object
        .get(scope, key.into())
        .is_some_and(|value| !value.is_undefined() && value.boolean_value(scope))
}
