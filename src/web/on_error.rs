#[derive(Default)]
pub(crate) struct OnErrorStore {
    handler: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OnErrorStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::window_event_handler_support::define(scope, "onerror", get_handler, set_handler)
}
pub(crate) fn dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "error" || !super::window_event_handler_support::is_window(scope, target) {
        return;
    }
    let handler = scope
        .get_slot::<OnErrorStore>()
        .and_then(|store| store.handler.clone());
    let Some(handler) = handler else {
        return;
    };
    let local = v8::Local::new(scope, &handler);
    let Ok(function) = v8::Local::<v8::Function>::try_from(local) else {
        return;
    };
    let message = event_property(scope, event, "message");
    let filename = event_property(scope, event, "filename");
    let lineno = event_property(scope, event, "lineno");
    let colno = event_property(scope, event, "colno");
    let error = event_property(scope, event, "error");
    let canceled = {
        v8::tc_scope!(let try_catch, scope);
        function
            .call(
                try_catch,
                target.into(),
                &[message, filename, lineno, colno, error],
            )
            .is_some_and(|value| value.is_boolean() && value.boolean_value(try_catch))
    };
    if canceled {
        super::event::cancel(scope, event);
    }
}
fn event_property<'s>(
    scope: &v8::PinScope<'s, '_>,
    event: v8::Local<'s, v8::Object>,
    name: &str,
) -> v8::Local<'s, v8::Value> {
    let Some(key) = v8::String::new(scope, name) else {
        return v8::undefined(scope).into();
    };
    event
        .get(scope, key.into())
        .unwrap_or_else(|| v8::undefined(scope).into())
}
fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    mut result: v8::ReturnValue<'_>,
) {
    if let Some(handler) = scope
        .get_slot::<OnErrorStore>()
        .and_then(|store| store.handler.clone())
    {
        result.set(v8::Local::new(scope, &handler));
    } else {
        result.set(v8::null(scope).into());
    }
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<OnErrorStore>() {
        store.handler = handler;
    }
}
