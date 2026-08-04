#[derive(Default)]
pub(crate) struct OnBeforeUnloadStore {
    handler: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OnBeforeUnloadStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::window_event_handler_support::define(scope, "onbeforeunload", get_handler, set_handler)
}
pub(crate) fn dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "beforeunload"
        || !super::window_event_handler_support::is_window(scope, target)
    {
        return;
    }
    let handler = scope
        .get_slot::<OnBeforeUnloadStore>()
        .and_then(|store| store.handler.clone());
    let Some(handler) = handler else {
        return;
    };
    let local = v8::Local::new(scope, &handler);
    if let Ok(function) = v8::Local::<v8::Function>::try_from(local) {
        v8::tc_scope!(let try_catch, scope);
        let _ = function.call(try_catch, target.into(), &[event.into()]);
    }
}
fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = scope
        .get_slot::<OnBeforeUnloadStore>()
        .and_then(|store| store.handler.clone());
    super::window_event_handler_support::return_handler(scope, handler, result);
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<OnBeforeUnloadStore>() {
        store.handler = handler;
    }
}
