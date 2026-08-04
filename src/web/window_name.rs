#[derive(Default)]
pub(crate) struct WindowNameStore {
    value: String,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowNameStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get name",
        0,
        v8::ConstructorBehavior::Throw,
        get_name,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set name",
        1,
        v8::ConstructorBehavior::Throw,
        set_name,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "name")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.name".to_owned())
    }
}

fn get_name(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = super::html_i_frame_element::current_name(scope).unwrap_or_else(|| {
        scope
            .get_slot::<WindowNameStore>()
            .map(|store| store.value.clone())
            .unwrap_or_default()
    });
    if let Some(value) = v8::String::new(scope, &value) {
        result.set(value.into());
    }
}

fn set_name(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let value = crate::webidl::value_to_string(scope, arguments.get(0));
    if super::html_i_frame_element::set_current_name(scope, value.clone()) {
        return;
    }
    if let Some(store) = scope.get_slot_mut::<WindowNameStore>() {
        store.value = value;
    }
}
