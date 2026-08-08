#[derive(Default)]
pub(crate) struct WindowStatusStore {
    value: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowStatusStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get status",
        0,
        v8::ConstructorBehavior::Throw,
        get_status,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set status",
        1,
        v8::ConstructorBehavior::Throw,
        set_status,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "status")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.status".to_owned())
    }
}

fn get_status(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<WindowStatusStore>()
        .map(|store| store.value.clone())
        .unwrap_or_default();
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_status(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<WindowStatusStore>() {
        store.value = value;
    }
}
