#[derive(Default)]
pub(crate) struct OnVolumeChangeStore {
    handler: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(isolate: &mut v8::OwnedIsolate) {
    isolate.set_slot(OnVolumeChangeStore::default());
}
pub(crate) fn install(scope: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::window_event_handler_support::define(scope, "onvolumechange", get_handler, set_handler)
}
pub(crate) fn dispatch(
    scope: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    event_type: &str,
) {
    if event_type != "volumechange"
        || !super::window_event_handler_support::is_window(scope, target)
    {
        return;
    }
    let handler = scope
        .get_slot::<OnVolumeChangeStore>()
        .and_then(|store| store.handler.clone());
    super::window_event_handler_support::invoke(scope, target, event, handler);
}
fn get_handler(
    scope: &mut v8::PinScope<'_, '_>,
    _: v8::FunctionCallbackArguments<'_>,
    result: v8::ReturnValue<'_>,
) {
    let handler = scope
        .get_slot::<OnVolumeChangeStore>()
        .and_then(|store| store.handler.clone());
    super::window_event_handler_support::return_handler(scope, handler, result);
}
fn set_handler(
    scope: &mut v8::PinScope<'_, '_>,
    arguments: v8::FunctionCallbackArguments<'_>,
    _: v8::ReturnValue<'_>,
) {
    let handler = super::window_event_handler_support::handler_value(scope, arguments.get(0));
    if let Some(store) = scope.get_slot_mut::<OnVolumeChangeStore>() {
        store.handler = handler;
    }
}
