use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct NavigatorGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(NavigatorGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm_id = realm_id(scope);
    let navigator = super::navigator::create(scope)?;
    let stored = v8::Global::new(scope, navigator);
    scope
        .get_slot_mut::<NavigatorGlobalStore>()
        .ok_or_else(|| "navigator global state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get navigator",
        0,
        v8::ConstructorBehavior::Throw,
        get_navigator,
    )?;
    let undefined = v8::undefined(scope);
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), undefined.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "navigator")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.navigator".to_owned())
    }
}

pub(crate) fn value<'s>(scope: &mut v8::PinScope<'s, '_>) -> Option<v8::Local<'s, v8::Object>> {
    let value = scope
        .get_slot::<NavigatorGlobalStore>()?
        .values
        .get(&realm_id(scope))?
        .clone();
    Some(v8::Local::new(scope, &value))
}

fn get_navigator(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(value) = value(scope) {
        result.set(value.into());
    }
}

fn realm_id(scope: &v8::PinScope<'_, '_>) -> i32 {
    crate::webidl::realm_id(scope)
}
