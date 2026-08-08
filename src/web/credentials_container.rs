use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CredentialsContainerStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, Vec<v8::Global<v8::Object>>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CredentialsContainerStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "CredentialsContainer", constructor.into())
}
fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(constructor) = scope
        .get_slot::<CredentialsContainerStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "CredentialsContainer",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_method(scope, prototype, "create", 0, create_credential)?;
    crate::webidl::define_method(scope, prototype, "get", 0, get)?;
    crate::webidl::define_method(
        scope,
        prototype,
        "preventSilentAccess",
        0,
        prevent_silent_access,
    )?;
    crate::webidl::define_method(scope, prototype, "store", 1, store)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<CredentialsContainerStore>()
        .ok_or_else(|| "CredentialsContainer state was not prepared".to_owned())?
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
        "Failed to construct 'CredentialsContainer': Illegal constructor",
    );
}
pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create CredentialsContainer".to_owned());
    }
    scope
        .get_slot_mut::<CredentialsContainerStore>()
        .ok_or_else(|| "CredentialsContainer state was not prepared".to_owned())?
        .records
        .insert(object.get_identity_hash().get(), Vec::new());
    Ok(object)
}
fn resolve(
    scope: &mut v8::PinScope<'_, '_>,
    value: v8::Local<'_, v8::Value>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Ok(promise) = super::writable_stream::resolved_promise(scope, value) {
        result.set(promise.into());
    }
}
fn create_credential(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<CredentialsContainerStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    resolve(scope, v8::null(scope).into(), result);
}
fn get(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let credential = scope
        .get_slot::<CredentialsContainerStore>()
        .and_then(|store| {
            store
                .records
                .get(&arguments.this().get_identity_hash().get())
        })
        .and_then(|credentials| credentials.last())
        .cloned();
    match credential {
        Some(credential) => {
            let credential = v8::Local::new(scope, &credential);
            resolve(scope, credential.into(), result);
        }
        None => resolve(scope, v8::null(scope).into(), result),
    }
}
fn store(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let Ok(credential) = v8::Local::<v8::Object>::try_from(arguments.get(0)) else {
        crate::webidl::throw_type_error(scope, "CredentialsContainer.store requires a Credential");
        return;
    };
    let stored = v8::Global::new(scope, credential);
    let Some(credentials) = scope
        .get_slot_mut::<CredentialsContainerStore>()
        .and_then(|store| {
            store
                .records
                .get_mut(&arguments.this().get_identity_hash().get())
        })
    else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    credentials.push(stored);
    resolve(scope, credential.into(), result);
}
fn prevent_silent_access(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    if !scope
        .get_slot::<CredentialsContainerStore>()
        .is_some_and(|store| {
            store
                .records
                .contains_key(&arguments.this().get_identity_hash().get())
        })
    {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    }
    resolve(scope, v8::undefined(scope).into(), result);
}
