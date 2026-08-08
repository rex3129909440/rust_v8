use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LocationbarGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LocationbarGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let bar = super::bar_prop::create(scope, true)?;
    let stored = v8::Global::new(scope, bar);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<LocationbarGlobalStore>()
        .ok_or_else(|| "locationbar state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get locationbar",
        0,
        v8::ConstructorBehavior::Throw,
        get_locationbar,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set locationbar",
        1,
        v8::ConstructorBehavior::Throw,
        set_locationbar,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "locationbar")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.locationbar".to_owned())
    }
}

fn get_locationbar(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<LocationbarGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    }
}

fn set_locationbar(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    let Some(key) = v8::String::new(scope, "locationbar") else {
        return;
    };
    let _ = global.define_own_property(
        scope,
        key.into(),
        arguments.get(0),
        v8::PropertyAttribute::NONE,
    );
}
