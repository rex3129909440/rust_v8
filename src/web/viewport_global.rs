use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ViewportGlobalStore {
    values: HashMap<i32, v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ViewportGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let value = super::viewport::create(scope)?;
    let global_value: v8::Local<v8::Value> = value.into();
    let stored_value = v8::Global::new(scope, global_value);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<ViewportGlobalStore>()
        .ok_or_else(|| "viewport state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored_value);
    let getter = crate::webidl::create_function(
        scope,
        "get viewport",
        0,
        v8::ConstructorBehavior::Throw,
        get_viewport,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set viewport",
        1,
        v8::ConstructorBehavior::Throw,
        set_viewport,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "viewport")?;
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.viewport".to_owned())
    }
}

fn get_viewport(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<ViewportGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value));
    }
}

fn set_viewport(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = v8::Global::new(scope, arguments.get(0));
    let realm_id = crate::webidl::realm_id(scope);
    if let Some(store) = scope.get_slot_mut::<ViewportGlobalStore>() {
        store.values.insert(realm_id, value);
    }
}
