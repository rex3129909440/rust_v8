use std::collections::HashMap;

#[derive(Clone)]
struct ResponseRecord {
    client_data: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct AuthenticatorResponseStore {
    constructor: crate::webidl::RealmConstructor,
    records: HashMap<i32, ResponseRecord>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(AuthenticatorResponseStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "AuthenticatorResponse", constructor.into())
}

pub(crate) fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    if let Some(value) = scope
        .get_slot::<AuthenticatorResponseStore>()
        .and_then(|store| store.constructor.get(crate::webidl::realm_id(scope)))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &value));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "AuthenticatorResponse",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    crate::webidl::define_readonly_accessor(
        scope,
        prototype,
        "clientDataJSON",
        get_client_data_json,
    )?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let realm_id = crate::webidl::realm_id(scope);
    let realm_constructor = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<AuthenticatorResponseStore>()
        .ok_or_else(|| "AuthenticatorResponse state missing".to_owned())?
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

pub(crate) fn attach(
    scope: &mut v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
    client_data: Vec<u8>,
) {
    scope
        .get_slot_mut::<AuthenticatorResponseStore>()
        .expect("AuthenticatorResponse state")
        .records
        .insert(
            object.get_identity_hash().get(),
            ResponseRecord { client_data },
        );
}

pub(crate) fn client_data(
    scope: &v8::PinScope<'_, '_>,
    object: v8::Local<'_, v8::Object>,
) -> Option<Vec<u8>> {
    Some(
        scope
            .get_slot::<AuthenticatorResponseStore>()?
            .records
            .get(&object.get_identity_hash().get())?
            .client_data
            .clone(),
    )
}

fn get_client_data_json(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let Some(bytes) = client_data(scope, arguments.this()) else {
        crate::webidl::throw_type_error(scope, "Illegal invocation");
        return;
    };
    let backing = v8::ArrayBuffer::new_backing_store_from_vec(bytes).make_shared();
    let buffer = v8::ArrayBuffer::with_backing_store(scope, &backing);
    result.set(buffer.into());
}
