#[derive(Default)]
pub(crate) struct OnBlurStore {
    handler: Option<v8::Global<v8::Value>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OnBlurStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get onblur",
        0,
        v8::ConstructorBehavior::Throw,
        get_onblur,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set onblur",
        1,
        v8::ConstructorBehavior::Throw,
        set_onblur,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "onblur")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.onblur".to_owned())
    }
}

pub(crate) fn dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "blur" {
        return;
    }
    let global = scope.get_current_context().global(scope);
    if target.get_identity_hash().get() != global.get_identity_hash().get() {
        return;
    }
    let handler = scope
        .get_slot::<OnBlurStore>()
        .and_then(|store| store.handler.clone());
    super::window_event_handler_support::invoke(scope, target, event, handler);
}

fn get_onblur(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(handler) = scope
        .get_slot::<OnBlurStore>()
        .and_then(|store| store.handler.clone())
    {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}

fn set_onblur(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<OnBlurStore>() {
        store.handler = handler;
    }
}
