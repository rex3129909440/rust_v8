#[derive(Default)]
pub(crate) struct WindowEventStore {
    dispatch_stack: Vec<v8::Global<v8::Object>>,
}

pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(WindowEventStore::default());
}

pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    let getter = crate::webidl::create_function(
        scope,
        "get event",
        0,
        v8::ConstructorBehavior::Throw,
        get_event,
    )?;
    let setter = crate::webidl::create_function(
        scope,
        "set event",
        1,
        v8::ConstructorBehavior::Throw,
        set_event,
    )?;
    let mut descriptor = v8::PropertyDescriptor::new_from_get_set(getter.into(), setter.into());
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    let key = crate::webidl::string(scope, "event")?;
    let global = scope.get_current_context().global(scope);
    if global.define_property(scope, key.into(), &descriptor) == Some(true) {
        Ok(())
    } else {
        Err("cannot define window.event".to_owned())
    }
}

pub(crate) fn begin_dispatch(scope: &mut v8::PinScope<'_, '_>, event: v8::Local<'_, v8::Object>) {
    let stored = v8::Global::new(scope, event);
    if let Some(store) = scope.get_slot_mut::<WindowEventStore>() {
        store.dispatch_stack.push(stored);
    }
}

pub(crate) fn finish_dispatch(scope: &mut v8::PinScope<'_, '_>) {
    if let Some(store) = scope.get_slot_mut::<WindowEventStore>() {
        let _ = store.dispatch_stack.pop();
    }
}

fn get_event(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(event) = scope
        .get_slot::<WindowEventStore>()
        .and_then(|store| store.dispatch_stack.last())
        .cloned()
    {
        result.set(v8::Local::new(scope, &event).into());
    }
}

fn set_event(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let global = scope.get_current_context().global(scope);
    let Some(key) = v8::String::new(scope, "event") else {
        return;
    };
    let _ = global.define_own_property(
        scope,
        key.into(),
        arguments.get(0),
        v8::PropertyAttribute::NONE,
    );
}
