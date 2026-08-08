use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct ScreenGlobalStore {
    values: HashMap<i32, v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(ScreenGlobalStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let realm_id = realm_id(scope);
    let profile = crate::fingerprint::edge(scope).screen.clone();
    let screen = super::screen::create(scope, &profile)?;
    let stored = v8::Global::new(scope, screen);
    scope
        .get_slot_mut::<ScreenGlobalStore>()
        .ok_or_else(|| "screen global state was not prepared".to_owned())?
        .values
        .insert(realm_id, stored);
    let getter = crate::webidl::create_function(
        scope,
        "get screen",
        0,
        v8::ConstructorBehavior::Throw,
        get_screen,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set screen",
        1,
        v8::ConstructorBehavior::Throw,
        set_screen,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "screen")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.screen".to_owned())
    }
}

fn get_screen(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    let value = scope
        .get_slot::<ScreenGlobalStore>()
        .and_then(|store| store.values.get(&realm_id(scope)))
        .cloned();
    if let Some(value) = value {
        result.set(v8::Local::new(scope, &value).into());
    }
}

fn set_screen(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    let Some(key) = v8::String::new(scope, "screen") else {
        return;
    };
    let _ = global.define_own_property(
        scope,
        key.into(),
        arguments.get(0),
        v8::PropertyAttribute::NONE,
    );
}

fn realm_id(scope: &v8::PinScope<'_, '_>) -> i32 {
    crate::webidl::realm_id(scope)
}
