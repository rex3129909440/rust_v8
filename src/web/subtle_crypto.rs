#[derive(Default)]
pub(crate) struct SubtleCryptoStore {
    constructors: std::collections::HashMap<i32, v8::Global<v8::Function>>,
    instances: std::collections::HashSet<i32>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(SubtleCryptoStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let constructor = ensure_constructor(scope)?;
    crate::webidl::define_global(scope, "SubtleCrypto", constructor.into())
}

fn ensure_constructor<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Function>, String> {
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(constructor) = scope
        .get_slot::<SubtleCryptoStore>()
        .and_then(|store| store.constructors.get(&realm_id))
        .cloned()
    {
        return Ok(v8::Local::new(scope, &constructor));
    }
    let constructor = crate::webidl::create_function(
        scope,
        "SubtleCrypto",
        0,
        v8::ConstructorBehavior::Allow,
        illegal_constructor,
    )?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    crate::webidl::reset_constructor_order(scope, prototype)?;
    super::subtle_crypto_decrypt::define(scope, prototype)?;
    super::subtle_crypto_derive_bits::define(scope, prototype)?;
    super::subtle_crypto_derive_key::define(scope, prototype)?;
    super::subtle_crypto_digest::define(scope, prototype)?;
    super::subtle_crypto_encrypt::define(scope, prototype)?;
    super::subtle_crypto_export_key::define(scope, prototype)?;
    super::subtle_crypto_generate_key::define(scope, prototype)?;
    super::subtle_crypto_import_key::define(scope, prototype)?;
    super::subtle_crypto_sign::define(scope, prototype)?;
    super::subtle_crypto_unwrap_key::define(scope, prototype)?;
    super::subtle_crypto_verify::define(scope, prototype)?;
    super::subtle_crypto_wrap_key::define(scope, prototype)?;
    crate::webidl::finish_constructor(scope, prototype, constructor)?;
    let stored = v8::Global::new(scope, constructor);
    scope
        .get_slot_mut::<SubtleCryptoStore>()
        .ok_or_else(|| "SubtleCrypto state missing".to_owned())?
        .constructors
        .insert(realm_id, stored);
    Ok(constructor)
}

fn illegal_constructor(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    crate::webidl::throw_type_error(
        scope,
        "Failed to construct 'SubtleCrypto': Illegal constructor",
    );
}

pub(crate) fn create<'s>(
    scope: &mut v8::PinScope<'s, '_>,
) -> Result<v8::Local<'s, v8::Object>, String> {
    let constructor = ensure_constructor(scope)?;
    let prototype = crate::webidl::prototype(scope, constructor)?;
    let object = v8::Object::new(scope);
    if crate::webidl::set_platform_prototype(scope, object, prototype.into()) != Some(true) {
        return Err("cannot create SubtleCrypto".to_owned());
    }
    scope
        .get_slot_mut::<SubtleCryptoStore>()
        .ok_or_else(|| "SubtleCrypto state missing".to_owned())?
        .instances
        .insert(object.get_identity_hash().get());
    Ok(object)
}

pub(crate) fn valid(scope: &v8::PinScope<'_, '_>, object: v8::Local<'_, v8::Object>) -> bool {
    scope
        .get_slot::<SubtleCryptoStore>()
        .is_some_and(|store| store.instances.contains(&object.get_identity_hash().get()))
}
