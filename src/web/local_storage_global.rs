use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LocalStorageGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LocalStorageGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get localStorage",
        0,
        v8::ConstructorBehavior::Throw,
        get_local_storage,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "localStorage")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.localStorage".to_owned())
    }
}
fn get_local_storage(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let existing = scope
        .get_slot::<LocalStorageGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned();
    if let Some(existing) = existing {
        result.set(v8::Local::new(scope, &existing).into());
        return;
    }
    let Ok(storage) = super::storage::create_local(scope) else {
        crate::webidl::throw_type_error(scope, "Cannot create localStorage");
        return;
    };
    let stored = v8::Global::new(scope, storage);
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(store) = scope.get_slot_mut::<LocalStorageGlobalStore>() {
        store.values.insert(realm_id, stored);
        result.set(storage.into());
    } else {
        crate::webidl::throw_type_error(scope, "localStorage state was not prepared");
    }
}
