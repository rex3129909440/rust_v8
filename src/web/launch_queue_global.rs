use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct LaunchQueueGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(LaunchQueueGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let value = super::launch_queue::create(scope)?;
    let stored = v8::Global::new(scope, value);
    let realm_id = crate::webidl::realm_id(scope);
    scope
        .get_slot_mut::<LaunchQueueGlobalStore>()
        .ok_or_else(|| "launchQueue state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get launchQueue",
        0,
        v8::ConstructorBehavior::Throw,
        get_launch_queue,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let global = scope.get_current_context().global(scope);
    let key = crate::webidl::string(scope, "launchQueue")?;
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.launchQueue".to_owned())
    }
}

fn get_launch_queue(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = scope
        .get_slot::<LaunchQueueGlobalStore>()
        .and_then(|store| store.values.get(&crate::webidl::realm_id(scope)))
        .cloned()
    {
        result.set(v8::Local::new(scope, &value).into());
    }
}
