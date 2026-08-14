#[derive(Default)]
pub(crate) struct Store {
    handler: Option<v8::Global<v8::Value>>,
}
pub(crate) fn prepare(i: &mut v8::OwnedIsolate) {
    i.set_slot(Store::default());
}
pub(crate) fn install(s: &mut v8::PinScope<'_, '_>) -> Result<(), String> {
    super::window_event_handler_support::define(s, "onorientationchange", get, set)
}
pub(crate) fn dispatch(
    s: &mut v8::PinScope<'_, '_>,
    target: v8::Local<'_, v8::Object>,
    event: v8::Local<'_, v8::Object>,
    kind: &str,
) {
    if kind == "orientationchange" && super::window_event_handler_support::is_window(s, target) {
        let h = s.get_slot::<Store>().and_then(|x| x.handler.clone());
        super::window_event_handler_support::invoke(s, target, event, h);
    }
}
fn get(s: &mut v8::PinScope<'_, '_>, _: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    let h = s.get_slot::<Store>().and_then(|x| x.handler.clone());
    super::window_event_handler_support::return_handler(s, h, r)
}
fn set(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, _: v8::ReturnValue<'_>) {
    let h = super::window_event_handler_support::handler_value(s, a.get(0));
    s.get_slot_mut::<Store>().unwrap().handler = h;
}
