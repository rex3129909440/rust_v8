pub(crate) fn define(
    s: &mut v8::PinScope<'_, '_>,
    p: v8::Local<'_, v8::Object>,
) -> Result<(), String> {
    crate::webidl::define_accessor(s, p, "onerror", get, set)
}
fn get(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, r: v8::ReturnValue<'_>) {
    super::shared_worker::get_onerror(s, a, r);
}
fn set(s: &mut v8::PinScope<'_, '_>, a: v8::FunctionCallbackArguments<'_>, _: v8::ReturnValue<'_>) {
    super::shared_worker::set_onerror(s, a);
}
