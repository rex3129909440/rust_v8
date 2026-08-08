use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct CryptoGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(CryptoGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm_id = realm_id(scope);
    let crypto = super::crypto::create(scope)?;
    let stored = v8::Global::new(scope, crypto);
    scope
        .get_slot_mut::<CryptoGlobalStore>()
        .ok_or_else(|| "crypto global state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get crypto",
        0,
        v8::ConstructorBehavior::Throw,
        get_crypto,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "crypto")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.crypto".to_owned())
    }
}

fn get_crypto(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<CryptoGlobalStore>()
        .and_then(|store| store.values.get(&realm_id(scope)))
        .cloned();
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    }
}

fn realm_id(scope: &v8::PinScope<'_, '_>) -> i32 {
    crate::webidl::realm_id(scope)
}
